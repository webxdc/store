use std::sync::Arc;

use crate::{
    bot::State,
    utils::{get_chat_xdc, send_webxdc},
};
use deltachat::{
    chat::{self, ChatId},
    context::Context,
    message::{Message, MsgId},
};
use log::info;
use serde::Deserialize;
use serde_json::json;

use super::FrontendRequest;

pub async fn handle_message(
    context: &Context,
    chat_id: ChatId,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling release message");

    if let Some(msg_text) = msg.get_text() {
        if msg_text == "/release" {
            let review_chat = state
                .db
                .get_review_chat(chat_id)
                .await?
                .ok_or(anyhow::anyhow!("No review chat found for chat {chat_id}"))?;

            let app_info = review_chat.get_app_info(&state.db).await?;
            if app_info.is_complete() {
                state.db.publish_app(&review_chat.app_info).await.unwrap();
                chat::send_text_msg(context, chat_id, "App published".into()).await?;
            } else {
                let missing = app_info.generate_missing_list();
                chat::send_text_msg(
                    context,
                    chat_id,
                    format!(
                        "You still need missing some required fields: {}",
                        missing.join(", ")
                    ),
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn handle_webxdc(
    context: &Context,
    chat_id: ChatId,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling webxdc submission");

    let review_chat = state
        .db
        .get_review_chat(chat_id)
        .await?
        .ok_or(anyhow::anyhow!("No review chat found for chat {chat_id}"))?;

    let mut app_info = review_chat.get_app_info(&state.db).await.unwrap();
    let file = msg.get_file(context).ok_or(anyhow::anyhow!(
        "Webxdc message {} has no file attached",
        msg.get_id()
    ))?;

    app_info.update_from_xdc(file).await?;
    state
        .db
        .update_app_info(&app_info, &review_chat.app_info)
        .await?;

    /* if get_chat_xdc(context, chat_id).await?.is_none() {
        send_webxdc(context, chat_id, "./review_helper.xdc").await?;
    } */
    send_webxdc(context, chat_id, "./review_helper.xdc").await?;

    let missing = app_info.generate_missing_list();

    if !missing.is_empty() {
        chat::send_text_msg(
            context,
            chat_id,
            format!("Missing fields: {}", missing.join(", ")),
        )
        .await?;
    } else {
        chat::send_text_msg(
            context,
            chat_id,
            "Hooray, I've got all information neeeded to publish your app! 🥳".into(),
        )
        .await?;
    }

    let msg_id = get_chat_xdc(context, chat_id)
        .await?
        .expect("Expecting an webxdc in review chat");

    context
        .send_webxdc_status_update_struct(
            msg_id,
            deltachat::webxdc::StatusUpdateItem {
                payload: json! {app_info},
                ..Default::default()
            },
            "",
        )
        .await?;

    Ok(())
}

#[derive(Deserialize)]
enum RequestType {
    UpdateInfo,
    UpdateReviewStatus,
}

pub async fn handle_status_update(
    _context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    _msg_id: MsgId,
    update: String,
) -> anyhow::Result<()> {
    if let Ok(req) = serde_json::from_str::<FrontendRequest<RequestType>>(&update) {
        match req.payload.request_type {
            RequestType::UpdateInfo => {
                let review_chat = state
                    .db
                    .get_review_chat(chat_id)
                    .await?
                    .ok_or(anyhow::anyhow!("No review chat found for chat {chat_id}"))?;

                let _app_info = review_chat.get_app_info(&state.db).await?;
            }
            RequestType::UpdateReviewStatus => todo!(),
        }
    } else {
        info!("Ignoring update: {}", &update[..100])
    }
    Ok(())
}
