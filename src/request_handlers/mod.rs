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

pub mod genisis;
pub mod release;
pub mod shop;

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
    pub xdc_blob_dir: Option<PathBuf>,   // bot
    pub version: Option<String>,         // manifest
    #[serde(default = "default_thing")]
    #[ts(skip)]
    pub originator: RecordId, // bot
    pub active: bool,                    // bot
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfoId {
    #[serde(flatten)]
    pub app_info: AppInfo,
    pub id: Thing,
}

impl AppInfo {
    pub async fn update_from_xdc(&mut self, file: PathBuf) -> anyhow::Result<()> {
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

    pub fn generate_missing_list(&self) -> Vec<String> {
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
