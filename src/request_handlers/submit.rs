use std::sync::Arc;

use crate::{
    bot::State,
    db::{self, RecordId},
    request_handlers::{
        review::{HandlePublishError, ReviewChat},
        WebxdcStatusUpdate,
    },
    utils::send_update_payload_only,
};
use deltachat::{
    chat::{self, ChatId},
    context::Context,
    message::{Message, MsgId},
};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use ts_rs::TS;

use super::AppInfo;

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct SubmitChat {
    pub submit_chat: ChatId,
    pub submit_helper: MsgId,
    pub app_info: RecordId,
}

impl SubmitChat {
    pub async fn get_app_info(&self, conn: &mut SqliteConnection) -> sqlx::Result<AppInfo> {
        db::get_app_info(conn, self.app_info).await
    }
}

#[derive(Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub enum SubmitRequest {
    Submit { app_info: AppInfo },
}

#[derive(Serialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct SubmitResponse {
    pub okay: bool,
}

pub async fn create_review_chat(
    context: &Context,
    state: Arc<State>,
    submit_chat: SubmitChat,
    _chat_id: ChatId,
) -> anyhow::Result<()> {
    let submit_helper = submit_chat.submit_helper;
    if let Err(e) = ReviewChat::from_submit_chat(context, state.clone(), submit_chat).await {
        let msg = match e {
            HandlePublishError::NotEnoughTesters | HandlePublishError::NotEnoughPublishers => {
                e.to_string()
            }
            e => return Err(anyhow::anyhow!(e)),
        };
        chat::send_text_msg(context, state.config.genesis_group, msg).await?;
        send_update_payload_only(context, submit_helper, SubmitResponse { okay: false }).await?;
    } else {
        send_update_payload_only(context, submit_helper, SubmitResponse { okay: true }).await?;
    };
    Ok(())
}

pub async fn handle_webxdc(
    context: &Context,
    chat_id: ChatId,
    state: Arc<State>,
    msg: Message,
) -> anyhow::Result<()> {
    info!("Handling webxdc submission");

    let submit_chat: SubmitChat =
        db::get_submit_chat(&mut *state.db.acquire().await?, chat_id).await?;

    let mut app_info = submit_chat
        .get_app_info(&mut *state.db.acquire().await?)
        .await?;
    let file = msg.get_file(context).ok_or(anyhow::anyhow!(
        "Webxdc message {} has no file attached",
        msg.get_id()
    ))?;

    // TODO: Verify update
    let (changed, upgraded) = app_info.update_from_xdc(file).await?;
    if upgraded {
        info!("Upgraded app info: {:?}", app_info.id)
        // TODO: Handle upgrade
    } else if changed && check_app_info(context, &app_info, chat_id).await? {
        info!("Updating app info: {:?}", app_info.id);
        db::update_app_info(&mut *state.db.acquire().await?, &app_info).await?;
    }
    Ok(())
}

pub async fn handle_status_update(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    update: String,
) -> anyhow::Result<()> {
    info!("Handling app info update");
    if let Ok(req) = serde_json::from_str::<WebxdcStatusUpdate<SubmitRequest>>(&update) {
        let conn = &mut *state.db.acquire().await?;
        let submit_chat: SubmitChat =
            db::get_submit_chat(&mut *state.db.acquire().await?, chat_id).await?;
        match req.payload {
            SubmitRequest::Submit { app_info } => {
                // TODO: merge with existing app info
                let current_app_info = submit_chat.get_app_info(conn).await?;
                let new_app_info = current_app_info.update_from_request(app_info);
                if check_app_info(context, &new_app_info, chat_id).await? {
                    db::update_app_info(conn, &new_app_info).await?;
                }
                create_review_chat(context, state, submit_chat, chat_id).await?;
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

/// Checkes if all fields for appinfo are presents and sends a message about the outcome to chat_id.
pub async fn check_app_info(
    context: &Context,
    app_info: &AppInfo,
    chat_id: ChatId,
) -> anyhow::Result<bool> {
    let missing = app_info.generate_missing_list();
    if !missing.is_empty() {
        chat::send_text_msg(
            context,
            chat_id,
            format!("Missing fields: {}", missing.join(", ")),
        )
        .await?;
    }
    Ok(true)
}
