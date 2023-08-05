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

/// `manifest.toml` structure.
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

/// Information about a single application in the store index.
#[derive(TS, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct AppInfo {
    #[allow(clippy::missing_docs_in_private_items)]
    #[serde(skip)]
    pub id: RecordId,

    /// Application ID, e.g. `webxdc-poll`.
    pub app_id: String,

    /// Release tag, e.g. `v2.2.0`.
    pub tag_name: String,

    /// Date as a timestamp in seconds.
    pub date: i64,

    /// Application name, e.g. `Checklist`.
    pub name: String,

    /// Source code URL, e.g. `https://codeberg.org/webxdc/checklist`.
    pub source_code_url: String,

    /// Application icon encoded as a data URL,
    /// for example `data:image/png;base64,...`.
    pub image: String,

    /// Human-readable application description.
    pub description: String,

    /// Application size in bytes.
    pub size: i64,

    /// Absolute path to the .xdc file.
    #[serde(skip)]
    pub xdc_blob_path: PathBuf,

    /// True if the application has been removed.
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
    /// `payload` field of the WebXDC update.
    pub payload: WebxdcStatusUpdatePayload,
}

/// WebXDC status update payload.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
#[serde(tag = "type")]
pub enum WebxdcStatusUpdatePayload {
    /// Request sent from the frontend to the bot
    /// when user clicks "Download"
    /// in the dialog notifying about
    /// the newer version of the `store.xdc`.
    UpdateWebxdc {
        /// Old serial of the store index.
        serial: u32,
    },

    /// Response sent by the bot if the bot receives a request
    /// to a previously sent `store.xdc` with a `tag_name`
    /// different `tag_name` from the current one.
    Outdated {
        /// Always true.
        critical: bool,

        /// `tag_name` field from the `manifest.toml` of the actual `store.xdc`.
        tag_name: String,
    },

    /// Response sent to an outdated version of `store.xdc`
    /// when the user requested a new `store.xdc` with an `UpdateWebxdc` request.
    ///
    /// This response is used by the frontend to display
    /// instructions for the user to look for an updated `store.xdc` in the chat.
    UpdateSent,

    /// Request to update the application index
    /// sent by the `store.xdc` frontend to the bot.
    UpdateRequest {
        /// Requested update sequence number.
        serial: u32,
        /// List of apps selected for caching.
        #[serde(default)]
        apps: Vec<(String, String)>,
    },

    /// Request to download the application .xdc
    /// sent by the frontend to the bot.
    Download {
        /// ID of the requested application.
        app_id: String,
    },

    /// Successful response to the download request.
    DownloadOkay {
        /// app_id of the downloaded app.
        app_id: String,

        /// Name to be used as filename in `sendToChat`.
        name: String,

        /// Base64 encoded webxdc.
        data: String,
    },

    /// Negative response to the download request.
    DownloadError {
        /// Application ID of the requested app.
        app_id: String,

        /// Error message.
        error: String,
    },

    /// Index update response sent by the bot to the frontend.
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
