//! Utility functions

use std::io::Write;
use std::{collections::HashMap, fs::File};

use anyhow::{Context as _, Result};
use async_zip::tokio::read::fs::ZipFileReader;
use deltachat::{
    chat::{self, ChatId},
    config::Config,
    context::Context,
    message::{Message, MsgId, Viewtype},
};
use directories::ProjectDirs;
use serde::Serialize;
use sqlx::SqliteConnection;
use futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{SqliteConnection, Type};
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::fs;

use crate::{
    bot::State,
    db,
    messages::store_message,
    request_handlers::{AppInfo, WebxdcStatusUpdatePayload, WexbdcManifest},
    STORE_XDC,
};

pub(crate) fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "XDC Store").context("cannot determine home directory")
}

pub async fn configure_from_env(ctx: &Context) -> Result<()> {
    let addr = env::var("addr").context("Missing environment variable addr")?;
    ctx.set_config(Config::Addr, Some(&addr)).await?;
    let pw = env::var("mail_pw").context("Missing environment variable mail_pw")?;
    ctx.set_config(Config::MailPw, Some(&pw)).await?;
    ctx.set_config(Config::Bot, Some("1")).await?;
    ctx.set_config(Config::E2eeEnabled, Some("1")).await?;
    ctx.configure()
        .await
        .context("configure failed, you might have wrong credentials")?;
    Ok(())
}

pub(crate) fn unpack_assets() -> Result<()> {
    let store_bytes = include_bytes!("../assets/store.xdc");

    std::fs::create_dir_all(project_dirs()?.config_dir())?;

    let store_path = get_store_xdc_path()?;
    let mut file = File::create(&store_path)
        .with_context(|| format!("failed to create {}", store_path.display()))?;
    file.write_all(store_bytes)?;
    Ok(())
}

/// Send a webxdc to a chat.
pub async fn send_store_xdc(
    context: &Context,
    state: &State,
    chat_id: ChatId,
    hydrate: bool,
) -> anyhow::Result<MsgId> {
    let mut webxdc_msg = Message::new(Viewtype::Webxdc);
    webxdc_msg.set_text(Some(store_message().to_string()));
    webxdc_msg.set_file(get_store_xdc_path()?.display(), None);
    let msg_id = chat::send_msg(context, chat_id, &mut webxdc_msg).await?;
    if hydrate {
        send_newest_updates(context, msg_id, &mut *state.db.acquire().await?, 0, vec![]).await?;
    }
    let conn = &mut *state.db.acquire().await?;
    db::set_store_tag_name(conn, msg_id, &state.store_tag_name).await?;
    Ok(msg_id)
}

pub fn to_hashmap<T: Serialize + for<'a> Deserialize<'a>>(
    a: T,
) -> serde_json::Result<HashMap<String, Value>> {
    serde_json::from_value(serde_json::to_value(a)?)
}

/// Sends a [deltachat::webxdc::StatusUpdateItem] with all [AppInfo]s greater than the given serial.
/// Updating tells the frontend which apps are going to receive an updated.
pub async fn send_newest_updates(
    context: &Context,
    msg_id: MsgId,
    db: &mut SqliteConnection,
    serial: u32,
    updating: Vec<String>,
) -> anyhow::Result<()> {
    let app_infos: Vec<_> = db::get_changed_app_infos_since(db, serial).await?;
    let old_app_infos = db::get_app_infos_for(
        db,
        &app_infos
            .iter()
            .map(|app_info| app_info.app_id.as_str())
            .collect::<Vec<_>>(),
        serial,
    )
    .await?;
    let old_app_infos = old_app_infos
        .into_iter()
        .map(|app_info| (app_info.app_id.clone(), app_info))
        .collect::<std::collections::HashMap<_, _>>();

    let changes = app_infos
        .into_iter()
        .map(|app_info| {
            let Some(old_info) = old_app_infos.get(app_info.app_id.as_str()) else {
                return to_hashmap(app_info)
            };
            let old_fields = to_hashmap(old_info.clone())?;
            let new_fields = to_hashmap(app_info.clone())?;

            let removed_fields = old_fields
                .iter()
                .filter(|(key, _)| !new_fields.contains_key(*key))
                .map(|(key, _)| (key.to_string(), Value::Null))
                .collect::<HashMap<_, _>>();

            let mut changed_fields = new_fields
                .into_iter()
                .filter(|(key, val)| {
                    if key == "version" || key == "app_id" {
                        return true;
                    }
                    if let Some(old_val) = old_fields.get(key) {
                        old_val != val
                    } else {
                        true
                    }
                })
                .collect::<HashMap<_, _>>();

            changed_fields.extend(removed_fields);
            Ok(changed_fields)
        })
        .collect_vec();

    let mut all_changes = vec![];
    for chang in changes {
        all_changes.push(chang?)
    }

    let serial = db::get_last_serial(db).await?;
    let resp = WebxdcStatusUpdatePayload::Update {
        app_infos: json!(all_changes),
        serial,
        updating,
    };
    send_update_payload_only(context, msg_id, resp).await?;
    Ok(())
}

pub async fn read_string(reader: &ZipFileReader, index: usize) -> anyhow::Result<String> {
    let mut entry = reader.reader_with_entry(index).await?;
    let mut data = String::new();
    entry.read_to_string_checked(&mut data).await?;
    Ok(data)
}

pub async fn read_vec(reader: &ZipFileReader, index: usize) -> anyhow::Result<Vec<u8>> {
    let mut entry = reader.reader_with_entry(index).await?;
    let mut data = Vec::new();
    entry.read_to_end_checked(&mut data).await?;
    Ok(data)
}

/// Sends a [deltachat::webxdc::StatusUpdateItem] with only the given payload.
pub async fn send_update_payload_only<T: Serialize>(
    context: &Context,
    msg_id: MsgId,
    payload: T,
) -> anyhow::Result<()> {
    context
        .send_webxdc_status_update_struct(
            msg_id,
            deltachat::webxdc::StatusUpdateItem {
                payload: serde_json::to_value(payload)?,
                ..Default::default()
            },
            "",
        )
        .await?;
    Ok(())
}

pub async fn get_webxdc_manifest(reader: &ZipFileReader) -> anyhow::Result<WexbdcManifest> {
    let entries = reader.file().entries();
    let manifest_index = entries
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
        .map(|a| a.0)
        .context("Can't find manifest.toml")?;

    Ok(toml::from_str(&read_string(reader, manifest_index).await?)?)
}

pub async fn get_webxdc_tag_name(file: impl AsRef<Path>) -> anyhow::Result<String> {
    let reader = ZipFileReader::new(file).await?;
    let manifest = get_webxdc_manifest(&reader).await?;
    Ok(manifest.tag_name)
}

#[derive(Debug, PartialEq)]
pub enum AddType {
    /// Add a new app_info
    Added,
    /// Update an existing app_info
    Updated,
    /// Ignored
    Ignored,
}

/// If added or updated, moves the file to the `dest`.
pub async fn maybe_upgrade_xdc(
    app_info: &mut AppInfo,
    conn: &mut SqliteConnection,
    dest: &Path,
) -> anyhow::Result<AddType> {
    let add_type = if db::app_tag_name_exists(conn, &app_info.app_id, &app_info.tag_name).await? {
        AddType::Ignored
    } else if db::app_exists(conn, &app_info.app_id).await? {
        AddType::Updated
    } else {
        AddType::Added
    };

    match add_type {
        AddType::Added | AddType::Updated => {
            fs::copy(
                &app_info.xdc_blob_path,
                &dest.join(
                    app_info
                        .xdc_blob_path
                        .file_name()
                        .context("Can't get file name from xdc_blob_dir")?,
                ),
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    app_info.xdc_blob_path.display(),
                    dest.display()
                )
            })?;
            app_info.xdc_blob_path = dest.join(
                app_info
                    .xdc_blob_path
                    .file_name()
                    .context("Can't get file name from xdc_blob_dir")?,
            );
            db::create_app_info(conn, app_info).await?;
        }
        AddType::Ignored => (),
    }
    Ok(add_type)
}

pub fn get_store_xdc_path() -> anyhow::Result<PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf().join(STORE_XDC))
}
