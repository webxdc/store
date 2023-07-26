use anyhow::{bail, Context};
use async_zip::tokio::read::fs::ZipFileReader;
use base64::encode;
use futures::future::join_all;
use serde::Deserialize;
use sqlx::SqliteConnection;
use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    path::{Path, PathBuf},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::fs::{self, File};

use crate::{
    db,
    request_handlers::AppInfo,
    utils::{maybe_upgrade_xdc, read_vec, AddType},
};

#[derive(Deserialize, Debug)]
pub struct WexbdcManifest {
    /// Webxdc application identifier.
    pub app_id: String,

    /// Tag name of the application.
    pub tag_name: String,

    /// Webxdc name, used on icons or page titles.
    pub name: String,

    /// Description of the application.
    pub description: String,

    /// URL of webxdc source code.
    pub source_code_url: String,

    /// Date displayed in the store as a Rfc3339 timestamp.
    pub date: String,

    /// Relative path from the sources.ini file to the cached xdc.
    pub cache_relname: PathBuf,
}

pub async fn import_many(
    path: &Path,
    xdcs_path: PathBuf,
    conn: &mut SqliteConnection,
) -> anyhow::Result<()> {
    let xdcget_lock = fs::read_to_string(path.join("xdcget.lock"))
        .await
        .context("Failed to read xdcget.lock")?;
    let xdc_metas: HashMap<String, WexbdcManifest> = toml::from_str(&xdcget_lock)?;

    let new_app_ids = HashSet::<_, RandomState>::from_iter(xdc_metas.keys().cloned());
    let curr_app_ids = HashSet::<_, RandomState>::from_iter(
        db::get_active_app_infos(conn).await?.into_iter().map(|a| a.app_id),
    );
    let removed_app_ids = curr_app_ids.difference(&new_app_ids);

    let mut removed = vec![];
    for app_id in removed_app_ids {
        let app_info = db::get_app_info_for_app_id(conn, app_id).await?;
        db::remove_app(conn, &app_info.app_id).await?;
        fs::remove_file(&app_info.xdc_blob_path).await?;
        removed.push(app_info.xdc_blob_path);
    }

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
                .map(|(index, entry)| {
                    (index, entry.entry().filename().as_str().unwrap_or_default())
                })
                .find(|(_, name)| *name == "icon.png" || *name == "icon.jpg");
            let image = if let Some((index, name)) = image {
                let res = read_vec(&reader, index).await?;
                let mut extension = name
                    .split('.')
                    .nth(1)
                    .context(format!("Can't extract file extension from {name}"))?;
                if extension == "jpg" {
                    extension = "jpeg"
                }
                let base64 = encode(&res);
                format!("data:image/{extension};base64,{base64}")
            } else {
                bail!("Could not find image for {}", path.display())
            };

            Ok(AppInfo {
                id: 0,
                app_id: xdc.app_id,
                tag_name: xdc.tag_name,
                date: OffsetDateTime::parse(&xdc.date, &Rfc3339)?.unix_timestamp(),
                name: xdc.name,
                source_code_url: xdc.source_code_url,
                image,
                description: xdc.description,
                xdc_blob_path: path,
                size,
                removed: false,
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

    for (list, name) in vec![added, updated, ignored, removed]
        .into_iter()
        .zip(&["Added", "Updated", "Ignored", "Removed"])
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

    // Add it to the db
    maybe_upgrade_xdc(&mut app_info, conn, dest).await
}
