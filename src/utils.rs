//! Utility functions

use anyhow::{bail, Context as _, Result};
use async_zip::tokio::read::fs::ZipFileReader;
use deltachat::{
    chat::{self, ChatId},
    config::Config,
    contact::{Contact, ContactId},
    context::Context,
    message::{Message, MsgId, Viewtype},
};
use futures::future::join_all;
use itertools::Itertools;
use serde::Serialize;
use sqlx::{SqliteConnection, Type};
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::task::JoinHandle;

use crate::{
    bot::State,
    db,
    request_handlers::{shop::ShopResponse, AppInfo, WexbdcManifest},
    REVIEW_HELPER_XDC, SHOP_XDC, SUBMIT_HELPER_XDC,
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

/// Send a webxdc to a chat.
pub async fn send_webxdc(
    context: &Context,
    state: &State,
    chat_id: ChatId,
    webxdc: Webxdc,
    text: Option<&str>,
) -> anyhow::Result<MsgId> {
    let mut webxdc_msg = Message::new(Viewtype::Webxdc);
    if let Some(text) = text {
        webxdc_msg.set_text(Some(text.to_string()));
    }
    webxdc_msg.set_file(webxdc.get_str_path(), None);
    let msg_id = chat::send_msg(context, chat_id, &mut webxdc_msg).await?;
    let conn = &mut *state.db.acquire().await?;
    let versions = db::get_current_webxdc_versions(conn).await?;
    db::set_webxdc_version(conn, msg_id, versions.get(webxdc).to_string(), webxdc).await?;
    Ok(msg_id)
}

/// Sends a [deltachat::webxdc::StatusUpdateItem] with all [AppInfo]s greater than the given serial.
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
    let resp = ShopResponse::Update { app_infos, serial };
    send_update_payload_only(context, msg_id, resp).await?;
    Ok(())
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

/// Sends an app_info to the frontend
pub async fn send_app_info(
    context: &Context,
    app_info: &AppInfo,
    msg_id: MsgId,
) -> anyhow::Result<()> {
    send_update_payload_only(context, msg_id, app_info).await
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

pub async fn get_webxdc_version(file: impl AsRef<Path>) -> anyhow::Result<String> {
    let reader = ZipFileReader::new(file).await?;
    let manifest = get_webxdc_manifest(&reader).await?;
    Ok(manifest.version)
}

#[derive(Clone, Copy, Type)]
pub enum Webxdc {
    Shop,
    Submit,
    Review,
}

impl Webxdc {
    pub fn get_str_path(&self) -> &'static str {
        match self {
            Webxdc::Shop => SHOP_XDC,
            Webxdc::Submit => SUBMIT_HELPER_XDC,
            Webxdc::Review => REVIEW_HELPER_XDC,
        }
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(self.get_str_path())
    }

    pub fn iter() -> impl Iterator<Item = Webxdc> {
        [Webxdc::Shop, Webxdc::Submit, Webxdc::Review]
            .iter()
            .copied()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WebxdcVersions {
    pub shop: String,
    pub submit: String,
    pub review: String,
}

impl WebxdcVersions {
    pub fn set(&mut self, webxdc: Webxdc, version: String) {
        match webxdc {
            Webxdc::Shop => self.shop = version,
            Webxdc::Submit => self.submit = version,
            Webxdc::Review => self.review = version,
        }
    }

    pub fn get(&self, webxdc: Webxdc) -> &str {
        match webxdc {
            Webxdc::Shop => &self.shop,
            Webxdc::Submit => &self.submit,
            Webxdc::Review => &self.review,
        }
    }
}

pub async fn read_webxdc_versions() -> anyhow::Result<WebxdcVersions> {
    let required_files = Webxdc::iter().map(|webxdc| webxdc.get_path()).collect_vec();

    let required_files_present = required_files
        .iter()
        .all(|path| path.try_exists().unwrap_or_default());

    if !required_files_present {
        bail!("It seems like the frontend hasn't been build yet! Look at the readme for further instructions.")
    }

    let mut futures: Vec<JoinHandle<anyhow::Result<(Webxdc, String)>>> = vec![];
    for webxdc in Webxdc::iter() {
        futures.push(tokio::spawn(async move {
            let version = get_webxdc_version(&webxdc.get_str_path()).await?;
            Ok((webxdc, version))
        }))
    }

    let mut versions = WebxdcVersions {
        shop: String::new(),
        submit: String::new(),
        review: String::new(),
    };
    for result in join_all(futures).await {
        let (webxdc, version) = result??;
        versions.set(webxdc, version);
    }
    Ok(versions)
}
