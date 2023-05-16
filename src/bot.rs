//! Entry for the bot code
use anyhow::{Context as _, Result};
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    config::Config,
    contact::{Contact, ContactId},
    context::Context,
    message::{Message, MsgId},
    stock_str::StockStrings,
    EventType, Events,
};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{env, io, sync::Arc};
use surrealdb::engine::local::Db;

use crate::{
    db::DB,
    request_handlers::{shop, AppInfo, ChatType, ReviewChat},
    utils::configure_from_env,
};

#[derive(Serialize, Deserialize)]
pub struct BotConfig {
    administrator: String,
}

/// Github Bot state
pub struct State {
    pub db: DB<Db>,
    pub config: BotConfig,
}

impl State {
    pub fn get_apps(&self) -> Vec<AppInfo> {
        vec![]
    }
}

/// Github Bot
pub struct Bot {
    dc_ctx: Context,
    state: Arc<State>,
}

impl Bot {
    pub async fn new() -> Self {
        let dbdir = env::current_dir().unwrap().join("deltachat.db");

        std::fs::create_dir_all(dbdir.clone())
            .context("failed to create db folder")
            .unwrap();

        let dbfile = dbdir.join("db.sqlite");
        let context = Context::new(dbfile.as_path(), 1, Events::new(), StockStrings::new())
            .await
            .context("Failed to create context")
            .unwrap();

        if !context.get_config_bool(Config::Configured).await.unwrap() {
            info!("start configuring...");
            configure_from_env(&context).await.unwrap();
            info!("configuration done");
        }

        let db = DB::new("bot.db").await;

        let config = match db.get_config().await {
            Ok(config) => config,
            Err(_) => {
                info!("No configuration found, start configuring...");
                let config = Self::configure(&context).await.unwrap();
                db.set_config(&config).await.unwrap();
                config
            }
        };

        Self {
            dc_ctx: context,
            state: Arc::new(State { db, config }),
        }
    }

    // creates the bot configuration.
    async fn configure(context: &Context) -> Result<BotConfig> {
        println!("For configuration, please enter the admistrators email address");

        let stdin = io::stdin();
        let mut email = String::new();
        email.clear();
        stdin.read_line(&mut email)?;

        let contact = Contact::create(context, "administrator", &email).await?;

        // create review chat
        let review_chat = chat::create_group_chat(
            context,
            ProtectionStatus::Unprotected,
            &format!("Appstore: Publishers"),
        )
        .await?;

        chat::add_contact_to_chat(context, review_chat, contact).await?;
        chat::send_text_msg(context, review_chat, "This is the reviewee group, you can add new members who will also take the role of reviewees".to_string()).await?;

        // create testers chat
        let tester_chat = chat::create_group_chat(
            context,
            ProtectionStatus::Unprotected,
            &format!("Appstore: Testers"),
        )
        .await?;

        chat::add_contact_to_chat(context, tester_chat, contact).await?;
        chat::send_text_msg(context, tester_chat, "This is the testers group, you can add new members who will also take the role of reviewees".to_string()).await?;

        Ok(BotConfig {
            administrator: email,
        })
    }

    /// Start the bot which includes:
    /// - starting dc-message-receive loop
    /// - starting webhook-receive loop
    ///   - starting receiving server
    pub async fn start(&mut self) {
        // start dc message handler
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

        self.state
            .db
            .create_chat(ReviewChat {
                chat_id: ChatId::new(1),
                publisher: ContactId::new(10),
                testers: Vec::new(),
                creator: ContactId::new(10),
                ios: false,
                android: false,
                desktop: false,
                app_info: AppInfo {
                    ..Default::default()
                },
            })
            .await
            .unwrap();

        info!("initiated dc message handler (1/2)");
        self.dc_ctx.start_io().await;
        info!("initiated dc io (2/2)");
        info!("successfully started bot! 🥳");
    }

    /// Handle _all_ dc-events
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
            other => {
                debug!("DC: [unhandled event] {other:?}");
            }
        }
        Ok(())
    }

    /// Handles chat messages from clients
    async fn handle_dc_message(
        context: &Context,
        state: Arc<State>,
        chat_id: ChatId,
        _msg_id: MsgId,
    ) -> Result<()> {
        match state.db.get_chat_type(chat_id).await {
            Ok(Some(chat_type)) => {
                info!("Handling message with type <{chat_type:?}>");
                match chat_type {
                    ChatType::Release => todo!(),
                    ChatType::Shop => shop::handle_message(context, chat_id).await?,
                    ChatType::ReviewPool | ChatType::TesterPool => (),
                }
            }
            Ok(None) => {
                info!("creating new 1:1 chat with type Shop");
                state.db.set_chat_type(chat_id, ChatType::Shop).await?;
                shop::handle_message(context, chat_id).await?;
            }
            Err(e) => {
                warn!("got some error: {}", e);
            }
        }
        Ok(())
    }

    /// Handles chat messages from clients
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
            .ok_or(anyhow::anyhow!("No chat for this message"))?;

        match chat_type {
            ChatType::Release => todo!(),
            ChatType::Shop => {
                shop::handle_status_update(context, state, chat_id, msg_id, update).await?
            }
            ChatType::ReviewPool => todo!(),
            ChatType::TesterPool => todo!(),
        }

        Ok(())
    }
}
