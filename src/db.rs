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

use deltachat::{chat::ChatId, contact::ContactId, message::MsgId};
use sqlx::{migrate::Migrator, Connection, Row, SqliteConnection};

use crate::{
    bot::BotConfig,
    request_handlers::{review::ReviewChat, submit::SubmitChat, AppInfo, ChatType},
};

pub static MIGRATOR: Migrator = sqlx::migrate!();

pub type RecordId = i64;

pub async fn set_config(c: &mut SqliteConnection, config: &BotConfig) -> anyhow::Result<()> {
    let tester_group = config.tester_group.to_u32();
    let reviewee_group = config.reviewee_group.to_u32();
    let genesis_group = config.genesis_group.to_u32();
    let serial = i64::try_from(config.serial).unwrap().to_string();
    sqlx::query!(
        "INSERT INTO config (genesis_qr, invite_qr, tester_group, reviewee_group, genesis_group, serial) VALUES (?, ?, ?, ?, ?, ?)",
        config.genesis_qr,
        config.invite_qr,
        tester_group,
        reviewee_group,
        genesis_group,
        serial
    ).execute(c).await?;
    Ok(())
}

pub async fn get_config(c: &mut SqliteConnection) -> sqlx::Result<BotConfig> {
    sqlx::query_as!(
        BotConfig,
        r#"SELECT genesis_qr, invite_qr, tester_group as "tester_group: u32", reviewee_group as 
        "reviewee_group: u32", genesis_group as "genesis_group: u32", serial as "serial: u32" 
        FROM config"#
    )
    .fetch_one(c)
    .await
}

pub async fn create_submit_chat(c: &mut SqliteConnection, chat: &SubmitChat) -> sqlx::Result<()> {
    let submit_helper = chat.submit_helper.to_u32();
    let submit_chat_id = chat.submit_chat.to_u32();
    sqlx::query!(
        "INSERT INTO chats (submit_chat_id, submit_helper, app_info) VALUES (?, ?, ?)",
        submit_chat_id,
        submit_helper,
        chat.app_info
    )
    .execute(c)
    .await?;
    Ok(())
}

/// Upgrade a submit chat with chat_id `id` to a review chat
pub async fn upgrade_to_review_chat(
    c: &mut SqliteConnection,
    chat: &ReviewChat,
) -> anyhow::Result<()> {
    let review_helper = chat.review_helper.to_u32();
    let submit_helper = chat.submit_helper.to_u32();
    let creator_chat = chat.submit_chat.to_u32();
    let publisher = chat.publisher.to_u32();
    let submit_chat_id = chat.submit_chat.to_u32();
    let review_chat_id = chat.review_chat.to_u32();

    sqlx::query!(
        "UPDATE chats SET review_helper = ?, submit_helper = ?, review_chat_id = ?, submit_chat_id = ?, publisher = ?, app_info = ? WHERE submit_chat_id = ?",
        review_helper,
        submit_helper,
        review_chat_id,
        creator_chat,
        publisher,
        chat.app_info,
        submit_chat_id
    ).execute(c).await?;
    Ok(())
}

pub async fn get_review_chat(
    c: &mut SqliteConnection,
    chat_id: ChatId,
) -> anyhow::Result<ReviewChat> {
    sqlx::query("SELECT review_helper, submit_helper, review_chat_id, submit_chat_id, publisher, app_info FROM chats WHERE (review_chat_id = ?)")
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
                testers: vec![],
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
        .map(|row| Ok(row.try_get("chat_type")?))?
}

/// TODO: this should add a new genesis, if contact_id is not yet set
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

/// TODO: this should add a new publisher, if contact_id is not yet set
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
    sqlx::query!("SELECT contact_id FROM users WHERE publisher=true ORDER BY RANDOM() LIMIT 1")
        .fetch_one(c)
        .await
        .map(|row| Ok(ContactId::new(row.contact_id as u32)))?
}

/// TODO: this should add a new tester, if contact_id is not yet set
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
    sqlx::query!(
        "SELECT contact_id FROM users WHERE tester=true ORDER BY RANDOM() LIMIT ?",
        count
    )
    .fetch_all(c)
    .await
    .map(|rows| {
        Ok(rows
            .into_iter()
            .map(|row| ContactId::new(row.contact_id as u32))
            .collect())
    })?
}

pub async fn increase_get_serial(c: &mut SqliteConnection) -> sqlx::Result<u32> {
    let serial = c
        .transaction(|txn| {
            Box::pin(async move {
                sqlx::query!("UPDATE config SET serial = serial + 1")
                    .execute(&mut **txn)
                    .await?;

                sqlx::query!("SELECT serial FROM config")
                    .fetch_one(&mut **txn)
                    .await
                    .map(|row| row.serial)
            })
        })
        .await?;
    Ok(serial as u32)
}

pub async fn create_app_info(
    c: &mut SqliteConnection,
    app_info: &mut AppInfo,
) -> anyhow::Result<()> {
    let next_serial = increase_get_serial(c).await?;
    let res = sqlx::query("INSERT INTO app_infos (name, description, version, image, author_name, author_email, xdc_blob_dir, active, originator, source_code_url, serial) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(app_info.name.as_str())
        .bind(&app_info.description)
        .bind(&app_info.version)
        .bind(&app_info.image)
        .bind(&app_info.author_name)
        .bind(&app_info.author_email)
        .bind(&app_info.xdc_blob_dir.as_deref().map(|a| a.to_str().unwrap()))
        .bind(&app_info.active)
        .bind(&app_info.originator)
        .bind(&app_info.source_code_url)
        .bind(next_serial)
        .bind(app_info.id)
        .execute(c)
        .await?;
    app_info.id = res.last_insert_rowid();
    Ok(())
}

pub async fn update_app_info(c: &mut SqliteConnection, app_info: &AppInfo) -> sqlx::Result<()> {
    sqlx::query("UPDATE app_infos SET name = ?, description = ?, version = ?, image = ?, author_name = ?, author_email = ?, xdc_blob_dir = ?, active = ?, originator = ?, source_code_url = ? WHERE id = ?")
        .bind(app_info.name.as_str())
        .bind(&app_info.description)
        .bind(&app_info.version)
        .bind(&app_info.image)
        .bind(&app_info.author_name)
        .bind(&app_info.author_email)
        .bind(&app_info.xdc_blob_dir.as_deref().map(|a| a.to_str().unwrap()))
        .bind(&app_info.active)
        .bind(&app_info.originator)
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
) -> anyhow::Result<AppInfo> {
    sqlx::query!("SELECT * FROM app_infos WHERE rowid = ?", resource_id)
        .fetch_one(c)
        .await
        .map(|a| {
            Ok(AppInfo {
                id: a.id as i64,
                name: a.name,
                description: a.description,
                version: a.version,
                image: a.image,
                author_name: a.author_name,
                author_email: a.author_email,
                xdc_blob_dir: a.xdc_blob_dir.map(|a| PathBuf::from(a)),
                active: a.active,
                originator: a.originator as i64,
                source_code_url: a.source_code_url,
            })
        })?
}

pub async fn _get_active_app_infos(c: &mut SqliteConnection) -> sqlx::Result<Vec<AppInfo>> {
    sqlx::query!("SELECT * FROM app_infos WHERE active = true")
        .fetch_all(c)
        .await
        .map(|rows| {
            Ok(rows
                .into_iter()
                .map(|a| AppInfo {
                    id: a.id as i64,
                    name: a.name,
                    description: a.description,
                    version: a.version,
                    image: a.image,
                    author_name: a.author_name,
                    author_email: a.author_email,
                    xdc_blob_dir: a.xdc_blob_dir.map(|a| PathBuf::from(a)),
                    active: a.active,
                    originator: a.originator as i64,
                    source_code_url: a.source_code_url,
                })
                .collect())
        })?
}

pub async fn get_active_app_infos_since(
    c: &mut SqliteConnection,
    serial: i64,
) -> anyhow::Result<Vec<AppInfo>> {
    sqlx::query!(
        "SELECT * FROM app_infos WHERE active = true AND serial > ?",
        serial
    )
    .fetch_all(c)
    .await
    .map(|rows| {
        Ok(rows
            .into_iter()
            .map(|a| AppInfo {
                id: a.id as i64,
                name: a.name,
                description: a.description,
                version: a.version,
                image: a.image,
                author_name: a.author_name,
                author_email: a.author_email,
                xdc_blob_dir: a.xdc_blob_dir.map(|a| PathBuf::from(a)),
                active: a.active,
                originator: a.originator as i64,
                source_code_url: a.source_code_url,
            })
            .collect())
    })?
}

pub async fn get_last_serial(c: &mut SqliteConnection) -> sqlx::Result<i64> {
    sqlx::query!("SELECT serial FROM config")
        .fetch_one(c)
        .await
        .map(|a| a.serial)
}

#[cfg(test)]
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
            ..Default::default()
        };

        upgrade_to_review_chat(&mut conn, &review_chat)
            .await
            .unwrap();

        get_review_chat(&mut conn, review_chat_id).await.unwrap();
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
