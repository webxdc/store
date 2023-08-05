//! Handlers for the different messages the bot receives
use crate::{
    db::RecordId,
    utils::{get_webxdc_manifest, read_vec},
};
use anyhow::{Context as _, Result};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::fs::File;
use ts_rs::TS;

pub mod store;

#[derive(Deserialize)]
pub struct WebxdcManifest {
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

#[derive(TS, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
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
    #[serde(skip)]
    pub removed: bool,
}

impl AppInfo {
    /// Create appinfo from webxdc file.
    pub async fn from_xdc(file: &Path) -> Result<Self> {
        let size = i64::try_from(File::open(&file).await?.metadata().await?.len())?;
        let reader = ZipFileReader::new(&file).await?;
        let entries = reader.file().entries();
        let manifest = get_webxdc_manifest(&reader).await?;

        let image = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (index, entry.entry().filename().as_str().unwrap_or_default()))
            .find(|(_, name)| *name == "icon.png" || *name == "icon.jpg");
        let image = if let Some((index, name)) = image {
            let res = read_vec(&reader, index).await?;
            let ending = name
                .split('.')
                .nth(1)
                .context(format!("Can't extract file ending from {name}"))?;
            let base64 = encode(&res);
            Ok(format!("data:image/{ending};base64,{base64}"))
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
            removed: false,
        })
    }
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
    UpdateWebxdc {
        /// Old serial of the store.
        serial: u32,
    },

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
        #[ts(type = "Record<string, (Partial<AppInfo> & {app_id: string} | null)>")]
        app_infos: Value,
        /// The newest serial of the bot.    
        serial: u32,
        /// The old serial that the request was also made with.
        /// If it is a full [AppInfo] update, this will be 0.
        old_serial: u32,
        /// `app_id`s of apps that will receive an update.
        /// The frontend can use these to set the state to updating.
        updating: Vec<String>,
    },
    /// First message send to the store xdc together containing all [AppInfo]s.
    Init {
        /// List of initial AppInfos.
        app_infos: Vec<AppInfo>,
        /// Last serial of the store.
        serial: u32,
    },
}
