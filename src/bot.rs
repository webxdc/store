//! Entry for the bot code
use anyhow::{Context as _, Result};
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    config::Config,
    context::Context,
    message::{Message, MsgId, Viewtype},
    securejoin,
    stock_str::StockStrings,
    EventType, Events,
};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};
use std::{collections::HashSet, fs, path::PathBuf, sync::Arc};

use crate::{
    db::{self, MIGRATOR},
    messages::store_message,
    request_handlers::{
        genisis, review, shop, submit, ChatType, GeneralFrontendRequest, GeneralFrontendResponse,
        WebxdcStatusUpdate,
    },
    utils::{
        configure_from_env, read_webxdc_versions, send_newest_updates, send_update_payload_only,
        send_webxdc, Webxdc, WebxdcVersions,
    },
    DB_URL, DC_DB_PATH, GENESIS_QR, INVITE_QR, VERSION,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct BotConfig {
    pub genesis_qr: String,
    pub invite_qr: String,
    pub tester_group: ChatId,
    pub reviewee_group: ChatId,
    pub genesis_group: ChatId,
    pub serial: i32,
    pub shop_xdc_version: String,
    pub submit_xdc_version: String,
    pub review_xdc_version: String,
}

/// Github Bot state
pub struct State {
    pub db: SqlitePool,
    pub config: BotConfig,
    pub webxdc_versions: WebxdcVersions,
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
        std::fs::create_dir(DC_DB_PATH).ok();
        let dbfile = PathBuf::from(DC_DB_PATH).join("db.sqlite");
        let context = Context::new(dbfile.as_path(), 1, Events::new(), StockStrings::new())
            .await
            .context("Failed to create context")?;

        if !context.get_config_bool(Config::Configured).await? {
            info!("DC: Start configuring...");
            configure_from_env(&context).await?;
            info!("DC: Configuration done");
        }

        let db_path = PathBuf::from(
            DB_URL
                .split("://")
                .nth(1)
                .context("Failed to extract path from db")?,
        );
        if !db_path.exists() {
            fs::create_dir(db_path.parent().context("db_path has no parant")?)?;
            fs::write(db_path, "")?;
        }

        let db = SqlitePool::connect(DB_URL).await?;
        MIGRATOR.run(&db).await?;

        let config = match db::get_config(&mut *db.acquire().await?).await {
            Ok(config) => config,
            Err(_) => {
                info!("Bot hasn't been configured yet, start configuring...");
                let config = Self::setup(&context).await.context("Failed to setup bot")?;
                let conn = &mut *db.acquire().await?;
                db::set_config(&mut *db.acquire().await?, &config).await?;

                // set chat types
                db::set_chat_type(conn, config.genesis_group, ChatType::Genesis).await?;
                db::set_chat_type(conn, config.reviewee_group, ChatType::ReviewPool).await?;
                db::set_chat_type(conn, config.tester_group, ChatType::TesterPool).await?;

                // save qr codes to disk
                qrcode_generator::to_png_to_file(
                    &config.genesis_qr,
                    QrCodeEcc::Low,
                    1024,
                    GENESIS_QR,
                )
                .context("failed to generate genesis QR")?;
                println!("Generated genisis group join QR-code at {GENESIS_QR}");
                qrcode_generator::to_png_to_file(
                    &config.invite_qr,
                    QrCodeEcc::Low,
                    1024,
                    INVITE_QR,
                )
                .context("failed to generate invite QR")?;
                println!("Generated 1:1 invite QR-code at {INVITE_QR}");
                config
            }
        };

        let webxdc_versions = read_webxdc_versions().await.map_err(|e| {
            anyhow::anyhow!("Problem while parsing one of the store `manifests.toml`s: \n {e}")
        })?;
        info!("Loaded webxdc versions: {:?}", webxdc_versions);

        Ok(Self {
            dc_ctx: context,
            state: Arc::new(State {
                db,
                config,
                webxdc_versions,
            }),
        })
    }

    /// Creates special groups and returns the complete bot config.
    async fn setup(context: &Context) -> Result<BotConfig> {
        let genesis_group =
            chat::create_group_chat(context, ProtectionStatus::Protected, "Appstore: Genesis")
                .await?;

        let reviewee_group =
            chat::create_group_chat(context, ProtectionStatus::Protected, "Appstore: Publishers")
                .await?;

        let tester_group =
            chat::create_group_chat(context, ProtectionStatus::Protected, "Appstore: Testers")
                .await?;

        let genesis_qr = securejoin::get_securejoin_qr(context, Some(genesis_group)).await?;

        let invite_qr = securejoin::get_securejoin_qr(context, None).await?;

        Ok(BotConfig {
            genesis_qr,
            invite_qr,
            reviewee_group,
            genesis_group,
            tester_group,
            serial: 0,
            ..Default::default()
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
        info!("successfully started bot! 🥳");
    }

    /// Handle dc-events.
    async fn dc_event_handler(
        context: &Context,
        state: Arc<State>,
        event: EventType,
    ) -> anyhow::Result<()> {
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
                        match chat_type {
                            ChatType::Genesis => {
                                info!("updating genesis contacts");
                                db::set_genesis_members(conn, &filtered.collect::<Vec<_>>())
                                    .await?;
                            }
                            ChatType::ReviewPool => {
                                info!("updating reviewer contacts");
                                db::set_publishers(conn, &filtered.collect::<Vec<_>>()).await?;
                            }
                            ChatType::TesterPool => {
                                info!("updating tester contacts");
                                db::set_testers(conn, &filtered.collect::<Vec<_>>()).await?;
                            }
                            ChatType::Submit => {
                                let new_members = filtered.collect::<HashSet<_>>();
                                if new_members.len() == 1 {
                                    info!("submit chat has only one member left, deleting");
                                    db::delete_submit_chat(
                                        &mut *state.db.acquire().await?,
                                        chat_id,
                                    )
                                    .await?;
                                    chat_id.delete(context).await?;
                                }
                            }
                            ChatType::Review => {
                                let mut conn = state.db.acquire().await?;
                                let review_chat = db::get_review_chat(&mut conn, chat_id).await?;
                                let new_members = filtered.collect::<HashSet<_>>();
                                let old_members: HashSet<_> =
                                    review_chat.get_members().into_iter().collect();
                                let removed_members = old_members.difference(&new_members);
                                for removed_member in removed_members {
                                    // handle publisher removal
                                    if *removed_member == review_chat.publisher {
                                        match db::get_new_random_publisher(
                                            &mut conn,
                                            *removed_member,
                                        )
                                        .await
                                        {
                                            Ok(new_publisher) => {
                                                db::set_review_chat_publisher(
                                                    &mut conn,
                                                    chat_id,
                                                    new_publisher,
                                                )
                                                .await?;
                                            }
                                            Err(_) => warn!(
                                                "Could not find a new publisher for chat {}",
                                                chat_id
                                            ),
                                        }
                                    }
                                    // handle tester removal
                                    else if review_chat.testers.contains(removed_member) {
                                        let mut new_tester =
                                            db::get_random_tester(&mut conn).await?;
                                        let mut count = 0;
                                        while new_tester == *removed_member || count >= 10 {
                                            new_tester = db::get_random_tester(&mut conn).await?;
                                            count += 1;
                                        }
                                        if count == 10 {
                                            warn!(
                                                "Could not find a new tester for chat {}",
                                                chat_id
                                            );
                                            continue;
                                        }
                                        let mut new_testers = review_chat
                                            .testers
                                            .iter()
                                            .copied()
                                            .filter(|t| t != removed_member)
                                            .collect_vec();

                                        new_testers.push(new_tester);

                                        db::set_review_chat_testers(
                                            &mut conn,
                                            chat_id,
                                            &new_testers,
                                        )
                                        .await?;
                                    }
                                }
                            }
                            _ => (),
                        };
                    }
                    Err(_e) => {
                        info!(
                        "Chat {chat_id} is not in the database, adding it as chat with type shop"
                    );
                        db::set_chat_type(conn, chat_id, ChatType::Shop).await?;
                        let msg = send_webxdc(
                            context,
                            &state,
                            chat_id,
                            Webxdc::Shop,
                            Some(store_message()),
                        )
                        .await?;
                        send_newest_updates(context, msg, &mut *state.db.acquire().await?, 0)
                            .await?;
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
        if let Some(text) = msg.get_text() {
            if text == "/version" {
                chat::send_text_msg(context, chat_id, VERSION.to_string()).await?;
                return Ok(());
            }
        }

        match db::get_chat_type(&mut *state.db.acquire().await?, chat_id).await {
            Ok(chat_type) => {
                info!("Handling message with type <{chat_type:?}>");
                match chat_type {
                    ChatType::Shop => {
                        let msg = Message::load_from_db(context, msg_id).await?;
                        if msg.get_viewtype() == Viewtype::Webxdc {
                            shop::handle_webxdc(context, state, msg).await?;
                        } else {
                            shop::handle_message(context, state, chat_id).await?;
                        }
                    }
                    ChatType::Submit => {
                        let msg = Message::load_from_db(context, msg_id).await?;
                        if msg.get_viewtype() == Viewtype::Webxdc {
                            submit::handle_webxdc(context, chat_id, state, msg).await?;
                        }
                    }
                    ChatType::Review => {}
                    ChatType::Genesis => {
                        genisis::handle_message(context, state, chat_id, msg_id).await?
                    }
                    ChatType::ReviewPool | ChatType::TesterPool => (),
                }
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    info!("creating new 1:1 chat with type Shop");
                    db::set_chat_type(&mut *state.db.acquire().await?, chat_id, ChatType::Shop)
                        .await?;
                    shop::handle_message(context, state, chat_id).await?;
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
    ) -> anyhow::Result<()> {
        let msg = Message::load_from_db(context, msg_id).await?;
        let chat_id = msg.get_chat_id();
        let conn = &mut *state.db.acquire().await?;
        let chat_type = db::get_chat_type(conn, chat_id).await?;
        let (webxdc, version) = db::get_webxdc_version(conn, msg.get_id()).await?;

        if let Ok(request) =
            serde_json::from_str::<WebxdcStatusUpdate<GeneralFrontendRequest>>(&update)
        {
            match request.payload {
                GeneralFrontendRequest::UpdateWebxdc => {
                    let msg = send_webxdc(context, &state, chat_id, webxdc, Some(store_message()))
                        .await?;
                    send_newest_updates(context, msg, &mut *state.db.acquire().await?, 0).await?;
                    send_update_payload_only(context, msg_id, GeneralFrontendResponse::UpdateSent)
                        .await?;
                    return Ok(());
                }
            }
        };

        if version != *state.webxdc_versions.get(webxdc) {
            info!("Webxdc version mismatch, updating");

            if serde_json::from_str::<WebxdcStatusUpdate<GeneralFrontendResponse>>(&update).is_ok()
            {
                return Ok(());
            }

            // Only try to upgrade version, if the webxdc event is _not_ an update response already
            if serde_json::from_str::<WebxdcStatusUpdate<GeneralFrontendRequest>>(&update).is_err()
            {
                send_update_payload_only(
                    context,
                    msg_id,
                    GeneralFrontendResponse::Outdated {
                        version: state.webxdc_versions.get(webxdc).to_string(),
                        critical: true,
                    },
                )
                .await?;
            };
            return Ok(());
        }

        match chat_type {
            ChatType::Submit => {
                submit::handle_status_update(context, state, chat_id, update).await?
            }
            ChatType::Shop => shop::handle_status_update(context, state, msg_id, update).await?,
            ChatType::ReviewPool | ChatType::TesterPool | ChatType::Genesis => (),
            ChatType::Review => {
                review::handle_status_update(context, state, chat_id, update).await?
            }
        }

        Ok(())
    }

    pub async fn get_db_connection(&self) -> sqlx::Result<PoolConnection<Sqlite>> {
        self.state.db.acquire().await
    }
}
