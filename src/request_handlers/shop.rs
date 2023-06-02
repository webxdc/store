use super::{AppInfo, FrontendRequest};
use crate::{
    bot::State,
    db::{FrontendAppInfo, DB},
    messages::appstore_message,
    request_handlers::{self, submit::SubmitChat, ChatType, FrontendRequestWithData},
    utils::send_webxdc,
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
use surrealdb::sql::{Id, Thing};
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
    app_infos: Vec<FrontendAppInfo>,
    serial: usize,
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
        send_newest_updates(context, msg, &state.db, 0).await?;
    }
    Ok(())
}

pub async fn handle_webxdc(
    context: &Context,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling webxdc message in shop chat");

    let mut app_info = AppInfo::from_xdc(&msg.get_file(context).context("Can't get file")?).await?;
    let contact = Contact::load_from_db(context, msg.get_from_id()).await?;

    app_info.author_email = contact.get_addr().to_string();
    app_info.author_name = contact.get_authname().to_string();

    let resource_id = Thing {
        tb: "appinfo".to_string(),
        id: Id::rand(),
    };

    state
        .db
        .create_app_info(&app_info, resource_id.clone())
        .await?;

    let chat_name = format!("Submit: {}", app_info.name);
    let chat_id = chat::create_group_chat(context, ProtectionStatus::Protected, &chat_name).await?;
    state.db.set_chat_type(chat_id, ChatType::Submit).await?;

    let creator = msg.get_from_id();
    chat::add_contact_to_chat(context, chat_id, creator).await?;

    chat::forward_msgs(context, &[msg.get_id()], chat_id).await?;
    let creator_webxdc = send_webxdc(context, chat_id, SUBMIT_HELPER_XDC, None).await?;

    state
        .db
        .create_submit(&SubmitChat {
            creator_chat: chat_id,
            creator_webxdc,
            app_info: resource_id,
        })
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
                    serde_json::from_str::<FrontendRequestWithData<RequestType, usize>>(&update)?;

                send_newest_updates(context, msg_id, &state.db, req.payload.data).await?;
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

    let app = state
        .db
        .get_app_info(&Thing {
            tb: "app_info".to_string(),
            id: Id::String(resource.clone()),
        })
        .await?;
    let mut msg = Message::new(Viewtype::Webxdc);
    if let Some(file) = app.xdc_blob_dir {
        msg.set_file(file.to_str().context("Can't covert file to str")?, None);
        chat::send_msg(context, chat_id, &mut msg).await?;
    } else {
        bail!("Appinfo {} has no xdc_blob_dir", app.name)
    }
    Ok(resource)
}

async fn send_newest_updates(
    context: &Context,
    msg_id: MsgId,
    db: &DB,
    serial: usize,
) -> anyhow::Result<()> {
    let app_infos: Vec<_> = db
        .get_active_app_infos_since(serial)
        .await?
        .into_iter()
        .map(FrontendAppInfo::from)
        .collect();

    let serial = 0; //state.db.get_last_serial().await?.context("no serial")?;
    let resp = UpdateResponse { app_infos, serial };
    context
        .send_webxdc_status_update_struct(
            msg_id,
            deltachat::webxdc::StatusUpdateItem {
                payload: json! {resp},
                ..Default::default()
            },
            "",
        )
        .await?;
    Ok(())
}
