use anyhow::{bail, Context};
use sqlx::SqliteConnection;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    request_handlers::AppInfo,
    utils::{maybe_upgrade_xdc, AddType},
};

pub async fn import_many(
    path: &Path,
    xdcs_path: PathBuf,
    conn: &mut SqliteConnection,
) -> anyhow::Result<()> {
    let dir_entry = std::fs::read_dir(path).context("Failed to read dir")?;

    // this silently skipps files if some file name can not be converted to str
    let xdcs: Vec<_> = dir_entry
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|e| e.is_file())
        .filter(|file| {
            file.file_name()
                .and_then(|a| a.to_str())
                .map(|a| a.ends_with(".xdc"))
                .unwrap_or_default()
        })
        .collect();

    if xdcs.is_empty() {
        eprintln!("No xdcs to add in {}", path.display());
        return Ok(());
    }

    let mut added = Vec::new();
    let mut updated = Vec::new();
    let mut ignored = Vec::new();
    for file in &xdcs {
        match import_one(file, &xdcs_path, conn).await? {
            AddType::Added => added.push(file),
            AddType::Updated => updated.push(file),
            AddType::Ignored => ignored.push(file),
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

    let mut app_info = AppInfo::from_xdc(file).await?;
    app_info.active = true;
    app_info.submitter_uri = Some("xdcstore".to_string());

    // copy the file to the `dest`
    let mut dest = PathBuf::from(dest);
    dest.push(file.file_name().context("Direntry has no filename")?);
    fs::copy(file, &dest)
        .with_context(|| format!("Failed to copy {} to {}", file.display(), dest.display()))?;
    app_info.xdc_blob_path = dest;

    // Add it to the db
    maybe_upgrade_xdc(&mut app_info, conn).await
}
