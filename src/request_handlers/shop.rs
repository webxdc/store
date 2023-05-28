use super::{AppInfo, FrontendRequest};
use crate::{
    bot::State,
    messages::appstore_message,
    request_handlers::{self, submit::SubmitChat, ChatType, FrontendRequestWithData},
    utils::send_webxdc,
};
use anyhow::Context as _;
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    constants,
    contact::Contact,
    context::Context,
    message::{Message, MsgId, Viewtype},
};
use log::{info, warn};
use serde::Deserialize;
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

pub async fn handle_message(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
) -> anyhow::Result<()> {
    // Handle normal messages to the bot
    let chat = chat::Chat::load_from_db(context, chat_id).await?;
    if let constants::Chattype::Single = chat.typ {
        let msg = send_webxdc(context, chat_id, "./appstore.xdc", Some(appstore_message())).await?;
        let curr_serial = state.db.get_last_serial().await?;
        let apps = state.db.get_active_app_infos().await?;
        context
            .send_webxdc_status_update_struct(
                msg,
                deltachat::webxdc::StatusUpdateItem {
                    payload: json! {{"app_infos": apps, "serial": curr_serial}},
                    ..Default::default()
                },
                "",
            )
            .await?;
    }
    Ok(())
}

pub async fn handle_webxdc(
    context: &Context,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling webxdc message in chat with type shop");

    let mut app_info = AppInfo::from_xdc(&msg.get_file(context).unwrap()).await?;
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
    let creator = msg.get_from_id();
    chat::add_contact_to_chat(context, chat_id, creator).await?;

    state.db.set_chat_type(chat_id, ChatType::Submit).await?;

    chat::forward_msgs(context, &[msg.get_id()], chat_id).await?;
    let creator_webxdc = send_webxdc(context, chat_id, "review_helper.xdc", None).await?;

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

                let apps = state
                    .db
                    .get_active_app_infos_since(req.payload.data)
                    .await?;

                let curr_serial = state.db.get_last_serial().await?.context("no serial")?;
                context
                    .send_webxdc_status_update_struct(
                        msg_id,
                        deltachat::webxdc::StatusUpdateItem {
                            payload: json! {{"app_infos": apps, "serial": curr_serial}},
                            ..Default::default()
                        },
                        "",
                    )
                    .await?;
            }
            RequestType::Dowload => {
                info!("Handling store download");
                let resource =
                    serde_json::from_str::<FrontendRequestWithData<RequestType, Thing>>(&update)?
                        .payload
                        .data;

                let app = state.db.get_app_info(&resource).await?;
                let mut msg = Message::new(Viewtype::Webxdc);
                if let Some(file) = app.xdc_blob_dir {
                    msg.set_file(file.to_str().unwrap(), None);
                    chat::send_msg(context, chat_id, &mut msg).await.unwrap();
                } else {
                    warn!("No path for downloaded app {}", app.name)
                }
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
