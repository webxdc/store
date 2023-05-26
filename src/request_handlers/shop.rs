use super::{AppInfo, FrontendRequest, ReviewChat};
use crate::{
    bot::State,
    messages::{appstore_message, creat_review_group_init_message},
    request_handlers::FrontendRequestWithData,
    utils::{get_contact_name, get_oon_peer, send_webxdc},
};
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
use thiserror::Error;
use ts_rs::TS;

#[derive(TS, Deserialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]

enum RequestType {
    Update,
    Dowload,
    Publish,
}

#[derive(TS, Deserialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]

pub struct PublishRequest {
    pub name: String,
    pub description: String,
}

pub async fn handle_message(context: &Context, chat_id: ChatId) -> anyhow::Result<()> {
    // Handle normal messages to the bot (resend the store itself).
    let chat = chat::Chat::load_from_db(context, chat_id).await?;
    if let constants::Chattype::Single = chat.typ {
        chat::send_text_msg(context, chat_id, appstore_message().to_string()).await?;
        send_webxdc(context, chat_id, "./appstore.xdc").await?;
    }
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
                let apps = state.get_apps().await?;
                info!("Handling store update request");
                context
                    .send_webxdc_status_update_struct(
                        msg_id,
                        deltachat::webxdc::StatusUpdateItem {
                            payload: json! {apps},
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
            RequestType::Publish => {
                info!("Handling store publish");
                let data = serde_json::from_str::<
                    FrontendRequestWithData<RequestType, PublishRequest>,
                >(&update)?
                .payload
                .data;
                if let Err(e) = handle_publish(context, state.clone(), chat_id, data).await {
                    match e {
                        HandlePublishError::NotEnoughTesters => {
                            chat::send_text_msg(context, state.config.genesis_group, "Tried to create review chat, but there are not enough testers available".into()).await?;
                        }
                        HandlePublishError::NotEnoughReviewee => {
                            chat::send_text_msg(context, state.config.genesis_group, "Tried to create review chat, but there are not enough publishers available".into()).await?;
                        }
                        e => return Err(anyhow::anyhow!(e)),
                    }
                }
            }
        }
    } else {
        info!("Ignoring update: {}", &update[..10])
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum HandlePublishError {
    #[error("Not enough testers in pool")]
    NotEnoughTesters,
    #[error("Not enough reviewee in pool")]
    NotEnoughReviewee,
    #[error(transparent)]
    SurrealDb(#[from] surrealdb::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub async fn handle_publish(
    context: &Context,
    state: Arc<State>,
    chat_id: ChatId,
    data: PublishRequest,
) -> Result<(), HandlePublishError> {
    // get publisher and testers
    let publisher = state
        .db
        .get_publisher()
        .await?
        .ok_or(HandlePublishError::NotEnoughReviewee)?;

    let testers = state.db.get_testers().await?;

    if testers.is_empty() {
        return Err(HandlePublishError::NotEnoughTesters);
    }

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
        creat_review_group_init_message(&tester_names, &get_contact_name(context, publisher).await),
    )
    .await?;

    // add new chat to local state
    let resource_id = Thing {
        tb: "app_info".to_string(),
        id: Id::rand(),
    };

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
            app_info: resource_id.clone(),
        })
        .await?;

    state
        .db
        .set_chat_type(chat_id, super::ChatType::Release)
        .await?;

    let creator_contact = Contact::load_from_db(context, creator).await?;

    state
        .db
        .create_app_info(
            &AppInfo {
                name: data.name.clone(),
                author_email: Some(creator_contact.get_addr().to_string()),
                author_name: creator_contact.get_name().to_string(),
                description: data.description,
                originator: Thing {
                    tb: "chat".to_string(),
                    id: Id::Number(chat_id.to_u32() as i64),
                },
                ..Default::default()
            },
            resource_id,
        )
        .await?;
    Ok(())
}
