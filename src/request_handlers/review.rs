use std::sync::Arc;
use thiserror::Error;

use crate::{
    bot::State,
    db::DB,
    messages::creat_review_group_init_message,
    utils::{get_contact_name, send_app_info, send_webxdc},
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
use surrealdb::opt::RecordId;

use super::{submit::SubmitChat, AppInfo};

#[derive(Serialize, Deserialize)]
pub struct ReviewChat {
    pub review_helper: MsgId,
    pub submit_helper: MsgId,
    pub review_chat: ChatId,
    pub creator_chat: ChatId,
    pub publisher: ContactId,
    pub testers: Vec<ContactId>,
    pub app_info: RecordId,
}

#[derive(Debug, Error)]
pub enum HandlePublishError {
    #[error("Not enough testers in pool")]
    NotEnoughTesters,
    #[error("Not enough reviewee in pool")]
    NotEnoughPublishers,
    #[error(transparent)]
    SurrealDb(#[from] surrealdb::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
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
        let app_info = submit_chat.get_app_info(&state.db).await?;

        let publisher = state
            .db
            .get_publisher()
            .await?
            .ok_or(HandlePublishError::NotEnoughPublishers)?;

        let testers = state.db.get_testers().await?;
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

        let submit_helper = send_webxdc(context, chat_id, REVIEW_HELPER_XDC, None).await?;
        send_app_info(context, &app_info, submit_helper).await?;

        let review_chat = ReviewChat {
            review_chat: chat_id,
            creator_chat: submit_chat.creator_chat,
            publisher,
            testers: testers.clone(),
            app_info: submit_chat.app_info,
            review_helper: submit_chat.creator_webxdc,
            submit_helper,
        };

        state.db.upgrade_to_review_chat(&review_chat).await?;

        state
            .db
            .set_chat_type(chat_id, super::ChatType::Review)
            .await?;

        Ok(review_chat)
    }

    pub async fn get_app_info(&self, db: &DB) -> anyhow::Result<AppInfo> {
        db.get_app_info(&self.app_info).await
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
            let review_chat = state
                .db
                .get_review_chat(chat_id)
                .await?
                .ok_or(anyhow::anyhow!("No review chat found for chat {chat_id}"))?;

            let app_info = review_chat.get_app_info(&state.db).await?;
            if app_info.is_complete() {
                state.db.publish_app(&review_chat.app_info).await?;
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