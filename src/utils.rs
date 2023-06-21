//! Utility functions

use anyhow::{Context as _, Result};
use async_zip::tokio::read::fs::ZipFileReader;
use deltachat::{
    chat::{self, ChatId},
    config::Config,
    contact::{Contact, ContactId},
    context::Context,
    message::{Message, MsgId, Viewtype},
};
use serde::Serialize;
use sqlx::SqliteConnection;
use std::env;
use tokio::fs;

use crate::{
    db,
    request_handlers::{shop::UpdateResponse, AppInfo},
};

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

pub async fn send_webxdc(
    context: &Context,
    chat_id: ChatId,
    path: &str,
    text: Option<&str>,
) -> anyhow::Result<MsgId> {
    let mut webxdc_msg = Message::new(Viewtype::Webxdc);
    if let Some(text) = text {
        webxdc_msg.set_text(Some(text.to_string()));
    }
    webxdc_msg.set_file(path, None);
    chat::send_msg(context, chat_id, &mut webxdc_msg).await
}

pub async fn send_newest_updates(
    context: &Context,
    msg_id: MsgId,
    db: &mut SqliteConnection,
    serial: i32,
) -> anyhow::Result<()> {
    let app_infos: Vec<_> = db::get_active_app_infos_since(db, serial)
        .await?
        .into_iter()
        .collect();

    let serial = db::get_last_serial(db).await?;
    let resp = UpdateResponse { app_infos, serial };
    send_update_payload_only(context, msg_id, resp).await?;
    Ok(())
}

/// Get the contact Id of the other user in an 1:1 chat.
pub async fn _get_oon_peer(context: &Context, chat_id: ChatId) -> anyhow::Result<ContactId> {
    let contacts = chat::get_chat_contacts(context, chat_id).await?;
    contacts
        .into_iter()
        .find(|contact| !contact.is_special())
        .ok_or(anyhow::anyhow!("No other contact"))
}

pub async fn get_contact_name(context: &Context, contact_id: ContactId) -> String {
    Contact::get_by_id(context, contact_id)
        .await
        .map(|contact| contact.get_name_n_addr())
        .unwrap_or_else(|_| contact_id.to_string())
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

/// Send an app_info to the frontend
pub async fn send_app_info(
    context: &Context,
    app_info: &AppInfo,
    msg_id: MsgId,
) -> anyhow::Result<()> {
    send_update_payload_only(context, msg_id, app_info).await
}

/// Updates a value and update changed accordingly.
pub fn ne_assign<T: PartialEq>(original: &mut T, new: Option<T>, changed: &mut bool) {
    if let Some(new) = new {
        if *original != new {
            *original = new;
            *changed = true;
        }
    }
}

/// Updates a value and update `changed` accordingly.
pub fn ne_assign_option<T: PartialEq>(
    original: &mut Option<T>,
    new: Option<T>,
    changed: &mut bool,
) {
    if let Some(new) = new {
        if let Some(original) = original {
            if *original != new {
                *original = new;
                *changed = true;
            }
        } else {
            *original = Some(new);
            *changed = true;
        }
    }
}

/// Returns the version taken from the `bot-data/VERSION` file.
pub async fn get_version() -> anyhow::Result<String> {
    Ok(fs::read_to_string("bot-data/VERSION").await?)
}

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
