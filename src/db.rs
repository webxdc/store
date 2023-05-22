//! Integration fo SurrealDBpub struct DB
use deltachat::{chat::ChatId, contact::ContactId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surrealdb::{
    engine::local::{Db, File},
    sql::Thing,
    Surreal,
};

use crate::{
    bot::BotConfig,
    request_handlers::{AppInfo, AppInfoId, ChatType, ReviewChat},
};

#[derive(Serialize, Deserialize)]
struct DBChatType {
    chat_type: ChatType,
}

#[derive(Serialize, Deserialize)]
struct DBContactId {
    contact_id: ContactId,
}

pub struct DB {
    db: Surreal<Db>,
}

#[allow(unused)]
impl DB {
    pub async fn new(store: &str) -> Self {
        let db = Surreal::new::<File>(store).await.unwrap();
        db.use_ns("bot").use_db("bot").await.unwrap();
        Self { db }
    }

    pub async fn get_review_chat(&self, chat_id: ChatId) -> surrealdb::Result<Option<ReviewChat>> {
        self.db.select(("chat", chat_id.to_u32().to_string())).await
    }

    pub async fn get_review_chats(&self) -> surrealdb::Result<Vec<ReviewChat>> {
        self.db.select("chat").await
    }

    pub async fn create_chat(&self, chat: &ReviewChat) -> surrealdb::Result<ReviewChat> {
        self.db
            .create(("chat", chat.chat_id.to_u32().to_string()))
            .content(chat)
            .await
    }

    pub async fn set_chat_type(
        &self,
        chat_id: ChatId,
        chat_type: ChatType,
    ) -> surrealdb::Result<()> {
        let _t: DBChatType = self
            .db
            .create(("chattype", chat_id.to_u32().to_string()))
            .content(DBChatType { chat_type })
            .await?;
        Ok(())
    }

    pub async fn get_chat_type(&self, chat_id: ChatId) -> surrealdb::Result<Option<ChatType>> {
        let c: Result<Option<DBChatType>, _> = self
            .db
            .select(("chattype", chat_id.to_u32().to_string()))
            .await;
        c.map(|a| a.map(|a| a.chat_type))
    }

    pub async fn add_contact_to_genesis(&self, contact_id: ContactId) -> surrealdb::Result<()> {
        let _t: DBContactId = self
            .db
            .create(("genesis", contact_id.to_u32().to_string()))
            .content(DBContactId { contact_id })
            .await?;
        Ok(())
    }

    pub async fn set_genesis_contacts(&self, contacts: &[ContactId]) -> surrealdb::Result<()> {
        let _t: Vec<()> = self.db.delete("genesis").await?;
        for contact_id in contacts {
            self.add_contact_to_genesis(*contact_id).await?;
        }
        Ok(())
    }

    pub async fn create_publisher(&self, contact_id: ContactId) -> surrealdb::Result<()> {
        let _t: DBContactId = self
            .db
            .create(("publisher", contact_id.to_u32().to_string()))
            .content(DBContactId { contact_id })
            .await?;
        Ok(())
    }

    pub async fn get_publisher(&self) -> surrealdb::Result<ContactId> {
        let mut result = self
            .db
            .query("SELECT contact_id FROM publisher LIMIT 1")
            .await?;
        let contact_id: Vec<ContactId> = result.take((0, "contact_id")).unwrap();
        Ok(contact_id[0])
    }

    pub async fn create_tester(&self, contact_id: ContactId) -> surrealdb::Result<()> {
        let _t: DBContactId = self
            .db
            .create(("testers", contact_id.to_u32().to_string()))
            .content(DBContactId { contact_id })
            .await?;
        Ok(())
    }

    pub async fn get_testers(&self) -> surrealdb::Result<Vec<ContactId>> {
        let mut result = self
            .db
            .query("SELECT contact_id FROM testers LIMIT 3")
            .await?;

        let testers = result.take::<Vec<ContactId>>((0, "contact_id")).unwrap();
        Ok(testers)
    }

    pub async fn set_config(&self, config: &BotConfig) -> surrealdb::Result<BotConfig> {
        let _t: Option<BotConfig> = self.db.delete(("config", "config")).await.ok();
        self.db.create(("config", "config")).content(config).await
    }

    pub async fn get_config(&self) -> surrealdb::Result<BotConfig> {
        self.db.select(("config", "config")).await
    }

    pub async fn create_app_info(
        &self,
        app_info: &AppInfo,
        resource_id: Thing,
    ) -> surrealdb::Result<AppInfo> {
        self.db.create(resource_id).content(app_info).await
    }

    pub async fn update_app_info(
        &self,
        app_info: &AppInfo,
        id: &Thing,
    ) -> surrealdb::Result<AppInfo> {
        self.db.update(id.clone()).content(app_info).await
    }

    pub async fn publish_app(&self, id: &Thing) -> surrealdb::Result<AppInfo> {
        self.db
            .update(id.clone())
            .merge(json!({"active": true}))
            .await
    }

    pub async fn get_app_info(&self, resource_id: &Thing) -> surrealdb::Result<AppInfo> {
        self.db.select(resource_id.clone()).await
    }

    pub async fn get_active_app_infos(&self) -> surrealdb::Result<Vec<AppInfoId>> {
        let mut result = self.db.query("select * from app_info").await?;
        let testers = result.take::<Vec<AppInfoId>>(0)?;
        Ok(testers)
    }
}
