//! Entry for the bot code.

use anyhow::{Context as _, Result};
use deltachat::{
    chat::{self, ChatId},
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
use sqlx::{pool::PoolConnection, FromRow, Sqlite, SqlitePool};
use std::{fs, sync::Arc};

use crate::{
    db::{self, MIGRATOR},
    project_dirs,
    request_handlers::{store, WebxdcStatusUpdate, WebxdcStatusUpdatePayload},
    utils::{
        configure_from_env, get_icon_path, get_store_xdc_path, get_webxdc_tag_name,
        send_update_payload_only, unpack_assets, update_store,
    },
    INVITE_QR, VERSION,
};

/// Bot configuration.
#[derive(FromRow, Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct BotConfig {
    /// QR code for the contact setup.
    pub invite_qr: String,

    /// Serial number incremented each time an application index is changed.
    pub serial: i32,
}

/// Bot state.
pub struct State {
    /// SQLite connection pool.
    pub db: SqlitePool,

    /// Bot configuration.
    pub config: BotConfig,

    /// `tag_name` field from the `manifest.toml` of the `store.xdc`.
    pub store_tag_name: String,
}

/// Store bot.
pub struct Bot {
    /// Delta Chat account.
    dc_ctx: Context,

    /// Reference to the bot state.
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
                db::set_config(&mut *db.acquire().await?, &config).await?;

                // Save QR code to disk.
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

    /// Sets avatar and creates a QR code.
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

        let invite_qr = securejoin::get_securejoin_qr(context, None).await?;

        Ok(BotConfig {
            invite_qr,
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

        info!("Handling message {msg_id}.");
        store::handle_message(context, state, chat_id).await?;

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
                store_tag_name, state.store_tag_name
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

        store::handle_status_update(context, state, msg_id, request.payload).await?;

        Ok(())
    }

    /// Retrieves a database connection from the pool.
    pub async fn get_db_connection(&self) -> sqlx::Result<PoolConnection<Sqlite>> {
        self.state.db.acquire().await
    }
}
