//! Handlers for the different messages the bot receives
use crate::db::DB;
use async_zip::tokio::read::fs::ZipFileReader;
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
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,                    // manifest
    pub author_name: String,             // bot
    pub author_email: Option<String>,    // bot
    pub source_code_url: Option<String>, // manifest
    pub image: Option<String>,           // webxdc
    pub description: String,             // submit
    pub xdc_blob_dir: Option<PathBuf>,   // bot
    pub version: Option<String>,         // manifest
    #[serde(skip)]
    #[serde(default = "default_thing")]
    pub originator: RecordId, // bot
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

        /* let icon = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.entry().filename().as_str().unwrap() == "icon.png")
            .map(|a| a.0);

        if let Some(index) = icon {
            let res = read_string(&reader, index).await.unwrap();
            self.image = Some(res);
        } */
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
}

pub async fn read_string(reader: &ZipFileReader, index: usize) -> anyhow::Result<String> {
    let mut entry = reader.reader_with_entry(index).await?;
    let mut data = String::new();
    entry.read_to_string_checked(&mut data).await?;
    Ok(data)
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
    ReviewPool,
    TesterPool,
    Release,
    Shop,
}

#[derive(Deserialize)]
pub struct WebxdcStatusUpdate<T> {
    payload: T,
}

pub mod release {
    use std::sync::Arc;

    use crate::bot::State;
    use deltachat::{
        chat::{self, ChatId},
        context::Context,
        message::Message,
    };
    use log::info;

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
        let missing = app_info.generate_missing_list();

        if !missing.is_empty() {
            chat::send_text_msg(
                context,
                chat_id,
                format!("Missing fields: {}", missing.join(", ")),
            )
            .await?;
        } else {
            chat::send_text_msg(context, chat_id, "All fields are present".into()).await?;
        }

        Ok(())
    }
}

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
        contact::Contact,
        context::Context,
        message::{Message, MsgId, Viewtype},
    };
    use log::info;
    use serde::Deserialize;
    use serde_json::json;
    use std::sync::Arc;
    use surrealdb::sql::{Id, Thing};
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

        let createor_contact = Contact::load_from_db(context, creator).await?;

        state
            .db
            .create_app_info(
                &AppInfo {
                    name: data.name.clone(),
                    author_email: Some(createor_contact.get_addr().to_string()),
                    author_name: createor_contact.get_name().to_string(),
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
