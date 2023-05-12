//! Handlers for the different messages the bot receives
use deltachat::{chat::ChatId, contact::ContactId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize)]
pub struct Chat {
    pub chat_type: ChatType,
    pub chat_id: ChatId,
    pub publisher: Option<ContactId>,
    pub tester: Vec<ContactId>,
    pub creator: Option<ContactId>,
}

#[derive(Serialize, Deserialize)]
pub enum ChatType {
    Release,
    Shop,
}

#[derive(Deserialize)]
pub struct WebxdcStatusUpdate<T> {
    payload: T,
}

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub author_name: String,
    pub author_email: String,
    pub source_code_url: String,
    pub image: String,
    pub description: String,
    pub xdc_blob_url: String,
    pub version: String,
}

pub mod review {}

pub mod shop {
    use std::sync::Arc;

    use deltachat::{
        chat::{self, ChatId, ProtectionStatus},
        context::Context,
        message::{Message, MsgId, Viewtype},
    };
    use itertools::Itertools;
    use log::info;
    use rand::seq::SliceRandom;
    use serde::Deserialize;
    use serde_json::json;
    use ts_rs::TS;

    #[derive(TS, Deserialize)]
    #[ts(export)]
    enum RequestType {
        Update,
        Dowload,
        Publish,
    }

    #[derive(TS, Deserialize)]
    #[ts(export)]
    struct AppstoreRequest {
        request_type: RequestType,
    }

    use crate::{bot::State, request_handlers::WebxdcStatusUpdate};

    use super::Chat;

    pub async fn handle_message(context: &Context, chat_id: ChatId) -> anyhow::Result<()> {
        // Handle normal messages to the bot (resend the store itself).
        chat::send_text_msg(
            context,
            chat_id,
            r#"Welcome to the appstore bot! 
will shortly send you the appstore itself wher you can explore new apps."#
                .to_string(),
        )
        .await?;

        let mut webxdc_msg = Message::new(Viewtype::Webxdc);
        webxdc_msg.set_file("appstore-bot.xdc", None);
        chat::send_msg(context, chat_id, &mut webxdc_msg).await?;

        Ok(())
    }

    pub async fn handle_status_update(
        context: &Context,
        state: Arc<State>,
        chat_id: ChatId,
        msg_id: MsgId,
        update: String,
    ) -> anyhow::Result<()> {
        if let Ok(req) = serde_json::from_str::<WebxdcStatusUpdate<AppstoreRequest>>(&update) {
            match req.payload.request_type {
                RequestType::Update => {
                    info!("Handling store update");
                    context
                        .send_webxdc_status_update_struct(
                            msg_id,
                            deltachat::webxdc::StatusUpdateItem {
                                payload: json! {state.get_apps()},
                                ..Default::default()
                            },
                            "",
                        )
                        .await?;
                }
                RequestType::Dowload => todo!(),
                RequestType::Publish => {
                    let publishers = state.db.get_pubslishers().await?;
                    let testers = state.db.get_testers().await?;
                    let chosen_publisher = publishers.choose(&mut rand::thread_rng()).unwrap();
                    let mut chosen_testers = testers
                        .choose_multiple(&mut rand::thread_rng(), 3)
                        .collect_vec();

                    loop {
                        let item = chosen_testers
                            .iter()
                            .position(|elem| chosen_publisher == *elem);
                        if let Some(position) = item {
                            chosen_testers.remove(position);
                            chosen_testers.push(testers.choose(&mut rand::thread_rng()).unwrap())
                        } else {
                            break;
                        }
                    }

                    let msg = Message::load_from_db(context, msg_id).await?;

                    state
                        .db
                        .create_chat(Chat {
                            chat_type: super::ChatType::Release,
                            chat_id,
                            publisher: Some(chosen_publisher.clone()),
                            tester: chosen_testers.iter().map(|a| *a.clone()).collect_vec(),
                            creator: Some(msg.get_from_id()),
                        })
                        .await?;

                    let chat = chat::create_group_chat(
                        context,
                        ProtectionStatus::Unprotected,
                        "Publish: <Some App>",
                    )
                    .await?;
                    for tester in chosen_testers {
                        chat::add_contact_to_chat(context, chat_id, *tester).await?;
                    }

                    chat::add_contact_to_chat(context, chat_id, *chosen_publisher).await?;
                    chat::send_text_msg(context, chat, "hi".to_string()).await?;
                }
            }
        } else {
            info!("Ignoring update: {update}")
        }
        Ok(())
    }
}
