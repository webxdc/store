//! Handlers for the different messages the bot receives
use crate::{
    db::RecordId,
    utils::{read_string, read_vec},
};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::path::{Path, PathBuf};
use ts_rs::TS;

pub mod genisis;
pub mod review;
pub mod shop;
pub mod submit;

#[derive(Deserialize)]
pub struct WexbdcManifest {
    /// Webxdc application identifier.
    pub app_id: String,

    /// Version of the application.
    pub version: String,

    /// Webxdc name, used on icons or page titles.
    pub name: String,

    /// Description of the application.
    pub description: String,

    /// URL of webxdc source code.
    pub source_code_url: Option<String>,

    /// Uri of the submitter.
    pub submitter_uri: Option<String>,
}

#[derive(TS, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct AppInfo {
    pub id: RecordId,
    #[serde(skip)]
    pub app_id: String, // manifest
    pub version: String,                 // manifest
    pub name: String,                    // manifest
    pub submitter_uri: Option<String>,   // bot
    pub source_code_url: Option<String>, // manifest
    pub image: String,                   // webxdc
    pub description: String,             // submit
    #[serde(skip)]
    pub xdc_blob_dir: PathBuf, // bot
    #[serde(skip)]
    pub originator: RecordId, // bot
    #[serde(skip)]
    pub active: bool,  // bot
}

impl AppInfo {
    /// Create appinfo from webxdc file.
    pub async fn from_xdc(file: &Path) -> anyhow::Result<Self> {
        let mut app = AppInfo {
            xdc_blob_dir: file.to_path_buf(),
            ..Default::default()
        };
        app.update_from_xdc(file.to_path_buf()).await?;
        Ok(app)
    }

    /// Reads a webxdc file and overwrites current fields.
    /// Returns true if the version has changed.
    pub async fn update_from_xdc(&mut self, file: PathBuf) -> anyhow::Result<bool> {
        let mut upgraded = false;
        let reader = ZipFileReader::new(&file).await?;
        let entries = reader.file().entries();
        let manifest = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                entry
                    .entry()
                    .filename()
                    .as_str()
                    .map(|name| name == "manifest.toml")
                    .unwrap_or_default()
            })
            .map(|a| a.0);

        if let Some(index) = manifest {
            let res = read_string(&reader, index).await?;
            let manifest: WexbdcManifest = toml::from_str(&res)?;
            self.app_id = manifest.app_id;
            if self.version != manifest.version {
                upgraded = true
            }
            self.version = manifest.version;
            self.name = manifest.name;
            self.description = manifest.description;
            self.source_code_url = manifest.source_code_url;
            self.submitter_uri = manifest.submitter_uri;
            // self.submission_date = manifest.submission_date;
        }

        self.xdc_blob_dir = file;

        let icon = entries
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                entry
                    .entry()
                    .filename()
                    .as_str()
                    .map(|name| name == "icon.png")
                    .unwrap_or_default()
            })
            .map(|a| a.0);

        if let Some(index) = icon {
            let res = read_vec(&reader, index).await?;
            self.image = encode(&res)
        }
        Ok(upgraded)
    }

    pub fn update_from_request(self, app_info: AppInfo) -> Self {
        Self {
            submitter_uri: app_info.submitter_uri,
            ..self
        }
    }
}

#[derive(Serialize, Deserialize, Type, Clone, Copy, Debug, PartialEq)]

pub enum ChatType {
    Shop,
    Submit,
    Review,
    Genesis,
    ReviewPool,
    TesterPool,
}

/// A generic webxdc update
#[derive(Deserialize)]
pub struct WebxdcStatusUpdate<T> {
    payload: T,
}
