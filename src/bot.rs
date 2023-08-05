//! Entry for the bot code
use anyhow::{Context as _, Result};
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    config::Config,
    context::Context,
    message::{Message, MsgId},
    securejoin,
    stock_str::StockStrings,
    EventType, Events,
};
use log::{debug, error, info, trace, warn};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};
use std::{fs, sync::Arc};

use crate::{
    db::{self, MIGRATOR},
    project_dirs,
    request_handlers::{genesis, store, ChatType, WebxdcStatusUpdate, WebxdcStatusUpdatePayload},
    utils::{
        configure_from_env, get_icon_path, get_store_xdc_path, get_webxdc_tag_name, init_store,
        send_update_payload_only, unpack_assets, update_store,
    },
    GENESIS_QR, INVITE_QR, VERSION,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct BotConfig {
    pub genesis_qr: String,
    pub invite_qr: String,
    pub genesis_group: ChatId,
    pub serial: i32,
}

/// Github Bot state
pub struct State {
    pub db: SqlitePool,
    pub config: BotConfig,
    pub store_tag_name: String,
}

/// Github Bot
pub struct Bot {
    dc_ctx: Context,
    state: Arc<State>,
}

impl Bot {
    /// Creates a new instance of the bot.
    /// Handles the configuration for dc and the bot itself.
    pub async fn new() -> Result<Self> {
        if std::env::var("XDCSTORE_KEEP_ASSETS")
            .unwrap_or_default()
            .is_empty()
        {
            unpack_assets().context("failed to unpack assets")?;
        }

        let dirs = project_dirs()?;

        std::fs::create_dir_all(dirs.config_dir())?;
        let deltachat_db_file = dirs.config_dir().to_path_buf().join("deltachat.db");
        let context = Context::new(
            deltachat_db_file.as_path(),
            1,
            Events::new(),
            StockStrings::new(),
        )
        .await
        .context("Failed to create context")?;

        if !context.get_config_bool(Config::Configured).await? {
            info!("DC: Start configuring...");
            configure_from_env(&context).await?;
            info!("DC: Configuration done");
        }

        let bot_db_file = dirs.config_dir().to_path_buf().join("bot.db");
        if !bot_db_file.exists() {
            fs::write(&bot_db_file, "")?;
        }
        let bot_db_url = format!("sqlite://{}", bot_db_file.display());
        let db = SqlitePool::connect(&bot_db_url)
            .await
            .with_context(|| format!("connect to database pool {bot_db_url:?}"))?;
        MIGRATOR.run(&db).await?;

        let config = match db::get_config(&mut *db.acquire().await?).await {
            Ok(config) => config,
            Err(_) => {
                info!("Bot hasn't been configured yet, start configuring...");
                let config = Self::setup(&context).await.context("Failed to setup bot")?;
                let conn = &mut *db.acquire().await?;
                db::set_config(&mut *db.acquire().await?, &config).await?;

                // setc chat type for genesis group
                db::set_chat_type(conn, config.genesis_group, ChatType::Genesis).await?;

                // save qr codes to disk
                let dest_path = dirs.config_dir().to_path_buf().join(GENESIS_QR);
                qrcode_generator::to_png_to_file(
                    &config.genesis_qr,
                    QrCodeEcc::Low,
                    1024,
                    &dest_path,
                )
                .context("failed to generate genesis QR at {dest_path}")?;
                eprintln!(
                    "Generated genesis group join QR-code at {}",
                    dest_path.display()
                );

                let dest_path = dirs.config_dir().to_path_buf().join(INVITE_QR);
                qrcode_generator::to_png_to_file(
                    &config.invite_qr,
                    QrCodeEcc::Low,
                    1024,
                    &dest_path,
                )
                .context("failed to generate invite QR at {dest_path}")?;
                eprintln!("Generated 1:1 invite QR-code at {}", dest_path.display());
                config
            }
        };

        let store_xdc_path = get_store_xdc_path()?;
        let store_tag_name = get_webxdc_tag_name(&store_xdc_path).await?;
        info!("Store tag_name: {store_tag_name}");
        info!("Store frontend location: {}", store_xdc_path.display());

        Ok(Self {
            dc_ctx: context,
            state: Arc::new(State {
                db,
                config,
                store_tag_name,
            }),
        })
    }

    /// Creates genesis group and qr-codes.
    /// Returns the complete bot config.
    async fn setup(context: &Context) -> Result<BotConfig> {
        context
            .set_config(
                Config::Selfavatar,
                Some(
                    get_icon_path()?
                        .to_str()
                        .context("Can't convert image file")?,
                ),
            )
            .await?;

        let genesis_group =
            chat::create_group_chat(context, ProtectionStatus::Protected, "Appstore: Genesis")
                .await?;

        let genesis_qr = securejoin::get_securejoin_qr(context, Some(genesis_group)).await?;
        let invite_qr = securejoin::get_securejoin_qr(context, None).await?;

        Ok(BotConfig {
            genesis_qr,
            invite_qr,
            genesis_group,
            serial: 0,
        })
    }

    /// Start the bot.
    pub async fn start(&mut self) {
        let events_emitter = self.dc_ctx.get_event_emitter();
        let ctx = self.dc_ctx.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            while let Some(event) = events_emitter.recv().await {
                if let Err(e) = Self::dc_event_handler(&ctx, state.clone(), event.typ).await {
                    warn!("{}", e)
                }
            }
        });
        self.dc_ctx.start_io().await;
        info!("Successfully started bot! 🥳");
    }

    /// Handle dc-events.
    async fn dc_event_handler(
        context: &Context,
        state: Arc<State>,
        event: EventType,
    ) -> Result<()> {
        match event {
            EventType::ConfigureProgress { progress, comment } => {
                trace!("DC: Configuring progress: {progress} {comment:?}")
            }
            EventType::Info(msg) => trace!("DC: {msg}"),
            EventType::Warning(msg) => warn!("DC: {msg}"),
            EventType::Error(msg) => error!("DC: {msg}"),
            EventType::ConnectivityChanged => trace!("DC: ConnectivityChanged"),
            EventType::IncomingMsg { chat_id, msg_id } => {
                Self::handle_dc_message(context, state, chat_id, msg_id).await?
            }
            EventType::WebxdcStatusUpdate {
                msg_id,
                status_update_serial,
            } => {
                let update_string = context
                    .get_status_update(msg_id, status_update_serial)
                    .await?;

                Self::handle_dc_webxdc_update(context, state, msg_id, update_string).await?
            }
            EventType::ChatModified(chat_id) => {
                let conn = &mut *state.db.acquire().await?;
                match db::get_chat_type(conn, chat_id).await {
                    Ok(chat_type) => {
                        let contacts = chat::get_chat_contacts(context, chat_id).await?;
                        let filtered = contacts.into_iter().filter(|ci| !ci.is_special());
                        if chat_type == ChatType::Genesis {
                            info!("Updating genesis contacts");
                            db::set_genesis_members(conn, &filtered.collect::<Vec<_>>()).await?;
                        };
                    }
                    Err(_e) => {
                        info!(
                        "Chat {chat_id} is not in the database, adding it as chat with type store"
                    );
                        db::set_chat_type(conn, chat_id, ChatType::Store).await?;
                        init_store(context, &state, chat_id).await?;
                    }
                }
            }
            other => {
                debug!("DC: [unhandled event] {other:?}");
            }
        }
        Ok(())
    }

    /// Handles chat messages from clients.
    async fn handle_dc_message(
        context: &Context,
        state: Arc<State>,
        chat_id: ChatId,
        msg_id: MsgId,
    ) -> Result<()> {
        let msg = Message::load_from_db(context, msg_id).await?;
        if msg.get_text() == "/version" {
            chat::send_text_msg(context, chat_id, VERSION.to_string()).await?;
            return Ok(());
        }

        match db::get_chat_type(&mut *state.db.acquire().await?, chat_id).await {
            Ok(chat_type) => {
                info!("Handling message with type <{chat_type:?}>");
                match chat_type {
                    ChatType::Store => {
                        store::handle_message(context, state, chat_id).await?;
                    }
                    ChatType::Genesis => {
                        genesis::handle_message(context, state, chat_id, msg_id).await?
                    }
                }
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    info!("creating new 1:1 chat with type Store");
                    db::set_chat_type(&mut *state.db.acquire().await?, chat_id, ChatType::Store)
                        .await?;
                    store::handle_message(context, state, chat_id).await?;
                }
                _ => warn!("Problem while retrieving [ChatType]: {}", e),
            },
        }
        Ok(())
    }

    /// Handles webxdc updates from clients.
    async fn handle_dc_webxdc_update(
        context: &Context,
        state: Arc<State>,
        msg_id: MsgId,
        update: String,
    ) -> Result<()> {
        let msg = Message::load_from_db(context, msg_id).await?;
        let chat_id = msg.get_chat_id();
        let conn = &mut *state.db.acquire().await?;
        let chat_type = db::get_chat_type(conn, chat_id).await?;
        let store_tag_name = db::get_store_tag_name(conn, msg.get_id()).await?;

        let Ok(request) = serde_json::from_str::<WebxdcStatusUpdate>(&update) else {
            info!(
                "Ignoring WebXDC update: {}",
                &update.get(..100.min(update.len())).unwrap_or_default()
            );
            return Ok(());
        };

        if let WebxdcStatusUpdatePayload::UpdateWebxdc { serial } = request.payload {
            send_update_payload_only(context, msg_id, WebxdcStatusUpdatePayload::UpdateSent)
                .await?;
            update_store(context, &state, chat_id, serial).await?;
            return Ok(());
        }

        if store_tag_name != state.store_tag_name {
            info!(
                "Store xdc frontend's tag_name changed from {} to {}, triggering update",
                state.store_tag_name, store_tag_name
            );

            // Only try to upgrade version, if the webxdc event is _not_ an update response already
            if !matches!(request.payload, WebxdcStatusUpdatePayload::Outdated { .. }) {
                send_update_payload_only(
                    context,
                    msg_id,
                    WebxdcStatusUpdatePayload::Outdated {
                        tag_name: state.store_tag_name.clone(),
                        critical: true,
                    },
                )
                .await?;
            }
            return Ok(());
        }

        if chat_type == ChatType::Store {
            store::handle_status_update(context, state, msg_id, request.payload).await?
        }

        Ok(())
    }

    pub async fn get_db_connection(&self) -> sqlx::Result<PoolConnection<Sqlite>> {
        self.state.db.acquire().await
    }
}
