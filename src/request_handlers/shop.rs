use super::{AppInfo, WebxdcStatusUpdate};
use crate::{
    bot::State,
    db,
    messages::store_message,
    request_handlers::{self, submit::SubmitChat, ChatType},
    utils::{send_app_info, send_newest_updates, send_update_payload_only, send_webxdc, Webxdc},
};
use anyhow::Context as _;
use base64::encode;
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    constants,
    contact::Contact,
    context::Context,
    message::{Message, MsgId},
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
enum ShopRequest {
    Update {
        /// Requested update sequence number.
        serial: u32,
        /// Listof apps selected for caching.
        #[serde(default)]
        apps: Vec<(String, i32)>,
    },
    Download {
        /// ID of the requested application.
        app_id: String,
    },
}

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[serde(tag = "type")]
pub enum ShopResponse {
    DownloadOkay {
        /// app_id of the downloaded app.
        app_id: String,
        /// Name to be used as filename in `sendToChat`.
        name: String,
        /// Base64 encoded webxdc.
        data: String,
    },
    DownloadError {
        app_id: String,
        error: String,
    },
    Update {
        /// List of new / updated app infos.
        app_infos: Vec<AppInfo>,
        serial: i32,
        /// `app_id`s of apps that will receive an update.
        /// The frontend can use these to set the state to updating.
        updating: Vec<String>,
    },
}

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
) -> anyhow::Result<()> {
    let chat = chat::Chat::load_from_db(context, chat_id).await?;
    if let constants::Chattype::Single = chat.typ {
        let msg = send_webxdc(
            context,
            &state,
            chat_id,
            Webxdc::Shop,
            Some(store_message()),
        )
        .await?;
        send_newest_updates(context, msg, &mut *state.db.acquire().await?, 0, vec![]).await?;
    }
    Ok(())
}

pub async fn handle_webxdc(
    context: &Context,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling webxdc message in shop chat");
    let conn = &mut *state.db.acquire().await?;
    let mut app_info = AppInfo::from_xdc(&msg.get_file(context).context("Can't get file")?).await?;
    let contact = Contact::load_from_db(context, msg.get_from_id()).await?;

    app_info.submitter_uri = Some(contact.get_authname().to_string());

    db::create_app_info(conn, &mut app_info).await?;

    let chat_name = format!("Submit: {}", app_info.name);
    let chat_id = chat::create_group_chat(context, ProtectionStatus::Protected, &chat_name).await?;
    db::set_chat_type(conn, chat_id, ChatType::Submit).await?;

    let creator = msg.get_from_id();
    chat::add_contact_to_chat(context, chat_id, creator).await?;

    chat::forward_msgs(context, &[msg.get_id()], chat_id).await?;
    let creator_webxdc = send_webxdc(context, &state, chat_id, Webxdc::Submit, None).await?;
    send_app_info(context, &app_info, creator_webxdc).await?;

    db::create_submit_chat(
        conn,
        &SubmitChat {
            submit_chat: chat_id,
            submit_helper: creator_webxdc,
            app_info: app_info.id,
        },
    )
    .await?;

    request_handlers::submit::handle_webxdc(context, chat_id, state, msg).await?;
    Ok(())
}

pub async fn handle_status_update(
    context: &Context,
    state: Arc<State>,
    msg_id: MsgId,
    update: String,
) -> anyhow::Result<()> {
    if let Ok(req) = serde_json::from_str::<WebxdcStatusUpdate<ShopRequest>>(&update) {
        match req.payload {
            ShopRequest::Update { serial, apps } => {
                info!("Handling store update request");

                // Get all updating xdcs
                let mut updating = vec![];
                let conn = &mut *state.db.acquire().await?;
                for (app_id, version) in apps {
                    if db::maybe_get_greater_version(conn, &app_id, version).await? {
                        updating.push(app_id);
                    }
                }

                info!("Updating multiple client apps: {:?}", updating);

                send_newest_updates(
                    context,
                    msg_id,
                    &mut *state.db.acquire().await?,
                    serial,
                    updating.clone(),
                )
                .await?;

                // Send updates
                for app_id in &updating {
                    let context = context.clone();
                    let state = state.clone();
                    let app_id = app_id.clone();
                    let resp = handle_download(&state, app_id).await;
                    send_update_payload_only(&context, msg_id, resp).await?;
                }
            }
            ShopRequest::Download { app_id } => {
                info!("Handling store download");
                let resp = handle_download(&state, app_id).await;
                send_update_payload_only(context, msg_id, resp).await?;
            }
        }
    } else {
        info!(
            "Ignoring self-sent update: {}",
            &update.get(..100.min(update.len())).unwrap_or_default()
        )
    }
    Ok(())
}

pub async fn handle_download(state: &State, app_id: String) -> ShopResponse {
    match get_webxdc_data(state, &app_id).await {
        Ok((data, name)) => ShopResponse::DownloadOkay { data, name, app_id },
        Err(e) => {
            warn!("Error while handling download request: {}", e);
            ShopResponse::DownloadError {
                error: e.to_string(),
                app_id,
            }
        }
    }
}

/// Handles a request to download a store app.
/// Returns the base64 encoded webxdc and the name of the app.
async fn get_webxdc_data(state: &State, app_id: &str) -> anyhow::Result<(String, String)> {
    let app = db::get_app_info_for_app_id(&mut *state.db.acquire().await?, app_id).await?;
    Ok((
        encode(
            &tokio::fs::read(
                app.xdc_blob_path
                    .to_str()
                    .context("Can't covert file '{file:?}' to str")?,
            )
            .await?,
        ),
        app.name,
    ))
}
