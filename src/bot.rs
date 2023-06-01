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
use log::{debug, error, info, trace, warn};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

use crate::{
    db::DB,
    messages::appstore_message,
    request_handlers::{genisis, review, shop, submit, ChatType},
    utils::{configure_from_env, get_db_path, send_webxdc},
    GENESIS_QR, INVITE_QR,
};

#[derive(Serialize, Deserialize)]
pub struct BotConfig {
    pub genesis_qr: String,
    pub invite_qr: String,
    pub tester_group: ChatId,
    pub reviewee_group: ChatId,
    pub genesis_group: ChatId,
    pub serial: usize,
}

/// Github Bot state
pub struct State {
    pub db: DB,
    pub config: BotConfig,
}

/// Github Bot
pub struct Bot {
    dc_ctx: Context,
    state: Arc<State>,
}

impl Bot {
    pub async fn new() -> Result<Self> {
        let dbdir = env::current_dir()?.join("deltachat.db");

        std::fs::create_dir_all(dbdir.clone()).context("Failed to create db folder")?;

        let dbfile = dbdir.join("db.sqlite");
        let context = Context::new(dbfile.as_path(), 1, Events::new(), StockStrings::new())
            .await
            .context("Failed to create context")?;

        if !context.get_config_bool(Config::Configured).await? {
            info!("Start configuring...");
            configure_from_env(&context).await?;
            info!("Configuration done");
        }

        let db = DB::new(&get_db_path()?).await?;

        let config = match db.get_config().await? {
            Some(config) => config,
            None => {
                info!("No bot configuration found, start configuring...");
                let config = Self::setup(&context).await?;
                db.set_config(&config).await?;

                // set chat types
                db.set_chat_type(config.genesis_group, ChatType::Genesis)
                    .await?;
                db.set_chat_type(config.reviewee_group, ChatType::ReviewPool)
                    .await?;
                db.set_chat_type(config.tester_group, ChatType::TesterPool)
                    .await?;

                // save qr codes to disk
                qrcode_generator::to_png_to_file(
                    &config.genesis_qr,
                    QrCodeEcc::Low,
                    1024,
                    GENESIS_QR,
                )?;
                println!("Generated genisis group join QR-code at {GENESIS_QR}");

                qrcode_generator::to_png_to_file(
                    &config.invite_qr,
                    QrCodeEcc::Low,
                    1024,
                    INVITE_QR,
                )?;
                println!("Generated 1:1 invite QR-code at {INVITE_QR}");

                config
            }
        };

        Ok(Self {
            dc_ctx: context,
            state: Arc::new(State { db, config }),
        })
    }

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
            EventType::ChatModified(chat_id) => match state.db.get_chat_type(chat_id).await? {
                Some(chat_type) => {
                    let contacts = chat::get_chat_contacts(context, chat_id).await?;
                    let filtered = contacts.into_iter().filter(|ci| !ci.is_special());
                    match chat_type {
                        ChatType::Genesis => {
                            info!("updating genesis contacts");
                            state
                                .db
                                .set_genesis_contacts(&filtered.collect::<Vec<_>>())
                                .await?;
                        }
                        ChatType::ReviewPool => {
                            info!("updating reviewer contacts");
                            state
                                .db
                                .set_tester_contacts(&filtered.collect::<Vec<_>>())
                                .await?;
                        }
                        ChatType::TesterPool => {
                            info!("updating tester contacts");
                            state
                                .db
                                .set_publisher_contacts(&filtered.collect::<Vec<_>>())
                                .await?;
                        }
                        // TODO: handle membership changes in review and submit group
                        _ => (),
                    };
                }
                None => {
                    info!(
                        "Chat {chat_id} is not in the database, adding it as chat with type shop"
                    );
                    state.db.set_chat_type(chat_id, ChatType::Shop).await?;
                    send_webxdc(context, chat_id, "./appstore.xdc", Some(appstore_message()))
                        .await?;
                }
            },
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
        match state.db.get_chat_type(chat_id).await {
            Ok(Some(chat_type)) => {
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
                        } else {
                            submit::handle_message(context, chat_id, state, msg).await?;
                        }
                    }
                    ChatType::Review => {
                        review::handle_message(context, chat_id, state, msg_id).await?;
                    }
                    ChatType::Genesis => {
                        genisis::handle_message(context, state, chat_id, msg_id).await?
                    }
                    ChatType::ReviewPool | ChatType::TesterPool => (),
                }
            }
            Ok(None) => {
                info!("creating new 1:1 chat with type Shop");
                state.db.set_chat_type(chat_id, ChatType::Shop).await?;
                shop::handle_message(context, state, chat_id).await?;
            }
            Err(e) => {
                warn!("Problem while retrieving [ChatType]: {}", e);
            }
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
        let chat_type = state
            .db
            .get_chat_type(chat_id)
            .await?
            .ok_or(anyhow::anyhow!("No chat for this webxdc update"))?;

        match chat_type {
            ChatType::Submit => {
                submit::handle_status_update(context, state, chat_id, update).await?
            }
            ChatType::Shop => {
                shop::handle_status_update(context, state, chat_id, msg_id, update).await?
            }
            ChatType::ReviewPool | ChatType::TesterPool | ChatType::Genesis => (),
            ChatType::Review => todo!(),
        }

        Ok(())
    }
}
