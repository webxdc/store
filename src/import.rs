use anyhow::{bail, Context};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use futures::future::join_all;
use serde::Deserialize;
use sqlx::SqliteConnection;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::fs::File;

use crate::{
    request_handlers::AppInfo,
    utils::{maybe_upgrade_xdc, read_vec, AddType},
};

#[derive(Deserialize, Debug)]
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

    /// Date displayed in the store.
    pub date: String,

    pub cache_relname: PathBuf,
}

pub async fn import_many(
    path: &Path,
    xdcs_path: PathBuf,
    conn: &mut SqliteConnection,
) -> anyhow::Result<()> {
    let sources_lock =
        fs::read_to_string(path.join("sources.lock")).context("Failed to read sources.lock")?;
    let xdc_metas: HashMap<String, WexbdcManifest> = toml::from_str(&sources_lock)?;

    let mut xdcs = vec![];
    for xdc in xdc_metas.into_values() {
        let path = PathBuf::from(path).join(&xdc.cache_relname);
        xdcs.push(tokio::spawn(async move {
            // compute file size
            let size = i64::try_from(
                File::open(&path)
                    .await
                    .context("Can't open cache_relname")?
                    .metadata()
                    .await?
                    .len(),
            )?;
            // extract icon
            let reader = ZipFileReader::new(&path).await?;
            let entries = reader.file().entries();
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
                encode(&res)
            } else {
                bail!("Could not find image")
            };

            Ok(AppInfo {
                id: 0,
                app_id: xdc.app_id,
                version: xdc.version,
                date: OffsetDateTime::parse(&xdc.date, &Rfc3339)?.unix_timestamp(),
                name: xdc.name, // xdc.name,
                submitter_uri: None,
                source_code_url: xdc.source_code_url,
                image,
                description: xdc.description,
                xdc_blob_path: path,
                size,
            })
        }))
    }

    let xdcs = join_all(xdcs).await;
    if xdcs.is_empty() {
        eprintln!("No xdcs from {} added", path.display());
        return Ok(());
    }

    let mut added = Vec::new();
    let mut updated = Vec::new();
    let mut ignored = Vec::new();
    let mut failed = 0;

    for file in xdcs.into_iter() {
        let app_info = file?;
        match app_info {
            Ok(mut app_info) => match maybe_upgrade_xdc(&mut app_info, conn, &xdcs_path).await {
                Ok(AddType::Added) => added.push(app_info.xdc_blob_path),
                Ok(AddType::Updated) => updated.push(app_info.xdc_blob_path),
                Ok(AddType::Ignored) => ignored.push(app_info.xdc_blob_path),
                Err(e) => {
                    eprintln!("{e:#}");
                    failed += 1;
                }
            },
            Err(e) => {
                eprintln!("{e:#}");
                failed += 1;
            }
        }
    }

    for (list, name) in vec![added, updated, ignored]
        .into_iter()
        .zip(&["Added", "Updated", "Ignored"])
    {
        if list.is_empty() {
            println!("{name}: None");
        } else {
            println!("{name}:");
            for file in list {
                println!("- {}", file.display());
            }
        }
    }
    if failed > 0 {
        eprintln!("Failed: {}", failed);
    }
    Ok(())
}

/// Add a single webxdc to the store
/// - Add it to the db
/// - Copy it into the `dest` location
pub async fn import_one(
    file: &Path,
    dest: &Path,
    conn: &mut SqliteConnection,
) -> anyhow::Result<AddType> {
    if !file
        .to_str()
        .context("can't convert to str")?
        .ends_with(".xdc")
    {
        bail!("File does not end with .xdc");
    }

    let mut app_info = AppInfo::from_xdc(file)
        .await
        .context(anyhow::anyhow!("Failed to load {}", file.display()))?;
    app_info.submitter_uri = Some("xdcstore".to_string());

    // Add it to the db
    maybe_upgrade_xdc(&mut app_info, conn, dest).await
}
