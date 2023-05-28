//! Handlers for the different messages the bot receives
use crate::utils::{ne_assign, ne_assign_option, read_string, read_vec};
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

#[derive(Deserialize)]
pub struct ExtendedWebxdcManifest {
    #[serde(flatten)]
    webxdc_manifest: WebxdcManifest,

    /// Version of the application.
    pub version: Option<String>,

    /// Version of the application.
    pub description: Option<String>,
}

#[derive(TS, Deserialize, Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,                    // manifest
    pub author_name: String,             // bot
    pub author_email: String,            // bot
    pub source_code_url: Option<String>, // manifest
    pub image: Option<String>,           // webxdc
    pub description: Option<String>,     // submit
    pub xdc_blob_dir: Option<PathBuf>,   // bot
    pub version: Option<String>,         // manifest
    #[serde(default = "default_thing")]
    #[ts(skip)]
    pub originator: RecordId, // bot
    pub active: bool,                    // bot
}

impl AppInfo {
    /// Create appinfo from webxdc file.
    pub async fn from_xdc(file: &Path) -> anyhow::Result<Self> {
        let mut app = AppInfo::default();
        app.update_from_xdc(file.to_path_buf()).await?;
        Ok(app)
    }

    /// Reads a webxdc file and overwrites current fields.
    /// Returns whether the xdc was `changed` and `upgraded`
    /// Upgrade means the version has changed.
    pub async fn update_from_xdc(&mut self, file: PathBuf) -> anyhow::Result<(bool, bool)> {
        let mut upgraded = false;
        let mut changed = false;

        let reader = ZipFileReader::new(&file).await.unwrap();
        let entries = reader.file().entries();
        let manifest = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.entry().filename().as_str().unwrap() == "manifest.toml")
            .map(|a| a.0);

        if let Some(index) = manifest {
            let res = read_string(&reader, index).await.unwrap();
            let manifest: ExtendedWebxdcManifest = toml::from_str(&res)?;

            ne_assign(&mut self.name, manifest.webxdc_manifest.name, &mut changed);
            ne_assign_option(
                &mut self.source_code_url,
                manifest.webxdc_manifest.source_code_url,
                &mut changed,
            );
            ne_assign_option(&mut self.version, manifest.version, &mut upgraded);
            if upgraded {
                changed = true
            }
            ne_assign_option(&mut self.description, manifest.description, &mut changed);
        }

        ne_assign_option(&mut self.xdc_blob_dir, Some(file), &mut changed);

        let icon = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.entry().filename().as_str().unwrap() == "icon.png")
            .map(|a| a.0);

        if let Some(index) = icon {
            let res = read_vec(&reader, index).await.unwrap();
            ne_assign_option(&mut self.image, Some(encode(&res)), &mut changed);
        }
        Ok((changed, upgraded))
    }

    /// Generates a list of missing values from the appinfo.
    pub fn generate_missing_list(&self) -> Vec<String> {
        let mut missing: Vec<String> = vec![];
        if self.name.is_empty() {
            missing.push("name".to_string());
        }
        if self.author_name.is_empty() {
            missing.push("author name".to_string());
        }
        if self.author_email.is_empty() {
            missing.push("author email".to_string());
        }
        if self.description.is_none() {
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
