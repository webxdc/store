use super::{AppInfo, FrontendRequest};
use crate::{
    bot::State,
    db,
    messages::appstore_message,
    request_handlers::{self, submit::SubmitChat, ChatType, FrontendRequestWithData},
    utils::{send_newest_updates, send_webxdc},
    SHOP_XDC, SUBMIT_HELPER_XDC,
};
use anyhow::{bail, Context as _};
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    constants,
    contact::Contact,
    context::Context,
    message::{Message, MsgId, Viewtype},
    webxdc::StatusUpdateItem,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use ts_rs::TS;

#[derive(TS, Deserialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
enum RequestType {
    Update,
    Dowload,
}

#[derive(TS, Deserialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct PublishRequest {
    pub name: String,
    pub description: String,
}

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct UpdateResponse {
    pub app_infos: Vec<AppInfo>,
    pub serial: i64,
}

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct DownloadResponse {
    okay: bool,
    id: Option<String>,
}

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
) -> anyhow::Result<()> {
    let chat = chat::Chat::load_from_db(context, chat_id).await?;
    if let constants::Chattype::Single = chat.typ {
        let msg = send_webxdc(context, chat_id, SHOP_XDC, Some(appstore_message())).await?;
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
    app_info.id = 0;

    db::create_app_info(conn, &app_info).await?;

    let chat_name = format!("Submit: {}", app_info.name);
    let chat_id = chat::create_group_chat(context, ProtectionStatus::Protected, &chat_name).await?;
    db::set_chat_type(conn, chat_id, ChatType::Submit).await?;

    let creator = msg.get_from_id();
    chat::add_contact_to_chat(context, chat_id, creator).await?;

    chat::forward_msgs(context, &[msg.get_id()], chat_id).await?;
    let creator_webxdc = send_webxdc(context, chat_id, SUBMIT_HELPER_XDC, None).await?;

    db::create_submit_chat(
        conn,
        &SubmitChat {
            submit_chat: chat_id,
            submit_helper: creator_webxdc,
            app_info: 69,
        },
    )
    .await?;

    request_handlers::submit::handle_webxdc(context, chat_id, state, msg).await?;
    Ok(())
}

pub async fn handle_status_update(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    msg_id: MsgId,
    update: String,
) -> anyhow::Result<()> {
    if let Ok(req) = serde_json::from_str::<FrontendRequest<RequestType>>(&update) {
        match req.payload.request_type {
            RequestType::Update => {
                info!("Handling store update request");
                let req =
                    serde_json::from_str::<FrontendRequestWithData<RequestType, i64>>(&update)?;

                send_newest_updates(
                    context,
                    msg_id,
                    &mut *state.db.acquire().await?,
                    req.payload.data,
                )
                .await?;
            }
            RequestType::Dowload => {
                info!("Handling store download");
                let result = handle_download_request(context, state, &update, chat_id).await;
                let resp = DownloadResponse {
                    okay: result.is_ok(),
                    id: result.ok(),
                };

                context
                    .send_webxdc_status_update_struct(
                        msg_id,
                        StatusUpdateItem {
                            payload: json! { resp },
                            ..Default::default()
                        },
                        "",
                    )
                    .await?;
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

async fn handle_download_request(
    context: &Context,
    state: Arc<State>,
    update: &str,
    chat_id: ChatId,
) -> anyhow::Result<String> {
    let resource = serde_json::from_str::<FrontendRequestWithData<RequestType, String>>(update)?
        .payload
        .data;

    let app = db::get_app_info(&mut *state.db.acquire().await?, resource.parse()?).await?;
    let mut msg = Message::new(Viewtype::Webxdc);
    if let Some(file) = app.xdc_blob_dir {
        msg.set_file(file.to_str().context("Can't covert file to str")?, None);
        chat::send_msg(context, chat_id, &mut msg).await?;
    } else {
        bail!("Appinfo {} has no xdc_blob_dir", app.name)
    }
    Ok(resource)
}
