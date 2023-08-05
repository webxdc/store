//! Bot Database
//! It consists of these tables:
//! - app_infos (Stores the app infos)
//! - config (Where config is stored)
//!
//! See migrations folder for further details.

use crate::{bot::BotConfig, request_handlers::AppInfo};
use anyhow::Result;
use deltachat::message::MsgId;
use itertools::Itertools;
use sqlx::{migrate::Migrator, Connection, FromRow, Row, SqliteConnection};
use std::path::PathBuf;

#[allow(clippy::missing_docs_in_private_items)]
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Only a intermediate struct because Decode can not (yet) be derived for [AppInfo].
#[derive(FromRow)]
pub struct DBAppInfo {
    #[allow(clippy::missing_docs_in_private_items)]
    pub id: RecordId,

    /// Application ID, e.g. `webxdc-poll`.
    pub app_id: String,

    /// Application name, e.g. `Checklist`.
    pub name: String,

    /// Date as a timestamp in seconds.
    pub date: i64,

    /// Source code URL, e.g. `https://codeberg.org/webxdc/checklist`.
    pub source_code_url: String,

    /// Application icon encoded as a data URL,
    /// for example `data:image/png;base64,...`.
    pub image: String,

    /// Human-readable application description.
    pub description: String,

    /// Absolute path to the .xdc file.
    pub xdc_blob_path: String,

    /// Application size in bytes.
    pub size: i64,

    /// Release tag, e.g. `v2.2.0`.
    pub tag_name: String,

    /// True if the application has been removed.
    pub removed: bool,
}

impl From<DBAppInfo> for AppInfo {
    fn from(db_app: DBAppInfo) -> Self {
        Self {
            id: db_app.id,
            app_id: db_app.app_id,
            name: db_app.name,
            date: db_app.date,
            source_code_url: db_app.source_code_url,
            image: db_app.image,
            description: db_app.description,
            xdc_blob_path: PathBuf::from(db_app.xdc_blob_path),
            size: db_app.size,
            tag_name: db_app.tag_name,
            removed: db_app.removed,
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
pub type RecordId = i32;

/// Stores the bot configuration into the `config` table of the bot database.
pub async fn set_config(c: &mut SqliteConnection, config: &BotConfig) -> Result<()> {
    sqlx::query("INSERT INTO config (invite_qr, serial) VALUES (?, ?)")
        .bind(&config.invite_qr)
        .bind(config.serial)
        .execute(c)
        .await?;
    Ok(())
}

/// Retrieves the bot configuration from the database.
pub async fn get_config(c: &mut SqliteConnection) -> Result<BotConfig> {
    let res: BotConfig = sqlx::query_as::<_, BotConfig>("SELECT invite_qr, serial FROM config")
        .fetch_one(c)
        .await?;
    Ok(res)
}

/// Returns the latest store serial.
pub async fn get_last_serial(c: &mut SqliteConnection) -> sqlx::Result<u32> {
    sqlx::query("SELECT serial FROM config")
        .fetch_one(c)
        .await
        .map(|a| a.get("serial"))
}

/// Increase serial by one and return the new serial.
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

/// Create [AppInfo].
pub async fn create_app_info(c: &mut SqliteConnection, app_info: &mut AppInfo) -> Result<()> {
    let mut trans = c.begin().await?;
    let next_serial = increase_get_serial(&mut trans).await?;
    let res = sqlx::query("INSERT INTO app_infos (app_id, name, description, tag_name, image, xdc_blob_path, source_code_url, serial, date, size) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(app_info.app_id.as_str())
        .bind(app_info.name.as_str())
        .bind(&app_info.description)
        .bind(&app_info.tag_name)
        .bind(&app_info.image)
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

/// Get [AppInfo] by app_id.
pub async fn get_app_info_for_app_id(
    c: &mut SqliteConnection,
    app_id: &str,
) -> sqlx::Result<AppInfo> {
    sqlx::query_as::<_, DBAppInfo>(
        "SELECT * FROM app_infos WHERE app_id = ? ORDER BY serial DESC LIMIT 1;",
    )
    .bind(app_id)
    .fetch_one(c)
    .await
    .map(|app| app.into())
}

/// Returns wheter an [AppInfo] with greater tag_name exists.
pub async fn maybe_get_greater_tag_name(
    c: &mut SqliteConnection,
    app_id: &str,
    tag_name: &str,
) -> sqlx::Result<bool> {
    sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM app_infos WHERE app_id = ? AND tag_name > ? AND removed = 0 LIMIT 1) AS exists_greater_tag_name",
    )
    .bind(app_id)
    .bind(tag_name)
    .fetch_one(c)
    .await
    .map(|app| app.get(0))
}

#[cfg(test)]
/// Return all [AppInfo]s.
pub async fn get_app_infos(c: &mut SqliteConnection) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>("SELECT * FROM app_infos")
        .fetch_all(c)
        .await
        .map(|app| app.into_iter().map(|a| a.into()).collect())
}

/// Returns the newest AppInfo for each app.
pub async fn get_active_app_infos(c: &mut SqliteConnection) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>(
        r#"SELECT a.*
    FROM app_infos a
    JOIN (
        SELECT app_id, MAX(serial) AS latest_serial
        FROM app_infos
        GROUP BY app_id
    ) b ON a.app_id = b.app_id AND a.serial = b.latest_serial"#,
    )
    .fetch_all(c)
    .await
    .map(|app| app.into_iter().map(|a| a.into()).collect())
}

/// Get the newest [AppInfo]s with a serial greater than serial.
/// Gets the latest versions of all changed apps.
pub async fn get_changed_app_infos_since(
    c: &mut SqliteConnection,
    serial: u32,
) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query_as::<_, DBAppInfo>(
        r#"SELECT a.*
FROM app_infos a
JOIN (
    SELECT app_id, MAX(serial) AS latest_serial
    FROM app_infos
    GROUP BY app_id
) b ON a.app_id = b.app_id AND a.serial = b.latest_serial
WHERE a.serial > ?"#,
    )
    .bind(serial)
    .fetch_all(c)
    .await
    .map(|app| app.into_iter().map(|a| a.into()).collect())
}

/// This function takes a list of app_ids's and returns the latest version of each app where serial <= serial.
pub async fn get_app_infos_for(
    c: &mut SqliteConnection,
    apps: &[&str],
    serial: u32,
) -> sqlx::Result<Vec<AppInfo>> {
    #[allow(unstable_name_collisions)]
    let list = apps
        .iter()
        .map(|app| app.replace('\'', "''"))
        .map(|app| format!("'{}'", app))
        .intersperse(",".to_string())
        .collect::<String>();
    sqlx::query_as::<_, DBAppInfo>(&format!(
        r#"
    SELECT a.*
    FROM app_infos a
    WHERE app_id IN ({list}) 
        AND serial <= ?
        AND serial = (
            SELECT MAX(serial)
            FROM app_infos
            WHERE app_id = a.app_id
              AND serial <= ?
        )
    "#
    ))
    .bind(serial)
    .bind(serial)
    .fetch_all(c)
    .await
    .map(|app| app.into_iter().map(|a| a.into()).collect())
}

/// Returns whether an [AppInfo] for given app_id exists.
pub async fn app_exists(c: &mut SqliteConnection, app_id: &str) -> sqlx::Result<bool> {
    sqlx::query("SELECT EXISTS(SELECT 1 FROM app_infos WHERE app_id = ? AND removed = 0)")
        .bind(app_id)
        .fetch_one(c)
        .await
        .map(|row| row.get(0))
}

/// Returns wheter an [AppInfo] with given tag_name exists for the given app_id.
pub async fn app_tag_name_exists(
    c: &mut SqliteConnection,
    app_id: &str,
    tag_name: &str,
) -> sqlx::Result<bool> {
    sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM app_infos WHERE app_id = ? AND tag_name = ? AND removed = 0)",
    )
    .bind(app_id)
    .bind(tag_name)
    .fetch_one(c)
    .await
    .map(|row| row.get(0))
}

/// Sets the webxdc tag_name for some sent webxdc.
pub async fn set_store_tag_name(
    c: &mut SqliteConnection,
    msg: MsgId,
    tag_name: &str,
) -> sqlx::Result<()> {
    sqlx::query("INSERT OR REPLACE INTO webxdc_tag_names (msg_id, tag_name) VALUES (?, ?)")
        .bind(msg.to_u32())
        .bind(tag_name)
        .execute(c)
        .await?;
    Ok(())
}

/// Returns the webxdc `tag_name` for some previously sent `store.xdc` instance.
pub async fn get_store_tag_name(c: &mut SqliteConnection, msg: MsgId) -> sqlx::Result<String> {
    sqlx::query("SELECT * FROM webxdc_tag_names WHERE msg_id = ?")
        .bind(msg.to_u32())
        .fetch_one(c)
        .await
        .map(|a| (a.get("tag_name")))
}

/// Removes app with app_id from store.
pub async fn remove_app(c: &mut SqliteConnection, app_id: &str) -> sqlx::Result<()> {
    let mut t = c.begin().await?;
    let next_serial = increase_get_serial(&mut t).await?;
    sqlx::query("UPDATE app_infos SET removed = 1, serial = ? WHERE app_id = ? AND serial = (SELECT MAX(serial) FROM app_infos WHERE app_id = ?);")
        .bind(next_serial)
        .bind(app_id)
        .bind(app_id)
        .execute(&mut *t)
        .await?;
    t.commit().await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::utils::AddType;
    use sqlx::{Connection, SqliteConnection};
    use std::{env, fs::create_dir, vec};

    #[tokio::test]
    async fn test_create_load_config() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let config = BotConfig {
            invite_qr: "invite_qr".to_string(),
            serial: 0,
        };
        set_config(&mut conn, &config).await.unwrap();
        let loaded_config = get_config(&mut conn).await.unwrap();
        assert_eq!(config, loaded_config);
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
    async fn store_tag_name_set_get() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();

        let msg = MsgId::new(1);
        set_store_tag_name(&mut conn, msg, "v1.2.1").await.unwrap();
        let loaded_tag_name = get_store_tag_name(&mut conn, msg).await.unwrap();
        assert_eq!(loaded_tag_name, "v1.2.1".to_string());
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
            tag_name: "v1.2.1".to_string(),
            name: "Sebastians coole app".to_string(),
            source_code_url: "https://git.example.com/sebastian/app".to_string(),
            image: "aaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            description: "This is a cool app".to_string(),
            xdc_blob_path: PathBuf::from("example-xdcs/webxdc-2048-v1.2.1.xdc"),
            removed: false,
        };

        create_app_info(&mut conn, &mut app_info).await.unwrap();

        let loaded_app_info = get_app_info_for_app_id(&mut conn, &app_info.app_id)
            .await
            .unwrap();

        assert_eq!(app_info, loaded_app_info);

        app_info.app_id = "test2".to_string();
        let dest = env::temp_dir().join("example-xdcs");
        create_dir(&dest).ok();
        let add_type = crate::utils::maybe_upgrade_xdc(&mut app_info, &mut conn, &dest)
            .await
            .unwrap();

        assert_eq!(add_type, crate::utils::AddType::Added);

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
        let dest = env::temp_dir().join("example-xdcs");
        create_dir(&dest).ok();

        let mut app_info = AppInfo {
            app_id: "testxdc".to_string(),
            xdc_blob_path: PathBuf::from("example-xdcs/webxdc-2048-v1.2.1.xdc"),
            tag_name: "v1.2.1".to_string(),
            ..Default::default()
        };

        super::create_app_info(&mut conn, &mut app_info)
            .await
            .unwrap();

        assert_eq!(super::get_app_infos(&mut conn).await.unwrap().len(), 1);

        let mut new_app_info = AppInfo {
            tag_name: "v1.2.2".to_string(),
            ..app_info.clone()
        };

        let state = crate::utils::maybe_upgrade_xdc(&mut new_app_info, &mut conn, &dest)
            .await
            .unwrap();

        assert_eq!(state, AddType::Updated);
        assert_eq!(
            super::get_changed_app_infos_since(&mut conn, 1)
                .await
                .unwrap(),
            vec![new_app_info.clone()]
        );

        let state = crate::utils::maybe_upgrade_xdc(&mut new_app_info, &mut conn, &dest)
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

        let dest = env::temp_dir().join("example-xdcs");
        create_dir(&dest).ok();

        let mut app_info = AppInfo {
            app_id: "testxdc".to_string(),
            xdc_blob_path: PathBuf::from("example-xdcs/webxdc-2048-v1.2.1.xdc"),
            ..Default::default()
        };

        crate::utils::maybe_upgrade_xdc(
            &mut app_info,
            &mut conn,
            &env::temp_dir().join("example-xdcs"),
        )
        .await
        .unwrap();

        // test that file has been moved
        assert!(dest.join("webxdc-2048-v1.2.1.xdc").exists());

        assert!(
            !maybe_get_greater_tag_name(&mut conn, &app_info.app_id, &app_info.tag_name)
                .await
                .unwrap()
        );

        crate::utils::maybe_upgrade_xdc(
            &mut AppInfo {
                tag_name: "v1.2.1".to_string(),
                app_id: "testxdc".to_string(),
                ..app_info.clone()
            },
            &mut conn,
            &dest,
        )
        .await
        .unwrap();

        assert!(
            maybe_get_greater_tag_name(&mut conn, &app_info.app_id, &app_info.tag_name)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_app_remove() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&mut conn).await.unwrap();
        set_config(&mut conn, &BotConfig::default()).await.unwrap();

        let mut app_info = AppInfo {
            app_id: "testxdc".to_string(),
            tag_name: "v0.0.1".to_string(),
            ..Default::default()
        };

        super::create_app_info(&mut conn, &mut app_info)
            .await
            .unwrap();
        app_info.tag_name = "v0.0.3".to_string();
        super::create_app_info(&mut conn, &mut app_info)
            .await
            .unwrap();

        app_info.tag_name = "v0.0.10".to_string();
        super::create_app_info(&mut conn, &mut app_info)
            .await
            .unwrap();

        let serial = super::get_last_serial(&mut conn).await.unwrap();
        super::remove_app(&mut conn, &app_info.app_id)
            .await
            .unwrap();

        let loaded_app_info = get_app_info_for_app_id(&mut conn, &app_info.app_id)
            .await
            .unwrap();

        assert!(loaded_app_info.removed);
        assert_eq!(loaded_app_info.tag_name, "v0.0.10".to_string());

        let changed = super::get_changed_app_infos_since(&mut conn, serial)
            .await
            .unwrap();

        assert_eq!(changed[0].tag_name, "v0.0.10".to_string());
        assert!(changed[0].removed)
    }
}
