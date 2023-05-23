//! Handlers for the different messages the bot receives
use crate::{
    db::DB,
    utils::{read_string, read_vec},
};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use deltachat::{chat::ChatId, contact::ContactId, webxdc::WebxdcManifest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use surrealdb::{
    opt::RecordId,
    sql::{Id, Thing},
};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,                    // manifest
    pub author_name: String,             // bot
    pub author_email: Option<String>,    // bot
    pub source_code_url: Option<String>, // manifest
    pub image: Option<String>,           // webxdc
    pub description: String,             // submit
    #[serde(skip)]
    pub xdc_blob_dir: Option<PathBuf>, // bot
    pub version: Option<String>,         // manifest
    #[serde(skip)]
    #[serde(default = "default_thing")]
    pub originator: RecordId, // bot
    #[serde(skip)]
    pub active: bool,  // bot
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfoId {
    #[serde(flatten)]
    pub app_info: AppInfo,
    pub id: Thing,
}

impl AppInfo {
    async fn update_from_xdc(&mut self, file: PathBuf) -> anyhow::Result<()> {
        let reader = ZipFileReader::new(&file).await.unwrap();
        self.xdc_blob_dir = Some(file);
        let entries = reader.file().entries();
        let manifest = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.entry().filename().as_str().unwrap() == "manifest.toml")
            .map(|a| a.0);

        if let Some(index) = manifest {
            let res = read_string(&reader, index).await.unwrap();
            let manifest = WebxdcManifest::from_string(&res)?;

            if let Some(name) = manifest.name {
                self.name = name;
            }
            if let Some(source_code_url) = manifest.source_code_url {
                self.source_code_url = Some(source_code_url);
            }
            if let Some(version) = manifest.version {
                self.version = Some(version);
            }
        }

        let icon = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.entry().filename().as_str().unwrap() == "icon.png")
            .map(|a| a.0);

        if let Some(index) = icon {
            let res = read_vec(&reader, index).await.unwrap();
            self.image = Some(encode(&res));
        }
        Ok(())
    }

    fn generate_missing_list(&self) -> Vec<String> {
        let mut missing = vec![];
        if self.name.is_empty() {
            missing.push("name".to_string());
        }
        if self.description.is_empty() {
            missing.push("description".to_string());
        }
        if self.image.is_none() {
            missing.push("image".to_string());
        }
        if self.source_code_url.is_none() {
            missing.push("source_code_url".to_string());
        }
        if self.version.is_none() {
            missing.push("version".to_string());
        }
        missing
    }

    pub fn is_complete(&self) -> bool {
        self.generate_missing_list().is_empty()
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            author_name: Default::default(),
            author_email: Default::default(),
            source_code_url: Default::default(),
            image: Default::default(),
            description: Default::default(),
            xdc_blob_dir: Default::default(),
            version: Default::default(),
            active: Default::default(),
            originator: default_thing(),
        }
    }
}

fn default_thing() -> Thing {
    Thing {
        tb: "hi".to_string(),
        id: Id::rand(),
    }
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
    pub app_info: RecordId,
}

impl ReviewChat {
    pub async fn get_app_info(&self, db: &DB) -> surrealdb::Result<AppInfo> {
        db.get_app_info(&self.app_info).await
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ChatType {
    Genesis,
    ReviewPool,
    TesterPool,
    Release,
    Shop,
}

/// A generic webxdc update
#[derive(Deserialize)]
pub struct WebxdcStatusUpdate<T> {
    payload: T,
}

#[derive(Deserialize)]
struct Request<T> {
    request_type: T,
}

#[derive(TS, Deserialize)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[allow(unused)]
struct RequestWithData<T, R> {
    request_type: T,
    data: R,
}

type FrontendRequest<T> = WebxdcStatusUpdate<Request<T>>;
type FrontendRequestWithData<T, R> = WebxdcStatusUpdate<RequestWithData<T, R>>;

pub mod release {
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
}

pub mod shop {
    use super::{AppInfo, FrontendRequest, ReviewChat};
    use crate::{
        bot::State,
        messages::{appstore_message, creat_review_group_init_message},
        request_handlers::FrontendRequestWithData,
        utils::{get_contact_name, get_oon_peer, send_webxdc},
    };
    use deltachat::{
        chat::{self, ChatId, ProtectionStatus},
        contact::Contact,
        context::Context,
        message::{Message, MsgId, Viewtype},
    };
    use log::info;
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
        chat::send_text_msg(context, chat_id, appstore_message().to_string()).await?;
        send_webxdc(context, chat_id, "./appstore.xdc").await
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
                    info!("Handling store update");
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
                    let data =
                        serde_json::from_str::<FrontendRequestWithData<RequestType, String>>(
                            &update,
                        )?
                        .payload
                        .data;
                    let mut parts = data.split(":");
                    let app = state
                        .db
                        .get_app_info(&Thing {
                            tb: parts.next().unwrap().into(),
                            id: parts.next().unwrap().into(),
                        })
                        .await?;

                    let mut msg = Message::new(Viewtype::Webxdc);
                    msg.set_file(&app.xdc_blob_dir.unwrap().to_str().unwrap(), None);
                    chat::send_msg(context, chat_id, &mut msg).await.unwrap();
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
            .await
            .map_err(|_| HandlePublishError::NotEnoughReviewee)?;

        let testers = state.db.get_testers().await?;

        if testers.len() < 1 {
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
            creat_review_group_init_message(
                &tester_names,
                &get_contact_name(context, publisher).await,
            ),
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
}

pub mod genisis {
    use std::sync::Arc;

    use clap::{CommandFactory, FromArgMatches};
    use deltachat::{
        chat,
        context::Context,
        message::{Message, MsgId},
    };

    use crate::{bot::State, cli::Genesis};

    pub async fn handle_message(
        context: &Context,
        state: Arc<State>,
        msg_id: MsgId,
    ) -> anyhow::Result<()> {
        let msg = Message::load_from_db(context, msg_id).await?;

        if let Some(text) = msg.get_text() {
            // only react to messages with right keywoard
            if text.starts_with("/") {
                match <Genesis as CommandFactory>::command()
                    .try_get_matches_from(text[1..].split(' '))
                {
                    Ok(mut matches) => {
                        let res = <Genesis as FromArgMatches>::from_arg_matches_mut(&mut matches)?;

                        match res.join {
                            crate::cli::GroupName::Join { name } => {
                                let contact_id = msg.get_from_id();

                                let chat_id = match name {
                                    crate::cli::BotGroup::Genesis => state.config.genesis_group,
                                    crate::cli::BotGroup::Reviewee => state.config.reviewee_group,
                                    crate::cli::BotGroup::Tester => state.config.tester_group,
                                };

                                chat::add_contact_to_chat(context, chat_id, contact_id).await?
                            }
                        }
                    }
                    Err(_) => todo!(),
                };
            }
        }
        Ok(())
    }
}
