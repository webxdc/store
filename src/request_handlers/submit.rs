use std::sync::Arc;

use crate::{
    bot::State,
    db::DB,
    request_handlers::{
        review::{HandlePublishError, ReviewChat},
        FrontendRequestWithData,
    },
    utils::send_app_info,
};
use deltachat::{
    chat::{self, ChatId},
    context::Context,
    message::{Message, MsgId},
};
use log::info;
use serde::{Deserialize, Serialize};
use surrealdb::opt::RecordId;

use super::AppInfo;

#[derive(Serialize, Deserialize)]
pub struct SubmitChat {
    pub creator_chat: ChatId,
    pub creator_webxdc: MsgId,
    pub app_info: RecordId,
}

impl SubmitChat {
    pub async fn get_app_info(&self, db: &DB) -> anyhow::Result<AppInfo> {
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
            if msg_text == "/publish" {
                // create review chat
                if let Err(e) =
                    ReviewChat::from_submit_chat(context, state.clone(), submit_chat).await
                {
                    let msg = match e {
                        HandlePublishError::NotEnoughTesters
                        | HandlePublishError::NotEnoughPublishers => e.to_string(),
                        e => return Err(anyhow::anyhow!(e)),
                    };
                    chat::send_text_msg(context, state.config.genesis_group, msg).await?;
                    chat::send_text_msg(
                        context,
                        chat_id,
                        "Problem creating your review chat.".to_string(),
                    )
                    .await?;
                } else {
                    chat::send_text_msg(
                        context,
                        chat_id,
                        "I've submitted your app for review!".to_string(),
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

    let mut app_info = submit_chat.get_app_info(&state.db).await?;
    let file = msg.get_file(context).ok_or(anyhow::anyhow!(
        "Webxdc message {} has no file attached",
        msg.get_id()
    ))?;

    // TODO: Verify update
    let (changed, upgraded) = app_info.update_from_xdc(file).await?;
    if upgraded {
        // TODO: Handle upgrade
    } else if changed {
        state
            .db
            .update_app_info(&app_info, &submit_chat.app_info)
            .await?;

        check_app_info(context, &app_info, &submit_chat, chat_id).await?;
    }
    Ok(())
}

pub async fn handle_status_update(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    update: String,
) -> anyhow::Result<()> {
    info!("Handling [AppInfo] update");
    if let Ok(req) = serde_json::from_str::<FrontendRequestWithData<String, AppInfo>>(&update) {
        let submit_chat = state
            .db
            .get_submit_chat(chat_id)
            .await?
            .ok_or(anyhow::anyhow!("No submit chat found for chat {chat_id}"))?;

        // TODO: verify update
        state
            .db
            .update_app_info(&req.payload.data, &submit_chat.app_info)
            .await?;

        check_app_info(context, &req.payload.data, &submit_chat, chat_id).await?;
    } else {
        info!(
            "Ignoring update: {}",
            &update.get(..100.min(update.len())).unwrap_or_default()
        )
    }
    Ok(())
}

/// Checkes if all fields for appinfo are presents and sends a message about the outcome to chat_id.
pub async fn check_app_info(
    context: &Context,
    app_info: &AppInfo,
    submit_chat: &SubmitChat,
    chat_id: ChatId,
) -> anyhow::Result<()> {
    send_app_info(context, app_info, submit_chat.creator_webxdc).await?;

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
    Ok(())
}