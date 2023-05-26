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
use std::env;

use crate::DB_PATH;

pub async fn configure_from_env(ctx: &Context) -> Result<()> {
    let addr = env::var("addr")?;
    ctx.set_config(Config::Addr, Some(&addr)).await?;
    let pw = env::var("mail_pw")?;
    ctx.set_config(Config::MailPw, Some(&pw)).await?;
    ctx.set_config(Config::Bot, Some("1")).await?;
    ctx.set_config(Config::E2eeEnabled, Some("1")).await?;
    ctx.configure()
        .await
        .context("configure failed, you might have wrong credentials")?;
    Ok(())
}

pub async fn _get_chat_xdc(context: &Context, chat_id: ChatId) -> anyhow::Result<Option<MsgId>> {
    let mut msg_ids = chat::get_chat_media(
        context,
        Some(chat_id),
        Viewtype::Webxdc,
        Viewtype::Unknown,
        Viewtype::Unknown,
    )
    .await?;
    Ok(msg_ids.pop())
}

pub async fn send_webxdc(context: &Context, chat_id: ChatId, path: &str) -> anyhow::Result<()> {
    let mut webxdc_msg = Message::new(Viewtype::Webxdc);
    webxdc_msg.set_file(path, None);
    chat::send_msg(context, chat_id, &mut webxdc_msg)
        .await
        .unwrap();
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

pub fn get_db_path() -> String {
    env::current_dir()
        .unwrap()
        .join(DB_PATH)
        .to_str()
        .unwrap()
        .to_string()
}
