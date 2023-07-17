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

use crate::{
    bot::BotConfig,
    request_handlers::{AppInfo, ChatType},
    utils::Webxdc,
};
use deltachat::{chat::ChatId, contact::ContactId, message::MsgId};
use sqlx::{migrate::Migrator, Connection, FromRow, Row, SqliteConnection};
use std::path::PathBuf;

pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Only a intermediate struct because Decode can not (yet) be derived for [AppInfo].
#[derive(FromRow)]
pub struct DBAppInfo {
    pub id: RecordId,
    pub app_id: String,
    pub name: String,
    pub date: i64,
    pub submitter_uri: Option<String>,
    pub source_code_url: Option<String>,
    pub image: String,
    pub description: String,
    pub xdc_blob_path: String,
    pub size: i64,
    pub version: u32,
}

impl From<DBAppInfo> for AppInfo {
    fn from(db_app: DBAppInfo) -> Self {
        Self {
            id: db_app.id,
            app_id: db_app.app_id,
            name: db_app.name,
            date: db_app.date,
            submitter_uri: db_app.submitter_uri,
            source_code_url: db_app.source_code_url,
            image: db_app.image,
            description: db_app.description,
            xdc_blob_path: PathBuf::from(db_app.xdc_blob_path),
            size: db_app.size,
            version: db_app.version,
        }
    }
}

#[derive(FromRow)]
struct DBBotConfig {
    pub genesis_qr: String,
    pub invite_qr: String,
    pub genesis_group: i32,
    pub serial: i32,
    pub store_xdc_version: String,
}

impl TryFrom<DBBotConfig> for BotConfig {
    type Error = anyhow::Error;
    fn try_from(db_bot_config: DBBotConfig) -> anyhow::Result<Self> {
        Ok(Self {
            genesis_qr: db_bot_config.genesis_qr,
            invite_qr: db_bot_config.invite_qr,
            genesis_group: ChatId::new(u32::try_from(db_bot_config.genesis_group)?),
            serial: db_bot_config.serial,
            store_xdc_version: db_bot_config.store_xdc_version,
        })
    }
}

pub type RecordId = i32;

pub async fn set_config(c: &mut SqliteConnection, config: &BotConfig) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO config (genesis_qr, invite_qr, genesis_group, serial, store_xdc_version) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&config.genesis_qr)
    .bind(&config.invite_qr)
    .bind(config.genesis_group.to_u32())
    .bind(config.serial)
    .bind(&config.store_xdc_version)
    .execute(c).await?;
    Ok(())
}

pub async fn get_config(c: &mut SqliteConnection) -> anyhow::Result<BotConfig> {
    let res: anyhow::Result<BotConfig> = sqlx::query_as::<_, DBBotConfig>(
        "SELECT genesis_qr, invite_qr, genesis_group, serial, store_xdc_version FROM config",
    )
    .fetch_one(c)
    .await
    .map(|db_bot_config| db_bot_config.try_into())?;
    res
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
    let mut trans = c.begin().await?;
    let next_serial = increase_get_serial(&mut trans).await?;
    let res = sqlx::query("INSERT INTO app_infos (app_id, name, description, version, image, submitter_uri, xdc_blob_path, source_code_url, serial, date, size) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(app_info.app_id.as_str())
        .bind(app_info.name.as_str())
        .bind(&app_info.description)
        .bind(app_info.version)
        .bind(&app_info.image)
        .bind(&app_info.submitter_uri)
        .bind(app_info.xdc_blob_path.to_str())
        .bind(&app_info.source_code_url)
        .bind(next_serial)
        .bind(app_info.date)
        .bind(app_info.size)
        .execute(&mut *trans)
        .await?;
    app_info.id = i32::try_from(res.last_insert_rowid())?;
    trans.commit().await?;
    Ok(())
}

/// Get app_info by app_id.
pub async fn get_app_info_for_app_id(
    c: &mut SqliteConnection,
    app_id: &str,
) -> sqlx::Result<AppInfo> {
    sqlx::query_as::<_, DBAppInfo>(
        "SELECT * FROM app_infos WHERE app_id = ? ORDER BY version DESC LIMIT 1;",
    )
    .bind(app_id)
    .fetch_one(c)
    .await
    .map(|app| app.into())
}

/// Get app_info with greate version.
pub async fn maybe_get_greater_version(
    c: &mut SqliteConnection,
    app_id: &str,
    version: u32,
) -> sqlx::Result<bool> {
    sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM app_infos WHERE app_id = ? AND version > ? LIMIT 1) AS exists_greater_version",
    )
    .bind(app_id)
    .bind(version)
    .fetch_one(c)
    .await
    .map(|app| app.get(0))
}

#[cfg(test)]
pub async fn get_app_infos(c: &mut SqliteConnection) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>("SELECT * FROM app_infos")
        .fetch_all(c)
        .await
        .map(|app| app.into_iter().map(|a| a.into()).collect())
}

pub async fn get_active_app_infos_since(
    c: &mut SqliteConnection,
    serial: u32,
) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>(
        r#"SELECT a.*
FROM app_infos a
JOIN (
    SELECT app_id, MAX(version) AS latest_version
    FROM app_infos
    GROUP BY app_id
) b ON a.app_id = b.app_id AND a.version = b.latest_version
WHERE a.serial > ?"#,
    )
    .bind(serial)
    .fetch_all(c)
    .await
    .map(|app| app.into_iter().map(|a| a.into()).collect())
}

pub async fn app_exists(c: &mut SqliteConnection, app_id: &str) -> sqlx::Result<bool> {
    sqlx::query("SELECT EXISTS(SELECT 1 FROM app_infos WHERE app_id = ?)")
        .bind(app_id)
        .fetch_one(c)
        .await
        .map(|row| row.get(0))
}

/// Returns wheter an [AppInfo] with given version exists for the app.
pub async fn app_version_exists(
    c: &mut SqliteConnection,
    id: &str,
    version: u32,
) -> sqlx::Result<bool> {
    sqlx::query("SELECT EXISTS(SELECT 1 FROM app_infos WHERE app_id = ? AND version = ?)")
        .bind(id)
        .bind(version)
        .fetch_one(c)
        .await
        .map(|row| row.get(0))
}

pub async fn get_last_serial(c: &mut SqliteConnection) -> sqlx::Result<i32> {
    sqlx::query("SELECT serial FROM config")
        .fetch_one(c)
        .await
        .map(|a| a.get("serial"))
}

/// Sets the webxdc version for some sent webxdc.
pub async fn set_webxdc_version(
    c: &mut SqliteConnection,
    msg: MsgId,
    version: u32,
    webxdc: Webxdc,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO webxdc_versions (msg_id, version, webxdc) VALUES (?, ?, ?)",
    )
    .bind(msg.to_u32())
    .bind(version)
    .bind(webxdc)
    .execute(c)
    .await?;
    Ok(())
}

/// Gets the webxdc version for some sent webxdc.
pub async fn get_webxdc_version(
    c: &mut SqliteConnection,
    msg: MsgId,
) -> sqlx::Result<(Webxdc, u32)> {
    sqlx::query("SELECT * FROM webxdc_versions WHERE msg_id = ?")
        .bind(msg.to_u32())
        .fetch_one(c)
        .await
        .map(|a| (a.get("webxdc"), a.get("version")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::utils::AddType;
    use sqlx::{Connection, SqliteConnection};
    use std::vec;

    #[tokio::test]
    async fn test_create_load_config() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let config = BotConfig {
            genesis_qr: "genesis_qr".to_string(),
            invite_qr: "invite_qr".to_string(),
            genesis_group: ChatId::new(1),
            serial: 0,
            store_xdc_version: "1.1.0".to_string(),
        };
        set_config(&mut conn, &config).await.unwrap();
        let loaded_config = get_config(&mut conn).await.unwrap();
        assert_eq!(config, loaded_config);
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
    async fn increase_serial() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();
        let serial = increase_get_serial(&mut conn).await.unwrap();
        assert_eq!(serial, 1);
    }

    #[tokio::test]
    async fn sent_webxdc_version_set_get() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let msg = MsgId::new(1);
        set_webxdc_version(&mut conn, msg, 1, Webxdc::Store)
            .await
            .unwrap();
        let (_, loaded_version) = get_webxdc_version(&mut conn, msg).await.unwrap();
        assert_eq!(loaded_version, 1);
    }

    #[tokio::test]
    async fn test_create_get_app_info() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();

        let mut app_info = AppInfo {
            size: 998887,
            date: 1688835984521,
            app_id: "app_id".to_string(),
            id: 12,
            version: 9,
            name: "Sebastians coole app".to_string(),
            submitter_uri: Some("https://example.com".to_string()),
            source_code_url: Some("https://git.example.com/sebastian/app".to_string()),
            image: "aaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            description: "This is a cool app".to_string(),
            xdc_blob_path: PathBuf::from("xdc_blob_path"),
        };

        create_app_info(&mut conn, &mut app_info).await.unwrap();

        let loaded_app_info = get_app_info_for_app_id(&mut conn, &app_info.app_id)
            .await
            .unwrap();

        assert_eq!(app_info, loaded_app_info);
    }

    #[tokio::test]
    async fn upgrade_app() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();

        let mut app_info = AppInfo {
            app_id: "testxdc".to_string(),
            version: 1,
            ..Default::default()
        };

        super::create_app_info(&mut conn, &mut app_info)
            .await
            .unwrap();

        assert_eq!(super::get_app_infos(&mut conn).await.unwrap().len(), 1);

        let mut new_app_info = AppInfo {
            version: 2,
            ..app_info.clone()
        };

        let state =
            crate::utils::maybe_upgrade_xdc(&mut new_app_info, &mut conn, &PathBuf::from(""))
                .await
                .unwrap();

        assert_eq!(state, AddType::Updated);
        assert_eq!(
            super::get_active_app_infos_since(&mut conn, 1)
                .await
                .unwrap(),
            vec![new_app_info.clone()]
        );

        let state =
            crate::utils::maybe_upgrade_xdc(&mut new_app_info, &mut conn, &PathBuf::from(""))
                .await
                .unwrap();

        assert_eq!(state, AddType::Ignored);
    }

    #[tokio::test]
    async fn test_app_exists() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();

        let mut app_info = AppInfo {
            app_id: "testxdc".to_string(),
            ..Default::default()
        };

        super::create_app_info(&mut conn, &mut app_info)
            .await
            .unwrap();

        assert!(super::app_exists(&mut conn, "testxdc").await.unwrap());
        assert!(!super::app_exists(&mut conn, "testxdc2").await.unwrap());
    }

    #[tokio::test]
    async fn test_maybe_get_greater() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();

        let mut app_info = AppInfo {
            app_id: "testxdc".to_string(),
            ..Default::default()
        };

        crate::utils::maybe_upgrade_xdc(&mut app_info, &mut conn, &PathBuf::from(""))
            .await
            .unwrap();

        assert!(
            !maybe_get_greater_version(&mut conn, &app_info.app_id, app_info.version)
                .await
                .unwrap()
        );

        crate::utils::maybe_upgrade_xdc(
            &mut AppInfo {
                version: 2,
                app_id: "testxdc".to_string(),
                ..app_info.clone()
            },
            &mut conn,
            &PathBuf::from(""),
        )
        .await
        .unwrap();

        assert!(
            maybe_get_greater_version(&mut conn, &app_info.app_id, app_info.version)
                .await
                .unwrap()
        );
    }
}
