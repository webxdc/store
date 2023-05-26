//! Handlers for the different messages the bot receives
use crate::utils::{read_string, read_vec};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use deltachat::webxdc::WebxdcManifest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use surrealdb::{
    opt::RecordId,
    sql::{Id, Thing},
};
use ts_rs::TS;

pub mod genisis;
pub mod review;
pub mod shop;
pub mod submit;

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

#[derive(Deserialize)]
pub struct ExtendedWebxdcManifest {
    #[serde(flatten)]
    webxdc_manifest: WebxdcManifest,

    /// Version of the application.
    pub version: Option<String>,

    /// Version of the application.
    pub description: Option<String>,
}

impl AppInfo {
    pub async fn from_xdc(file: &Path) -> anyhow::Result<Self> {
        let mut app = AppInfo::default();
        app.update_from_xdc(file.to_path_buf()).await?;
        Ok(app)
    }

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
            let manifest: ExtendedWebxdcManifest = serde_json::from_str(&res)?;

            if let Some(name) = manifest.webxdc_manifest.name {
                self.name = name;
            }
            if let Some(source_code_url) = manifest.webxdc_manifest.source_code_url {
                self.source_code_url = Some(source_code_url);
            }
            if let Some(version) = manifest.version {
                self.version = Some(version);
            }
            if let Some(description) = manifest.description {
                self.version = Some(description)
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ChatType {
    Genesis,
    ReviewPool,
    TesterPool,
    Review,
    Submit,
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
