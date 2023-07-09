//! Handlers for the different messages the bot receives
use crate::{
    db::RecordId,
    utils::{get_webxdc_manifest, read_vec},
};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, FromRow, Type};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use ts_rs::TS;

pub mod genesis;
pub mod store;

#[derive(Deserialize)]
pub struct WexbdcManifest {
    /// Webxdc application identifier.
    pub app_id: String,

    /// Version of the application.
    pub version: u32,

    /// Webxdc name, used on icons or page titles.
    pub name: String,

    /// Description of the application.
    pub description: String,

    /// URL of webxdc source code.
    pub source_code_url: Option<String>,

    /// Uri of the submitter.
    pub submitter_uri: Option<String>,
}

#[derive(TS, Deserialize, Serialize, Clone, Debug, Default, PartialEq, FromRow, Decode)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct AppInfo {
    #[serde(skip)]
    pub id: RecordId,
    pub app_id: String, // manifest
    pub version: u32,   // manifest
    pub date: i64,      // manifest
    pub name: String,   // manifest
    #[serde(skip)]
    pub submitter_uri: Option<String>, // bot
    pub source_code_url: Option<String>, // manifest
    pub image: String,  // webxdc
    pub description: String, // submit
    #[serde(skip)]
    pub xdc_blob_path: PathBuf, // bot
    pub size: i64,      //bot
}

impl AppInfo {
    /// Create appinfo from webxdc file.
    pub async fn from_xdc(file: &Path) -> anyhow::Result<Self> {
        let size = i64::try_from(File::open(&file).await?.metadata().await?.len())?;
        let reader = ZipFileReader::new(&file).await?;
        let entries = reader.file().entries();
        let manifest = get_webxdc_manifest(&reader).await?;

        let image = entries
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
        let image = if let Some(index) = image {
            let res = read_vec(&reader, index).await?;
            Ok(encode(&res))
        } else {
            Err(anyhow::anyhow!("Could not find image"))
        };

        Ok(Self {
            size,
            date: 1688893410072,
            app_id: manifest.app_id,
            version: manifest.version,
            name: manifest.name,
            submitter_uri: manifest.submitter_uri,
            source_code_url: manifest.source_code_url,
            image: image?,
            description: manifest.description,
            xdc_blob_path: file.to_path_buf(),
            id: 0, // This will be updated by the db on insert
        })
    }
}

#[derive(Serialize, Deserialize, Type, Clone, Copy, Debug, PartialEq)]

pub enum ChatType {
    Store,
    Genesis,
}

/// A generic webxdc update
#[derive(Deserialize)]
pub struct WebxdcStatusUpdate<T> {
    pub payload: T,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[serde(tag = "type")]

pub enum GeneralFrontendResponse {
    Outdated { critical: bool, version: u32 },
    UpdateSent,
}

#[derive(Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[serde(tag = "type")]
pub enum GeneralFrontendRequest {
    UpdateWebxdc,
}
