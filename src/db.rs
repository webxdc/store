//! Integration fo SurrealDBpub struct DB
use deltachat::{chat::ChatId, contact::ContactId};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::local::{Db, File},
    Surreal,
};

use crate::request_handlers::{ChatType, ReviewChat};

pub struct DB {
    db: Surreal<Db>,
}
#[derive(Serialize, Deserialize)]
struct DBChatType {
    chat_type: ChatType,
}

#[derive(Serialize, Deserialize)]
struct DBContactId {
    contact_id: ContactId,
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

    pub async fn create_chat(&self, chat: ReviewChat) -> surrealdb::Result<()> {
        self.set_chat_type(chat.chat_id, chat.chat_type).await?;

        let _t: ReviewChat = self
            .db
            .create(("chat", chat.chat_id.to_u32().to_string()))
            .content(chat)
            .await?;
        Ok(())
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

    pub async fn create_tester(&self, contact: ContactId) -> surrealdb::Result<()> {
        let _t: DBContactId = self
            .db
            .create(("testers", contact.to_u32().to_string()))
            .content(DBContactId {
                contact_id: contact,
            })
            .await?;
        Ok(())
    }

    pub async fn get_testers(&self) -> surrealdb::Result<Vec<ContactId>> {
        let mut result = self
            .db
            .query("SELECT contact_id FROM testers LIMIT 3")
            .await?;

        let users = result.take::<Vec<ContactId>>((0, "contact_id")).unwrap();
        Ok(users)
    }
}
