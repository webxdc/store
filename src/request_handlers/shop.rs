use super::{AppInfo, WebxdcStatusUpdate};
use crate::{
    bot::State,
    db,
    messages::store_message,
    request_handlers::{self, submit::SubmitChat, ChatType},
    utils::{send_app_info, send_newest_updates, send_update_payload_only, send_webxdc},
    SHOP_XDC, SUBMIT_HELPER_XDC,
};
use anyhow::{bail, Context as _};
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
        serial: i32,
    },
    Download {
        /// ID of the requested application.
        app_id: i32,
    },
}

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[serde(tag = "type")]
pub enum ShopResponse {
    DownloadOkay {
        id: i32,
        /// Name to be used as filename in `sendToChat`.
        name: String,
        /// Base64 encoded webxdc.
        data: String,
    },
    DownloadError {
        id: i32,
        error: String,
    },
    Update {
        app_infos: Vec<AppInfo>,
        serial: i32,
    },
}

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
) -> anyhow::Result<()> {
    let chat = chat::Chat::load_from_db(context, chat_id).await?;
    if let constants::Chattype::Single = chat.typ {
        let msg = send_webxdc(context, chat_id, SHOP_XDC, Some(store_message())).await?;
        send_newest_updates(context, msg, &mut *state.db.acquire().await?, 0).await?;
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

    app_info.author_email = contact.get_addr().to_string();
    app_info.author_name = contact.get_authname().to_string();

    db::create_app_info(conn, &mut app_info).await?;

    let chat_name = format!("Submit: {}", app_info.name);
    let chat_id = chat::create_group_chat(context, ProtectionStatus::Protected, &chat_name).await?;
    db::set_chat_type(conn, chat_id, ChatType::Submit).await?;

    let creator = msg.get_from_id();
    chat::add_contact_to_chat(context, chat_id, creator).await?;

    chat::forward_msgs(context, &[msg.get_id()], chat_id).await?;
    let creator_webxdc = send_webxdc(context, chat_id, SUBMIT_HELPER_XDC, None).await?;
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
            ShopRequest::Update { serial } => {
                info!("Handling store update request");
                send_newest_updates(context, msg_id, &mut *state.db.acquire().await?, serial)
                    .await?;
            }
            ShopRequest::Download { app_id } => {
                info!("Handling store download");

                let resp = match handle_download_request(state, app_id).await {
                    Ok((data, name)) => ShopResponse::DownloadOkay {
                        data,
                        name,
                        id: app_id,
                    },
                    Err(e) => {
                        warn!("Error while handling download request: {}", e);
                        ShopResponse::DownloadError {
                            error: e.to_string(),
                            id: app_id,
                        }
                    }
                };

                send_update_payload_only(context, msg_id, resp).await?;
            }
        }
    } else {
        info!(
            "Ignoring update: {}",
            &update.get(..100.min(update.len())).unwrap_or_default()
        )
    }
    Ok(())
}

/// Handles a request to download a store app.
/// Returns the base64 encoded webxdc and the name of the app.
async fn handle_download_request(
    state: Arc<State>,
    app_id: i32,
) -> anyhow::Result<(String, String)> {
    let app = db::get_app_info(&mut *state.db.acquire().await?, app_id).await?;
    if let Some(file) = app.xdc_blob_dir {
        Ok((
            encode(
                &tokio::fs::read(
                    file.to_str()
                        .context("Can't covert file '{file:?}' to str")?,
                )
                .await?,
            ),
            app.name,
        ))
    } else {
        bail!("Appinfo {} has no xdc_blob_dir", app.name)
    }
}
