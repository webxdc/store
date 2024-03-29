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
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::SqliteConnection;
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::fs;

use crate::{
    bot::State,
    db,
    messages::store_message,
    request_handlers::{AppInfo, WebxdcManifest, WebxdcStatusUpdatePayload},
};

#[allow(clippy::missing_docs_in_private_items)]
pub(crate) fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "XDC Store").context("cannot determine home directory")
}

/// Configures the bot account according to the environment variables `addr` and `mail_pw`.
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

/// Unpacks the assets built into the bot binary into the configuration directory.
pub(crate) fn unpack_assets() -> Result<()> {
    std::fs::create_dir_all(project_dirs()?.config_dir())?;

    let store_bytes = include_bytes!("../assets/store.xdc");
    let store_path = get_store_xdc_path()?;
    let mut file = File::create(&store_path)
        .with_context(|| format!("failed to create {}", store_path.display()))?;
    file.write_all(store_bytes)?;

    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon_path = get_icon_path()?;
    let mut file = File::create(&icon_path)
        .with_context(|| format!("failed to create {}", icon_path.display()))?;
    file.write_all(icon_bytes)?;
    Ok(())
}

/// Send newest version to chat together with all [AppInfo]s.
pub async fn init_store(context: &Context, state: &State, chat_id: ChatId) -> Result<()> {
    update_store(context, state, chat_id, 0).await?;
    Ok(())
}

/// Send newest store webxdc to a chat together with newest updates.
///
/// `_serial` is the serial number sent by the old frontend
/// in a request for upgrade.
/// It is currently not used because updating the store
/// is essentially sending a new store, so the whole
/// index has to be sent from scratch in any case.
pub async fn update_store(
    context: &Context,
    state: &State,
    chat_id: ChatId,
    _serial: u32,
) -> Result<()> {
    let mut webxdc_msg = Message::new(Viewtype::Webxdc);
    webxdc_msg.set_text(store_message().to_string());
    webxdc_msg.set_file(get_store_xdc_path()?.display(), None);
    chat_id.set_draft(context, Some(&mut webxdc_msg)).await?;

    let conn = &mut *state.db.acquire().await?;
    let serial = 0;
    if serial == 0 {
        let app_infos = db::get_active_app_infos(conn).await?;
        let serial = db::get_last_serial(conn).await?;
        send_update_payload_only(
            context,
            webxdc_msg.get_id(),
            WebxdcStatusUpdatePayload::Init { app_infos, serial },
        )
        .await?;
    } else {
        // Currently unused code path.
        //
        // This will be used when webxdc message is replaced
        // without changing the msg_id, thus preserving old updates.
        send_newest_updates(
            context,
            webxdc_msg.get_id(),
            &mut *state.db.acquire().await?,
            serial,
            vec![],
        )
        .await?;
    }

    db::set_store_tag_name(conn, webxdc_msg.get_id(), &state.store_tag_name).await?;
    chat::send_msg(context, chat_id, &mut webxdc_msg).await?;
    Ok(())
}

#[allow(clippy::missing_docs_in_private_items)]
pub fn to_hashmap<T: Serialize + for<'a> Deserialize<'a>>(
    a: T,
) -> serde_json::Result<HashMap<String, Value>> {
    serde_json::from_value(serde_json::to_value(a)?)
}

/// Sends a [deltachat::webxdc::StatusUpdateItem] with all [AppInfo]s greater than the given serial.
/// `updating` tells the frontend which apps are going to receive an updated.
pub async fn send_newest_updates(
    context: &Context,
    msg_id: MsgId,
    db: &mut SqliteConnection,
    serial: u32,
    updating: Vec<String>,
) -> Result<()> {
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

    let (removed, app_infos) = app_infos
        .into_iter()
        .partition::<Vec<_>, _>(|app_info| app_info.removed);

    let old_app_infos = old_app_infos
        .into_iter()
        .map(|app_info| (app_info.app_id.clone(), app_info))
        .collect::<std::collections::HashMap<_, _>>();

    let changes: Vec<Result<(String, HashMap<String, Value>)>> = app_infos
        .into_iter()
        .map(|app_info| {
            let Some(old_info) = old_app_infos.get(app_info.app_id.as_str()) else {
                return Ok((app_info.app_id.clone(), to_hashmap(app_info)?))
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
                    old_fields.get(key) != Some(val)
                })
                .collect::<HashMap<_, _>>();

            changed_fields.extend(removed_fields);
            Ok((app_info.app_id, changed_fields))
        })
        .collect_vec();

    let mut all_changes = HashMap::new();
    for change in changes {
        let (key, val) = change?;
        all_changes.insert(key, val);
    }

    let mut app_infos = json!(all_changes);
    for removed_app in removed {
        app_infos
            .as_object_mut()
            .context("Problem accessing property")?
            .insert(removed_app.app_id, Value::Null);
    }
    let new_serial = db::get_last_serial(db).await?;
    let resp = WebxdcStatusUpdatePayload::Update {
        app_infos,
        serial: new_serial,
        old_serial: serial,
        updating,
    };
    send_update_payload_only(context, msg_id, resp).await?;
    Ok(())
}

/// Reads the given ZIP file entry into a string.
pub async fn read_string(reader: &ZipFileReader, index: usize) -> Result<String> {
    let mut entry = reader.reader_with_entry(index).await?;
    let mut data = String::new();
    entry.read_to_string_checked(&mut data).await?;
    Ok(data)
}

/// Reads the given ZIP file entry into a byte vector.
pub async fn read_vec(reader: &ZipFileReader, index: usize) -> Result<Vec<u8>> {
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
) -> Result<()> {
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

/// Extracts and parses `manifest.toml` from the .xdc ZIP archive.
pub async fn get_webxdc_manifest(reader: &ZipFileReader) -> Result<WebxdcManifest> {
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

/// Returns the `tag_name` field from the `manifest.toml` of the given `.xdc` file.
pub async fn get_webxdc_tag_name(file: impl AsRef<Path>) -> Result<String> {
    let reader = ZipFileReader::new(file).await?;
    let manifest = get_webxdc_manifest(&reader).await?;
    Ok(manifest.tag_name)
}

#[allow(clippy::missing_docs_in_private_items)]
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
) -> Result<AddType> {
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

/// Returns the file path to the store frontend .xdc file.
pub fn get_store_xdc_path() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf().join("store.xdc"))
}

/// Returns the file path to the store avatar.
pub fn get_icon_path() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf().join("icon.png"))
}
