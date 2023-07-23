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
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::fs::File;
use ts_rs::TS;

pub mod genesis;
pub mod store;

#[derive(Deserialize)]
pub struct WexbdcManifest {
    /// Webxdc application identifier.
    pub app_id: String,

    /// Tag Name of the application.
    pub tag_name: String,

    /// Webxdc name, used on icons or page titles.
    pub name: String,

    /// Description of the application.
    pub description: String,

    /// URL of webxdc source code.
    pub source_code_url: String,

    /// Date displayed in the store.
    pub date: String,
}

#[derive(TS, Deserialize, Serialize, Clone, Debug, Default, PartialEq, FromRow, Decode)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct AppInfo {
    #[serde(skip)]
    pub id: RecordId,
    pub app_id: String,
    pub tag_name: String,
    pub date: i64,
    pub name: String,
    pub source_code_url: String,
    pub image: String,
    pub description: String,
    pub size: i64,
    #[serde(skip)]
    pub xdc_blob_path: PathBuf,
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
                    .map(|name| name == "icon.png" || name == "icon.jpg")
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
            date: OffsetDateTime::parse(&manifest.date, &Rfc3339)?.unix_timestamp(),
            app_id: manifest.app_id,
            tag_name: manifest.tag_name,
            name: manifest.name,
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

/// WebXDC status update.
#[derive(Serialize, Deserialize)]
pub struct WebxdcStatusUpdate {
    pub payload: WebxdcStatusUpdatePayload,
}

/// WebXDC status update payload.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[serde(tag = "type")]
pub enum WebxdcStatusUpdatePayload {
    // General update request.
    UpdateWebxdc,

    // General update response.
    Outdated {
        critical: bool,
        tag_name: String,
    },
    UpdateSent,

    // Store WebXDC requests.
    UpdateRequest {
        /// Requested update sequence number.
        serial: u32,
        /// List of apps selected for caching.
        #[serde(default)]
        apps: Vec<(String, String)>,
    },
    Download {
        /// ID of the requested application.
        app_id: String,
    },

    // Store bot responses.
    DownloadOkay {
        /// app_id of the downloaded app.
        app_id: String,
        /// Name to be used as filename in `sendToChat`.
        name: String,
        /// Base64 encoded webxdc.
        data: String,
    },
    DownloadError {
        app_id: String,
        error: String,
    },
    Update {
        /// List of new / updated app infos.
        app_infos: Vec<AppInfo>,
        serial: i32,
        /// `app_id`s of apps that will receive an update.
        /// The frontend can use these to set the state to updating.
        updating: Vec<String>,
    },
}
