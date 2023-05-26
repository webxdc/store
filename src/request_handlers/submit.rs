use std::sync::Arc;

use crate::{
    bot::State,
    db::DB,
    request_handlers::review::{HandlePublishError, ReviewChat},
    utils::send_webxdc,
};
use deltachat::{
    chat::{self, ChatId},
    context::Context,
    message::{Message, MsgId},
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use surrealdb::opt::RecordId;

use super::{AppInfo, FrontendRequest};

#[derive(Serialize, Deserialize)]
pub struct SubmitChat {
    pub creator_chat: ChatId,
    pub creator_webxdc: MsgId,
    pub app_info: RecordId,
}

impl SubmitChat {
    pub async fn get_app_info(&self, db: &DB) -> surrealdb::Result<AppInfo> {
        db.get_app_info(&self.app_info).await
    }
}

pub async fn handle_message(
    context: &Context,
    chat_id: ChatId,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling message in submit-chat");
    let submit_chat: SubmitChat = state
        .db
        .get_submit_chat(chat_id)
        .await?
        .expect("Submit chat should exist");

    if let Some(msg_text) = msg.get_text() {
        if msg_text.starts_with('/') {
            if msg_text == "/accept-release" {
                // create review chat
                if let Err(e) =
                    ReviewChat::from_submit_chat(context, state.clone(), submit_chat).await
                {
                    let msg = match e {
                        HandlePublishError::NotEnoughTesters
                        | HandlePublishError::NotEnoughPublishers => e.to_string(),
                        e => return Err(anyhow::anyhow!(e)),
                    };
                    chat::send_text_msg(context, state.config.genesis_group, msg.into()).await?;
                    chat::send_text_msg(
                        context,
                        chat_id,
                        "Problem creating your review chat".to_string(),
                    )
                    .await?;
                }
            } else {
                chat::send_text_msg(context, chat_id, "Command not found".to_string()).await?;
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

    let submit_chat = state
        .db
        .get_submit_chat(chat_id)
        .await?
        .ok_or(anyhow::anyhow!("No submit chat found for chat {chat_id}"))?;

    let mut app_info = submit_chat.get_app_info(&state.db).await.unwrap();
    let file = msg.get_file(context).ok_or(anyhow::anyhow!(
        "Webxdc message {} has no file attached",
        msg.get_id()
    ))?;

    app_info.update_from_xdc(file).await?;

    state
        .db
        .update_app_info(&app_info, &submit_chat.app_info)
        .await?;

    /*
    TODO: resend if it is different
    if get_chat_xdc(context, chat_id).await?.is_none() {
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
            "I've got all information needed, if you want to publish it, type '/publish' and I will send it into review.".into(),
        )
        .await?;
    }

    context
        .send_webxdc_status_update_struct(
            submit_chat.creator_webxdc,
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
    _state: Arc<State>,
    _chat_id: ChatId,
    _msg_id: MsgId,
    _update: String,
) -> anyhow::Result<()> {
    // TODO: handle changes on frontend
    /* if let Ok(req) = serde_json::from_str::<FrontendRequest<String>>(&update) {
        let review_chat = state
            .db
            .get_review_chat(chat_id)
            .await?
            .ok_or(anyhow::anyhow!("No review chat found for chat {chat_id}"))?;

        let _app_info = review_chat.get_app_info(&state.db).await?;
    } else {
        info!("Ignoring update: {}", &update[..100])
    } */
    Ok(())
}
