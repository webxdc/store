//! Handlers for the different messages the bot receives
use std::path::PathBuf;

use deltachat::{chat::ChatId, contact::ContactId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct AppInfo {
    pub name: String,
    pub author_name: String,
    pub author_email: Option<String>,
    pub source_code_url: Option<String>,
    pub image: Option<String>,
    pub description: String,
    pub xdc_blob_dir: Option<PathBuf>,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ReviewChat {
    pub chat_id: ChatId,
    pub publisher: ContactId,
    pub testers: Vec<ContactId>,
    pub creator: ContactId,
    pub ios: bool,
    pub android: bool,
    pub desktop: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ChatType {
    ReviewPool,
    TesterPool,
    Release,
    Shop,
}

#[derive(Deserialize)]
pub struct WebxdcStatusUpdate<T> {
    payload: T,
}

pub mod review {}

pub mod shop {
    use super::{AppInfo, ReviewChat};
    use crate::{
        bot::State,
        messages::{appstore_message, creat_review_group_init_message},
        request_handlers::WebxdcStatusUpdate,
        utils::{get_contact_name, get_oon_peer},
    };
    use deltachat::{
        chat::{self, ChatId, ProtectionStatus},
        context::Context,
        message::{Message, MsgId, Viewtype},
    };
    use log::info;
    use serde::Deserialize;
    use serde_json::json;
    use std::sync::Arc;
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
    pub struct PublishRequest {
        pub name: String,
        pub description: String,
    }

    #[derive(Deserialize)]
    struct StoreRequest {
        request_type: RequestType,
    }

    #[derive(TS, Deserialize)]
    #[ts(export)]
    #[allow(unused)]
    struct StoreRequestWithData<T> {
        request_type: RequestType,
        data: T,
    }

    pub async fn handle_message(context: &Context, chat_id: ChatId) -> anyhow::Result<()> {
        // Handle normal messages to the bot (resend the store itself).
        chat::send_text_msg(context, chat_id, appstore_message().to_string()).await?;
        let mut webxdc_msg = Message::new(Viewtype::Webxdc);
        webxdc_msg.set_file("./appstore-bot.xdc", None);
        chat::send_msg(context, chat_id, &mut webxdc_msg)
            .await
            .unwrap();
        Ok(())
    }

    pub async fn handle_status_update(
        context: &Context,
        state: Arc<State>,
        chat_id: ChatId,
        msg_id: MsgId,
        update: String,
    ) -> anyhow::Result<()> {
        if let Ok(req) = serde_json::from_str::<WebxdcStatusUpdate<StoreRequest>>(&update) {
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
                    info!("Handling store publish");
                    let data = serde_json::from_str::<
                        WebxdcStatusUpdate<StoreRequestWithData<PublishRequest>>,
                    >(&update)?
                    .payload
                    .data;
                    handle_publish(context, state, chat_id, data).await?;
                }
            }
        } else {
            info!("Ignoring update: {update}")
        }
        Ok(())
    }

    pub async fn handle_publish(
        context: &Context,
        state: Arc<State>,
        chat_id: ChatId,
        data: PublishRequest,
    ) -> anyhow::Result<()> {
        // get publisher and testers
        let publisher = state.db.get_publisher().await.unwrap();
        let testers = state.db.get_testers().await.unwrap();

        let creator = get_oon_peer(context, chat_id).await?;

        // create the new chat
        let chat_id = chat::create_group_chat(
            context,
            ProtectionStatus::Unprotected,
            &format!("Publish: {}", data.name),
        )
        .await?;

        // add all chat members
        for tester in testers.iter() {
            chat::add_contact_to_chat(context, chat_id, *tester).await?;
        }
        chat::add_contact_to_chat(context, chat_id, publisher).await?;
        chat::add_contact_to_chat(context, chat_id, creator).await?;

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

        // add new chat to local state
        state
            .db
            .create_chat(&ReviewChat {
                chat_id,
                publisher,
                testers: testers.clone(),
                creator,
                ios: false,
                android: false,
                desktop: false,
            })
            .await?;

        state
            .db
            .set_chat_type(chat_id, super::ChatType::Release)
            .await?;

        state
            .db
            .create_app_info(
                &AppInfo {
                    name: data.name.clone(),
                    description: data.description,
                    ..Default::default()
                },
                &chat_id,
            )
            .await?;
        Ok(())
    }
}
