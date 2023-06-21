use sqlx::SqliteConnection;
use std::sync::Arc;
use thiserror::Error;
use ts_rs::TS;

use crate::{
    bot::State,
    db::{self, RecordId},
    messages::creat_review_group_init_message,
    request_handlers::WebxdcStatusUpdate,
    utils::{get_contact_name, send_app_info, send_update_payload_only, send_webxdc},
    REVIEW_HELPER_XDC,
};
use deltachat::{
    chat::{self, ChatId, ProtectionStatus},
    contact::ContactId,
    context::Context,
    message::{Message, MsgId},
};
use log::info;
use serde::{Deserialize, Serialize};

use super::{submit::SubmitChat, AppInfo};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct ReviewChat {
    // Xdc helper references.
    pub review_helper: MsgId,
    pub submit_helper: MsgId,

    // Chat references.
    pub review_chat: ChatId,
    pub submit_chat: ChatId,

    // Special roles.
    pub publisher: ContactId,
    pub testers: Vec<ContactId>,

    // Reference to AppInfo in [DB].
    pub app_info: RecordId,
}

#[derive(Debug, Error)]
pub enum HandlePublishError {
    #[error("Not enough testers in pool")]
    NotEnoughTesters,
    #[error("Not enough publishers in pool")]
    NotEnoughPublishers,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub enum ReviewRequest {
    Publish { app_info: RecordId },
}

#[derive(Serialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
struct ReviewResponse {
    okay: bool,
}

impl ReviewChat {
    // TODO: refactor this to some more idiomatic version
    pub async fn from_submit_chat(
        context: &Context,
        state: Arc<State>,
        submit_chat: SubmitChat,
        //testers: &[ContactId],
        //publisher: ContactId,
    ) -> Result<Self, HandlePublishError> {
        let app_info = submit_chat
            .get_app_info(&mut *state.db.acquire().await?)
            .await?;
        let conn = &mut *state.db.acquire().await?;
        let publisher = db::get_random_publisher(conn)
            .await
            .map_err(|_e| HandlePublishError::NotEnoughPublishers)?;

        let testers = db::get_random_testers(conn, 3).await?;
        if testers.is_empty() {
            return Err(HandlePublishError::NotEnoughTesters);
        }

        // create review chat
        let chat_id = chat::create_group_chat(
            context,
            ProtectionStatus::Protected,
            &format!("Testing: {}", app_info.name),
        )
        .await?;

        // add testers and publishers
        for tester in testers.iter() {
            chat::add_contact_to_chat(context, chat_id, *tester).await?;
        }
        chat::add_contact_to_chat(context, chat_id, publisher).await?;

        // create initial message
        let mut tester_names = Vec::new();
        for tester in &testers {
            tester_names.push(get_contact_name(context, *tester).await);
        }

        chat::send_text_msg(
            context,
            chat_id,
            creat_review_group_init_message(
                &tester_names,
                &get_contact_name(context, publisher).await,
            ),
        )
        .await?;

        let review_helper = send_webxdc(context, chat_id, REVIEW_HELPER_XDC, None).await?;
        send_app_info(context, &app_info, review_helper).await?;

        let review_chat = ReviewChat {
            review_chat: chat_id,
            publisher,
            review_helper,
            testers: testers.clone(),
            submit_chat: submit_chat.submit_chat,
            app_info: submit_chat.app_info,
            submit_helper: submit_chat.submit_helper,
        };

        db::upgrade_to_review_chat(conn, &review_chat).await?;

        db::set_chat_type(conn, chat_id, super::ChatType::Review).await?;

        Ok(review_chat)
    }

    pub async fn get_app_info(&self, conn: &mut SqliteConnection) -> sqlx::Result<AppInfo> {
        db::get_app_info(conn, self.app_info).await
    }
}

pub async fn handle_message(
    context: &Context,
    chat_id: ChatId,
    state: Arc<State>,
    message_id: MsgId,
) -> anyhow::Result<()> {
    info!("Handling review message");
    let msg = Message::load_from_db(context, message_id).await?;
    if let Some(msg_text) = msg.get_text() {
        if msg_text == "/release" {
            let conn = &mut *state.db.acquire().await?;
            let review_chat = db::get_review_chat(conn, chat_id).await?;
            let app_info = review_chat.get_app_info(conn).await?;
            if app_info.is_complete() {
                db::publish_app_info(conn, review_chat.app_info).await?;
                chat::send_text_msg(context, chat_id, "App published".into()).await?;
            } else {
                let missing = app_info.generate_missing_list();
                chat::send_text_msg(
                    context,
                    chat_id,
                    format!(
                        "You still are still missing some required fields: {}",
                        missing.join(", ")
                    ),
                )
                .await?;
            }
        }
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
    if let Ok(req) = serde_json::from_str::<WebxdcStatusUpdate<ReviewRequest>>(&update) {
        match req.payload {
            ReviewRequest::Publish { app_info } => {
                let conn = &mut *state.db.acquire().await?;
                db::publish_app_info(conn, app_info).await?;
                let review_chat = db::get_review_chat(conn, chat_id).await?;

                send_update_payload_only(
                    context,
                    review_chat.review_helper,
                    ReviewResponse { okay: true },
                )
                .await?;
                chat::send_text_msg(
                    context,
                    review_chat.submit_chat,
                    "Your app has been accepted and published to the appstore!".into(),
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
