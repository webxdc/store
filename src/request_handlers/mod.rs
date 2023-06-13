//! Handlers for the different messages the bot receives
use crate::{
    db::RecordId,
    utils::{ne_assign, ne_assign_option, read_string, read_vec},
};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use deltachat::webxdc::WebxdcManifest;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::path::{Path, PathBuf};
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

    /// Description of the application.
    pub description: Option<String>,
}

#[derive(TS, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct AppInfo {
    pub id: RecordId,
    pub name: String,                    // manifest
    pub author_name: String,             // bot
    pub author_email: String,            // bot
    pub source_code_url: Option<String>, // manifest
    pub image: Option<String>,           // webxdc
    pub description: Option<String>,     // submit
    pub version: Option<String>,         // manifest
    #[serde(skip)]
    pub xdc_blob_dir: Option<PathBuf>, // bot
    #[serde(skip)]
    pub originator: RecordId, // bot
    #[serde(skip)]
    pub active: bool,  // bot
}

impl AppInfo {
    /// Create appinfo from webxdc file.
    pub async fn from_xdc(file: &Path) -> anyhow::Result<Self> {
        let mut app = AppInfo::default();
        app.xdc_blob_dir = Some(file.to_path_buf());
        app.update_from_xdc(file.to_path_buf()).await?;
        Ok(app)
    }

    /// Reads a webxdc file and overwrites current fields.
    /// Returns whether the xdc was `changed` and `upgraded`
    /// Upgrade means the version has changed.
    pub async fn update_from_xdc(&mut self, file: PathBuf) -> anyhow::Result<(bool, bool)> {
        let mut upgraded = false;
        let mut changed = false;

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
            ne_assign_option(&mut self.image, Some(encode(&res)), &mut changed);
        }
        Ok((changed, upgraded))
    }

    pub fn update_from_request(self, app_info: AppInfo) -> Self {
        Self {
            id: self.id,
            name: app_info.name,
            author_name: app_info.author_name,
            author_email: self.author_email,
            source_code_url: self.source_code_url,
            image: self.image,
            description: app_info.description,
            version: self.version,
            xdc_blob_dir: self.xdc_blob_dir,
            originator: self.originator,
            active: self.active,
        }
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
