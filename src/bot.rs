//! Entry for the bot code

use anyhow::{Context as _, Result};
use clap::{CommandFactory, FromArgMatches};
use deltachat::{
    chat::{self, send_text_msg, Chat, ChatId},
    chatlist::Chatlist,
    config::Config,
    constants::Chattype,
    context::Context,
    message::{Message, MsgId, Viewtype},
    stock_str::StockStrings,
    EventType, Events,
};
use log::{debug, error, info, warn};
use std::{collections::HashMap, env, sync::Arc};
use tokio::{
    fs,
    sync::mpsc::{self, Receiver},
};

use crate::{
    db::DB,
    cli::{Cli, Commands},
    server::Server,
    shared::AppInfo,
    utils::configure_from_env,
};

/// Internal representation of a git repository that can be subscribed to
#[derive(Debug, Default)]
pub struct GitRepository {
    pub name: String,
    pub id: RepositoryId,
}

type RepositoryId = i64;

/// Github Bot state
pub struct State {
    pub db: DB,
    pub ip: String,
}

/// Github Bot
pub struct Bot {
    dc_ctx: Context,
    hook_receiver: Option<Receiver<Vec<AppInfo>>>,
    hook_server: Server,
    state: Arc<State>,
}

impl Bot {
    pub async fn new() -> Self {
        let dbdir = env::current_dir().unwrap().join("deltachat.db");
        std::fs::create_dir_all(dbdir.clone())
            .context("failed to create db folder")
            .unwrap();
        let dbfile = dbdir.join("db.sqlite");
        let ctx = Context::new(dbfile.as_path(), 1, Events::new(), StockStrings::new())
            .await
            .context("Failed to create context")
            .unwrap();
        let is_configured = ctx.get_config_bool(Config::Configured).await.unwrap();
        if !is_configured {
            info!("configuring");
            configure_from_env(&ctx).await.unwrap();
            info!("configuration done");
        }

        let (tx, rx) = mpsc::channel(100);

        let db = DB::new("file://bot.db").await;

        Self {
            dc_ctx: ctx,
            hook_receiver: Some(rx),
            state: Arc::new(State {
                db,
                ip: pnet::datalink::interfaces()
                    .iter()
                    .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
                    .expect("should have an ip")
                    .ips
                    .get(0)
                    .unwrap()
                    .ip()
                    .to_string(),
            }),
            hook_server: Server::new(tx),
        }
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
                Self::dc_event_handler(&ctx, state.clone(), event.typ).await;
            }
        });
        info!("initiated dc message handler (1/4)");

        self.dc_ctx.start_io().await;

        info!("initiated dc io (2/4)");

        // start webhook-server
        self.hook_server.start();

        info!("initiated webhook server (3/4)");

        // start webhook-handler
        let mut manifest_update_receiver = self.hook_receiver.take().unwrap();
        let state = self.state.clone();
        let ctx = self.dc_ctx.clone();
        tokio::spawn(async move {
            while let Some(event) = manifest_update_receiver.recv().await {
                if let Err(e) = Self::handle_manifest_change(state.clone(), &ctx, event).await {
                    error!("{e}")
                }
            }
        });
        info!("initiated webhook handler (4/4)");
        info!("successfully started bot! 🥳");
    }

    /// Handle _all_ dc-events
    async fn dc_event_handler(ctx: &Context, state: Arc<State>, event: EventType) {
        match event {
            EventType::ConfigureProgress { progress, comment } => {
                info!("DC: Configuring progress: {progress} {comment:?}")
            }
            EventType::Info(..) => (), //info!("DC: {msg}"),
            EventType::Warning(msg) => warn!("DC: {msg}"),
            EventType::Error(msg) => error!("DC: {msg}"),
            EventType::ConnectivityChanged => {
                warn!(
                    "DC: ConnectivityChanged: {:?}",
                    ctx.get_connectivity().await
                )
            }
            EventType::IncomingMsg { chat_id, msg_id } => {
                if let Err(err) = Self::handle_dc_message(ctx, state, chat_id, msg_id).await {
                    error!("DC: error handling message: {err}");
                }
            }
            other => {
                debug!("DC: [unhandled event] {other:?}");
            }
        }
    }

    /// Handles chat messages from clients
    async fn handle_dc_message(
        ctx: &Context,
        _state: Arc<State>,
        chat_id: ChatId,
        msg_id: MsgId,
    ) -> Result<()> {
        let msg = Message::load_from_db(ctx, msg_id).await?;

        if let Some(err) = msg.error() {
            error!("msg has the following error: {err}");
            if err.as_str() == "Decrypting failed: missing key" {
                send_text_msg(ctx, chat_id, "Unable to decrypt your message, but this message might have fixed it, so try again.".to_string()).await?;
            }
        }

        if let Some(text) = msg.get_text() {
            // only react to messages with right keywoard
            if text.starts_with("appstore") {
                match <Cli as CommandFactory>::command().try_get_matches_from(text.split(' ')) {
                    Ok(mut matches) => {
                        let res = <Cli as FromArgMatches>::from_arg_matches_mut(&mut matches)?;

                        match &res.command {
                            Commands::Download { file } => todo!(),
                            Commands::Open => todo!(),
                        }
                    }
                    Err(err) => {
                        send_text_msg(ctx, chat_id, err.to_string()).await.unwrap();
                    }
                };
            } else {
                if !chat_id.is_special() {
                    let chat = Chat::load_from_db(ctx, chat_id).await?;
                    if let Chattype::Single = chat.typ {
                        send_text_msg(
                            ctx,
                            chat_id,
                            "Commands must start with `appstore`".to_string(),
                        )
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a parsed webhook-event
    async fn handle_manifest_change(
        _state: Arc<State>,
        ctx: &Context,
        manifest: Vec<AppInfo>,
    ) -> anyhow::Result<()> {
        info!("Handling webhook event");
        let old_manifest_string = fs::read_to_string("./appstore_manifest.json")
            .await
            .unwrap_or_default();
        let old_manifest: Vec<AppInfo> =
            serde_json::from_str(&old_manifest_string).context("Failed to parse old manifest")?;
        let versions: HashMap<_, _> = old_manifest
            .into_iter()
            .map(|appinfo| (appinfo.source_code_url, appinfo.version))
            .collect();

        let updated_apps = manifest.into_iter().filter(|app| {
            versions
                .get(&app.source_code_url)
                .and_then(|version| Some(*version == app.version))
                .unwrap_or(true)
        });

        let update_manifest = serde_json::to_string(&updated_apps.collect::<Vec<_>>())?;
        // TODO: build all updated apps

        let chatlist = Chatlist::try_load(ctx, 0, None, None).await?;

        for (chat_id, _) in chatlist.iter() {
            let mut msg_ids = chat::get_chat_media(
                ctx,
                Some(*chat_id),
                Viewtype::Webxdc,
                Viewtype::Unknown,
                Viewtype::Unknown,
            )
            .await?;
            if let Some(msg_id) = msg_ids.pop() {
                ctx.send_webxdc_status_update(
                    msg_id,
                    &update_manifest,
                    &format!("updated some webxdc messages: {update_manifest}"),
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn stop(self) {
        self.hook_server.stop()
    }
}
