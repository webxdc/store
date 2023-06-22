//! Bot Database
//! It consists of these tables:
//! - users (Stores reviewers, publishers, genesis members etc.)
//! - app_infos (Stores the app infos)
//! - chats (Stores information about the review and submit chats)
//! - chat_to_chat_type (Acts as a map between ChatId and ChatType)
//! - config (Where config is stored)
//!
//! A chat entry will be created when submitting a webxdc and holds a [SubmitChat].
//! When the app is send to review, it will turn into a [ReviewChat] using the same row but with
//! the review chats chat_id stored in a dedicated field.
//!
//! See migrations folder for further details.

use std::path::PathBuf;

use anyhow::Context;
use deltachat::{chat::ChatId, contact::ContactId, message::MsgId};
use sqlx::{migrate::Migrator, Connection, FromRow, Row, SqliteConnection};

use crate::{
    bot::BotConfig,
    request_handlers::{review::ReviewChat, submit::SubmitChat, AppInfo, ChatType},
};

pub static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(FromRow)]
pub struct DBAppInfo {
    pub id: RecordId,
    pub name: String,                    // manifest
    pub author_name: String,             // bot
    pub author_email: String,            // bot
    pub source_code_url: Option<String>, // manifest
    pub image: Option<String>,           // webxdc
    pub description: Option<String>,     // submit
    pub xdc_blob_dir: Option<String>,    // bot
    pub version: Option<String>,         // manifest
    pub originator: RecordId,            // bot
    pub active: bool,                    // bot
}

impl From<DBAppInfo> for AppInfo {
    fn from(db_app: DBAppInfo) -> Self {
        Self {
            id: db_app.id,
            name: db_app.name,
            author_name: db_app.author_name,
            author_email: db_app.author_email,
            source_code_url: db_app.source_code_url,
            image: db_app.image,
            description: db_app.description,
            xdc_blob_dir: db_app.xdc_blob_dir.map(PathBuf::from),
            version: db_app.version,
            originator: db_app.originator,
            active: db_app.active,
        }
    }
}

#[derive(FromRow)]
struct DBBotConfig {
    pub genesis_qr: String,
    pub invite_qr: String,
    pub tester_group: i32,
    pub reviewee_group: i32,
    pub genesis_group: i32,
    pub serial: i32,
}

impl TryFrom<DBBotConfig> for BotConfig {
    type Error = anyhow::Error;
    fn try_from(db_bot_config: DBBotConfig) -> anyhow::Result<Self> {
        Ok(Self {
            genesis_qr: db_bot_config.genesis_qr,
            invite_qr: db_bot_config.invite_qr,
            tester_group: ChatId::new(u32::try_from(db_bot_config.tester_group)?),
            reviewee_group: ChatId::new(u32::try_from(db_bot_config.reviewee_group)?),
            genesis_group: ChatId::new(u32::try_from(db_bot_config.genesis_group)?),
            serial: db_bot_config.serial,
        })
    }
}

pub type RecordId = i32;

pub async fn set_config(c: &mut SqliteConnection, config: &BotConfig) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO config (genesis_qr, invite_qr, tester_group, reviewee_group, genesis_group, serial) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&config.genesis_qr)
    .bind(&config.invite_qr)
    .bind(config.tester_group.to_u32())
    .bind(config.reviewee_group.to_u32())
    .bind(config.genesis_group.to_u32())
    .bind(config.serial)
    .execute(c).await?;
    Ok(())
}

pub async fn get_config(c: &mut SqliteConnection) -> anyhow::Result<BotConfig> {
    let res: anyhow::Result<BotConfig> = sqlx::query_as::<_, DBBotConfig>(
        "SELECT genesis_qr, invite_qr, tester_group, reviewee_group, genesis_group, serial FROM config",
    )
    .fetch_one(c)
    .await
    .map(|db_bot_config| db_bot_config.try_into())?;
    res
}

pub async fn create_submit_chat(c: &mut SqliteConnection, chat: &SubmitChat) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO chats (submit_chat_id, submit_helper, app_info) VALUES (?, ?, ?)")
        .bind(chat.submit_chat.to_u32())
        .bind(chat.submit_helper.to_u32())
        .bind(chat.app_info)
        .execute(c)
        .await?;
    Ok(())
}

pub async fn delete_submit_chat(c: &mut SqliteConnection, chat_id: ChatId) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM chats WHERE submit_chat_id = ?")
        .bind(chat_id.to_u32())
        .execute(c)
        .await?;
    Ok(())
}

/// Upgrade a submit chat with chat_id `id` to a review chat.
pub async fn upgrade_to_review_chat(
    c: &mut SqliteConnection,
    chat: &ReviewChat,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE chats SET review_helper = ?, review_chat_id = ?, publisher = ?, testers = ? WHERE submit_chat_id = ?"
    )
    .bind(chat.review_helper.to_u32())
    .bind(chat.review_chat.to_u32())
    .bind(chat.publisher.to_u32())
    .bind(serde_json::to_string(&chat.testers)?)
    .bind(chat.submit_chat.to_u32())
    .execute(c).await?;
    Ok(())
}

pub async fn set_review_chat_testers(
    c: &mut SqliteConnection,
    chat_id: ChatId,
    testers: &[ContactId],
) -> anyhow::Result<()> {
    sqlx::query("UPDATE chats SET testers = ? WHERE review_chat_id = ?")
        .bind(serde_json::to_string(testers)?)
        .bind(chat_id.to_u32())
        .execute(c)
        .await?;
    Ok(())
}

pub async fn set_review_chat_publisher(
    c: &mut SqliteConnection,
    chat_id: ChatId,
    publisher: ContactId,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE chats SET publisher = ? WHERE review_chat_id = ?")
        .bind(publisher.to_u32())
        .bind(chat_id.to_u32())
        .execute(c)
        .await?;
    Ok(())
}

#[cfg(test)]
pub async fn create_review_chat(
    c: &mut SqliteConnection,
    review_chat: &ReviewChat,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO chats (review_helper, submit_helper, review_chat_id, submit_chat_id, publisher, testers, app_info) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(review_chat.review_helper.to_u32())
        .bind(review_chat.submit_helper.to_u32())
        .bind(review_chat.review_chat.to_u32())
        .bind(review_chat.submit_chat.to_u32())
        .bind(review_chat.publisher.to_u32())
        .bind(serde_json::to_string(&review_chat.testers)?)
        .bind(review_chat.app_info)
        .execute(c)
        .await?;
    Ok(())
}

pub async fn get_review_chat(
    c: &mut SqliteConnection,
    chat_id: ChatId,
) -> anyhow::Result<ReviewChat> {
    sqlx::query("SELECT review_helper, submit_helper, review_chat_id, submit_chat_id, publisher, testers, app_info FROM chats WHERE review_chat_id = ?")
        .bind(chat_id.to_u32())
        .fetch_one(c)
        .await
        .map(|row| {
            Ok(ReviewChat {
                review_helper: MsgId::new(row.try_get("review_helper")?),
                submit_helper: MsgId::new(row.try_get("submit_helper")?),
                review_chat: ChatId::new(row.try_get("review_chat_id")?),
                submit_chat: ChatId::new(row.try_get("submit_chat_id")?),
                publisher: ContactId::new(row.try_get("publisher")?),
                app_info: row.try_get("app_info")?,
                testers: serde_json::from_str(row.try_get("testers")?)?,
            })
        })?
}

pub async fn get_submit_chat(
    c: &mut SqliteConnection,
    chat_id: ChatId,
) -> anyhow::Result<SubmitChat> {
    sqlx::query(
        "SELECT submit_helper, submit_chat_id, app_info FROM chats WHERE (submit_chat_id = ?)",
    )
    .bind(chat_id.to_u32())
    .fetch_one(c)
    .await
    .map(|row| {
        Ok(SubmitChat {
            submit_helper: MsgId::new(row.try_get("submit_helper")?),
            submit_chat: ChatId::new(row.try_get("submit_chat_id")?),
            app_info: row.try_get("app_info")?,
        })
    })?
}

pub async fn set_chat_type(
    c: &mut SqliteConnection,
    chat_id: ChatId,
    chat_type: ChatType,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO chat_to_chat_type (chat_id, chat_type) VALUES (?, ?)")
        .bind(chat_id.to_u32())
        .bind(chat_type)
        .execute(c)
        .await?;
    Ok(())
}

pub async fn get_chat_type(c: &mut SqliteConnection, chat_id: ChatId) -> sqlx::Result<ChatType> {
    sqlx::query("SELECT chat_type FROM chat_to_chat_type WHERE (chat_id = ?)")
        .bind(chat_id.to_u32())
        .fetch_one(c)
        .await
        .map(|row| row.try_get("chat_type"))?
}

pub async fn add_genesis(c: &mut SqliteConnection, contact_id: ContactId) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO users (genesis, tester, publisher, contact_id) VALUES (true, false, false, ?) ON CONFLICT (contact_id) DO UPDATE SET genesis=true")
        .bind(contact_id.to_u32())
        .execute(c)
        .await?;
    Ok(())
}

pub async fn set_genesis_members(
    c: &mut SqliteConnection,
    contacts: &[ContactId],
) -> anyhow::Result<()> {
    for genesis in contacts {
        add_genesis(c, *genesis).await?;
    }
    Ok(())
}

pub async fn add_publisher(c: &mut SqliteConnection, contact_id: ContactId) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO users (genesis, tester, publisher, contact_id) VALUES (false, false, true, ?) ON CONFLICT (contact_id) DO UPDATE SET publisher=true")
        .bind(contact_id.to_u32())
        .execute(c)
        .await?;
    Ok(())
}

pub async fn set_publishers(
    c: &mut SqliteConnection,
    contacts: &[ContactId],
) -> anyhow::Result<()> {
    for publisher in contacts {
        add_publisher(c, *publisher).await?;
    }
    Ok(())
}

pub async fn get_random_publisher(c: &mut SqliteConnection) -> sqlx::Result<ContactId> {
    sqlx::query("SELECT contact_id FROM users WHERE publisher=true ORDER BY RANDOM() LIMIT 1")
        .fetch_one(c)
        .await
        .map(|row| Ok(ContactId::new(row.get("contact_id"))))?
}

pub async fn get_new_random_publisher(
    c: &mut SqliteConnection,
    old_publisher: ContactId,
) -> sqlx::Result<ContactId> {
    sqlx::query("SELECT contact_id FROM users WHERE publisher=true AND contact_id != ? ORDER BY RANDOM() LIMIT 1")
        .bind(old_publisher.to_u32())
        .fetch_one(c)
        .await
        .map(|row| Ok(ContactId::new(row.get("contact_id"))))?
}

pub async fn add_tester(c: &mut SqliteConnection, contact_id: ContactId) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO users (genesis, tester, publisher, contact_id) VALUES (false, true, false, ?) ON CONFLICT (contact_id) DO UPDATE SET tester=true")
        .bind(contact_id.to_u32())
        .execute(c)
        .await?;
    Ok(())
}

pub async fn set_testers(c: &mut SqliteConnection, contacts: &[ContactId]) -> anyhow::Result<()> {
    for tester in contacts {
        add_tester(c, *tester).await?;
    }
    Ok(())
}

pub async fn get_random_testers(
    c: &mut SqliteConnection,
    count: u32,
) -> anyhow::Result<Vec<ContactId>> {
    sqlx::query("SELECT contact_id FROM users WHERE tester=true ORDER BY RANDOM() LIMIT ?")
        .bind(count)
        .fetch_all(c)
        .await
        .map(|rows| {
            Ok(rows
                .into_iter()
                .map(|row| ContactId::new(row.get("contact_id")))
                .collect())
        })?
}

pub async fn get_random_tester(c: &mut SqliteConnection) -> anyhow::Result<ContactId> {
    sqlx::query("SELECT contact_id FROM users WHERE tester=true ORDER BY RANDOM() LIMIT 1")
        .fetch_one(c)
        .await
        .map(|row| Ok(ContactId::new(row.get("contact_id"))))?
}

pub async fn increase_get_serial(c: &mut SqliteConnection) -> sqlx::Result<u32> {
    let serial: u32 = c
        .transaction(|txn| {
            Box::pin(async move {
                sqlx::query("UPDATE config SET serial = serial + 1")
                    .execute(&mut **txn)
                    .await?;

                sqlx::query("SELECT serial FROM config")
                    .fetch_one(&mut **txn)
                    .await
                    .map(|row| row.get("serial"))
            })
        })
        .await?;
    Ok(serial)
}

pub async fn create_app_info(
    c: &mut SqliteConnection,
    app_info: &mut AppInfo,
) -> anyhow::Result<()> {
    let next_serial = increase_get_serial(c).await?;
    let blob_dir = if let Some(dir) = &app_info.xdc_blob_dir {
        Some(dir.to_str().context("Can't convert to str")?)
    } else {
        None
    };
    let res = sqlx::query("INSERT INTO app_infos (name, description, version, image, author_name, author_email, xdc_blob_dir, active, originator, source_code_url, serial) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(app_info.name.as_str())
        .bind(&app_info.description)
        .bind(&app_info.version)
        .bind(&app_info.image)
        .bind(&app_info.author_name)
        .bind(&app_info.author_email)
        .bind(blob_dir)
        .bind(app_info.active)
        .bind(app_info.originator)
        .bind(&app_info.source_code_url)
        .bind(next_serial)
        .bind(app_info.id)
        .execute(c)
        .await?;
    app_info.id = i32::try_from(res.last_insert_rowid())?;
    Ok(())
}

pub async fn update_app_info(c: &mut SqliteConnection, app_info: &AppInfo) -> anyhow::Result<()> {
    let blob_dir = if let Some(dir) = &app_info.xdc_blob_dir {
        Some(dir.to_str().context("Can't convert to str")?)
    } else {
        None
    };
    sqlx::query("UPDATE app_infos SET name = ?, description = ?, version = ?, image = ?, author_name = ?, author_email = ?, xdc_blob_dir = ?, active = ?, originator = ?, source_code_url = ? WHERE id = ?")
        .bind(app_info.name.as_str())
        .bind(&app_info.description)
        .bind(&app_info.version)
        .bind(&app_info.image)
        .bind(&app_info.author_name)
        .bind(&app_info.author_email)
        .bind(blob_dir)
        .bind(app_info.active)
        .bind(app_info.originator)
        .bind(&app_info.source_code_url)
        .bind(app_info.id)
        .execute(c)
        .await?;
    Ok(())
}

pub async fn publish_app_info(c: &mut SqliteConnection, id: RecordId) -> anyhow::Result<()> {
    sqlx::query("UPDATE app_infos SET active = true WHERE id = ?")
        .bind(id)
        .execute(c)
        .await?;
    Ok(())
}

pub async fn get_app_info(
    c: &mut SqliteConnection,
    resource_id: RecordId,
) -> sqlx::Result<AppInfo> {
    sqlx::query_as::<_, DBAppInfo>("SELECT * FROM app_infos WHERE rowid = ?")
        .bind(resource_id)
        .fetch_one(c)
        .await
        .map(|app| app.into())
}

pub async fn _get_active_app_infos(c: &mut SqliteConnection) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>("SELECT * FROM app_infos WHERE active = true")
        .fetch_all(c)
        .await
        .map(|app| app.into_iter().map(|a| a.into()).collect())
}

pub async fn get_active_app_infos_since(
    c: &mut SqliteConnection,
    serial: i32,
) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>("SELECT * FROM app_infos WHERE active = true AND serial > ?")
        .bind(serial)
        .fetch_all(c)
        .await
        .map(|app| app.into_iter().map(|a| a.into()).collect())
}

pub async fn get_last_serial(c: &mut SqliteConnection) -> sqlx::Result<i32> {
    sqlx::query("SELECT serial FROM config")
        .fetch_one(c)
        .await
        .map(|a| a.get("serial"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn test_create_load_config() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let config = BotConfig {
            genesis_qr: "genesis_qr".to_string(),
            invite_qr: "invite_qr".to_string(),
            tester_group: ChatId::new(1),
            reviewee_group: ChatId::new(1),
            genesis_group: ChatId::new(1),
            serial: 0,
        };
        set_config(&mut conn, &config).await.unwrap();
        let loaded_config = get_config(&mut conn).await.unwrap();
        assert_eq!(config, loaded_config);
    }

    #[tokio::test]
    async fn test_create_load_submit_chat() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let submit_chat_id = ChatId::new(1);
        let submit_chat = SubmitChat {
            submit_chat: submit_chat_id,
            ..Default::default()
        };
        create_submit_chat(&mut conn, &submit_chat).await.unwrap();
        let loaded_submit_chat = get_submit_chat(&mut conn, submit_chat_id).await.unwrap();
        assert_eq!(submit_chat, loaded_submit_chat);
    }

    #[tokio::test]
    async fn test_create_upgrade_get_review_chat() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let submit_chat_id = ChatId::new(1);
        let submit_chat = SubmitChat {
            submit_chat: submit_chat_id,
            ..Default::default()
        };
        create_submit_chat(&mut conn, &submit_chat).await.unwrap();

        let review_chat_id = ChatId::new(2);
        let review_chat = ReviewChat {
            app_info: submit_chat.app_info,
            submit_chat: submit_chat.submit_chat,
            submit_helper: submit_chat.submit_helper,
            review_chat: review_chat_id,
            testers: vec![ContactId::new(3)],
            ..Default::default()
        };

        upgrade_to_review_chat(&mut conn, &review_chat)
            .await
            .unwrap();

        let loaded_review_chat = super::get_review_chat(&mut conn, review_chat_id)
            .await
            .unwrap();
        assert_eq!(loaded_review_chat.testers, review_chat.testers);
    }

    #[tokio::test]
    async fn test_create_get_chattype() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let chat_type = ChatType::Genesis;
        let chat_id = ChatId::new(0);

        set_chat_type(&mut conn, chat_id, chat_type).await.unwrap();
        let loaded_chat_type = get_chat_type(&mut conn, chat_id).await.unwrap();
        assert_eq!(chat_type, loaded_chat_type)
    }

    #[tokio::test]
    async fn test_roles() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let contact_id = ContactId::new(0);
        let contact_id_u32 = contact_id.to_u32();

        add_publisher(&mut conn, contact_id).await.unwrap();
        add_tester(&mut conn, contact_id).await.unwrap();
        add_genesis(&mut conn, contact_id).await.unwrap();

        assert!(
            sqlx::query("SELECT tester, genesis, publisher FROM users WHERE contact_id = ?")
                .bind(contact_id_u32)
                .fetch_one(&mut conn)
                .await
                .map(|row| { [row.get("genesis"), row.get("tester"), row.get("publisher"),] })
                .unwrap()
                .into_iter()
                .reduce(|acc, elem| acc && elem)
                .unwrap()
        );
    }

    #[tokio::test]
    async fn get_random_publisher() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        add_publisher(&mut conn, ContactId::new(0)).await.unwrap();
        add_publisher(&mut conn, ContactId::new(1)).await.unwrap();
        super::get_random_publisher(&mut conn).await.unwrap();
    }

    #[tokio::test]
    async fn set_review_chat_testers() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let chat_id = ChatId::new(0);

        let review_chat = ReviewChat {
            review_chat: chat_id,
            ..Default::default()
        };
        super::create_review_chat(&mut conn, &review_chat)
            .await
            .unwrap();

        super::set_review_chat_testers(
            &mut conn,
            chat_id,
            &[ContactId::new(0), ContactId::new(1), ContactId::new(2)],
        )
        .await
        .unwrap();
        assert_eq!(
            super::get_review_chat(&mut conn, chat_id)
                .await
                .unwrap()
                .testers,
            vec![ContactId::new(0), ContactId::new(1), ContactId::new(2)]
        )
    }

    #[tokio::test]
    async fn get_new_random_publisher() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        add_publisher(&mut conn, ContactId::new(0)).await.unwrap();
        add_publisher(&mut conn, ContactId::new(1)).await.unwrap();
        assert_eq!(
            super::get_new_random_publisher(&mut conn, ContactId::new(0))
                .await
                .unwrap(),
            ContactId::new(1)
        );
    }

    #[tokio::test]
    async fn get_random_tester() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        add_tester(&mut conn, ContactId::new(1)).await.unwrap();
        add_tester(&mut conn, ContactId::new(2)).await.unwrap();
        add_tester(&mut conn, ContactId::new(3)).await.unwrap();

        let testers = super::get_random_testers(&mut conn, 3).await.unwrap();
        assert_eq!(testers.len(), 3);
    }

    #[tokio::test]
    async fn increase_serial() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();
        let serial = increase_get_serial(&mut conn).await.unwrap();
        assert_eq!(serial, 1);
    }

    #[tokio::test]
    async fn app_info_create_get() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        set_config(&mut conn, &BotConfig::default()).await.unwrap();
        let mut app_info = AppInfo::default();
        create_app_info(&mut conn, &mut app_info).await.unwrap();
        let loaded_app_info = get_app_info(&mut conn, app_info.id).await.unwrap();
        assert_eq!(app_info, loaded_app_info);
    }
}
