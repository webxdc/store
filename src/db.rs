//! Bot Database
//! It consists of these tables:
//! - Users (Stores reviewers, publishers, genesis members etc.)
//! - AppInfos (Stores the app infos)
//! - Chats (Stores information about the review and submit chats)
//! - ChatTypes (Acts as a map between ChatId and ChatType)
//!
//! A chat entry will be created when submitting a webxdc and holds a [SubmitChat].
//! When the app is send to review, it will turn into a [ReviewChat].

use deltachat::{chat::ChatId, contact::ContactId};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    bot::BotConfig,
    request_handlers::{review::ReviewChat, submit::SubmitChat, AppInfo, ChatType},
};

#[derive(Deserialize, Serialize, Clone, Debug, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/")]
pub struct FrontendAppInfo {
    pub name: String,                    // manifest
    pub author_name: String,             // bot
    pub author_email: String,            // bot
    pub source_code_url: Option<String>, // manifest
    pub image: Option<String>,           // webxdc
    pub description: Option<String>,     // submit
    pub version: Option<String>,         // manifest
    pub id: String,
}

pub type RecordId = u64;

pub async fn set_config(c: Connection, config: &BotConfig) -> anyhow::Result<BotConfig> {
    todo!()
}

pub async fn get_config(c: Connection) -> anyhow::Result<BotConfig> {
    todo!()
}

pub async fn get_review_chat(c: Connection, chat_id: ChatId) -> anyhow::Result<ReviewChat> {
    todo!()
}

pub async fn get_submit_chat(c: Connection, chat_id: ChatId) -> anyhow::Result<Option<SubmitChat>> {
    todo!()
}

pub async fn create_submit_chat(c: Connection, chat: &SubmitChat) -> anyhow::Result<()> {
    todo!();
}

pub async fn upgrade_to_review_chat(c: Connection, chat: &ReviewChat) -> anyhow::Result<()> {
    todo!();
}

pub async fn set_chat_type(
    c: Connection,
    chat_id: ChatId,
    chat_type: ChatType,
) -> anyhow::Result<()> {
    todo!();
}

pub async fn get_chat_type(c: Connection, chat_id: ChatId) -> anyhow::Result<ChatType> {
    todo!();
}

pub async fn add_genesis(c: Connection, contact_id: ContactId) -> anyhow::Result<()> {
    todo!();
}

pub async fn set_genesis_contacts(c: Connection, contacts: &[ContactId]) -> anyhow::Result<()> {
    todo!();
}

pub async fn add_publisher(c: Connection, contact_id: ContactId) -> anyhow::Result<()> {
    todo!();
}

pub async fn set_publishers(c: Connection, contacts: &[ContactId]) -> anyhow::Result<()> {
    todo!();
}

pub async fn get_random_publisher(c: Connection) -> anyhow::Result<ContactId> {
    todo!()
}

pub async fn add_tester(c: Connection, contact_id: ContactId) -> anyhow::Result<()> {
    todo!()
}

pub async fn set_testers(c: Connection, contacts: &[ContactId]) -> anyhow::Result<()> {
    todo!()
}

pub async fn get_random_testers(c: Connection) -> anyhow::Result<Vec<ContactId>> {
    todo!()
}

pub async fn increase_get_serial(c: Connection) -> anyhow::Result<usize> {
    todo!()
}

// TODO: take string as resource id
// TOOD: don't take resource id from
pub async fn create_app_info(c: Connection, app_info: &AppInfo) -> anyhow::Result<AppInfo> {
    //let next_serial = self.increase_get_serial().await?;
    todo!()
}

pub async fn update_app_info(c: Connection, app_info: &AppInfo) -> anyhow::Result<AppInfo> {
    todo!()
}

pub async fn publish_app_info(c: Connection, id: RecordId) -> anyhow::Result<AppInfo> {
    todo!()
}

pub async fn get_app_info(c: Connection, resource_id: RecordId) -> anyhow::Result<AppInfo> {
    todo!()
}

pub async fn get_active_app_infos(c: Connection) -> anyhow::Result<Vec<AppInfo>> {
    todo!()
}

pub async fn get_active_app_infos_since(
    c: Connection,
    serial: usize,
) -> anyhow::Result<Vec<AppInfo>> {
    todo!()
}

pub async fn get_last_serial(c: Connection) -> anyhow::Result<usize> {
    todo!()
}
