//! Handlers for the different messages the bot receives
use deltachat::{chat::ChatId, contact::ContactId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize)]
pub struct ReviewChat {
    pub chat_type: ChatType,
    pub chat_id: ChatId,
    pub publisher: ContactId,
    pub tester: Vec<ContactId>,
    pub creator: ContactId,
    pub ios: bool,
    pub android: bool,
    pub desktop: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
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
    use std::{any::Any, sync::Arc};

    use deltachat::{
        chat::{self, ChatId, ProtectionStatus},
        contact::ContactId,
        context::Context,
        message::{Message, MsgId, Viewtype},
    };
    use itertools::Itertools;
    use log::info;
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
    struct PublishRequest {
        name: String,
    }

    #[derive(TS, Deserialize)]
    #[ts(export)]
    struct AppstoreRequest {
        request_type: RequestType,
        data: String,
    }

    impl AppstoreRequest {
        fn get_data_as<T: for<'a> Deserialize<'a>>(&self) -> serde_json::Result<T> {
            serde_json::from_str(&self.data)
        }
    }

    use crate::{bot::State, request_handlers::WebxdcStatusUpdate, utils::get_oon_peer};

    use super::ReviewChat;

    fn creat_review_group_message(testers: &[ContactId], publisher: &ContactId) -> String {
        todo!()
    }

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
                    // get publisher and testers
                    let publisher = state.db.get_publisher().await.unwrap();
                    let testers = state.db.get_testers().await.unwrap();

                    let creator = get_oon_peer(context, chat_id).await?;

                    state
                        .db
                        .create_chat(ReviewChat {
                            chat_type: super::ChatType::Release,
                            chat_id,
                            publisher,
                            tester: testers.iter().copied().collect_vec(),
                            creator,
                            ios: false,
                            android: false,
                            desktop: false,
                        })
                        .await?;

                    // create the new chat
                    let group_date = req.payload.get_data_as::<PublishRequest>()?;
                    let chat_id = chat::create_group_chat(
                        context,
                        ProtectionStatus::Unprotected,
                        &format!("Publish: {}", group_date.name),
                    )
                    .await?;

                    // add all chat members
                    for tester in testers.iter() {
                        chat::add_contact_to_chat(context, chat_id, *tester).await?;
                    }
                    chat::add_contact_to_chat(context, chat_id, publisher).await?;
                    chat::add_contact_to_chat(context, chat_id, creator).await?;

                    chat::send_text_msg(
                        context,
                        chat_id,
                        creat_review_group_message(&testers, &publisher),
                    )
                    .await?;
                }
            }
        } else {
            info!("Ignoring update: {update}")
        }
        Ok(())
    }
}
