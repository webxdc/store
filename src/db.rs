//! Integration fo SurrealDBpub struct DB {
#![allow(unused)]

use deltachat::{chat::ChatId, constants::Chattype, contact::ContactId};
use serde::{Deserialize, Deserializer};
use surrealdb::{
    dbs::Session,
    engine::local::{Db, File},
    Surreal,
};

use crate::request_handlers::{Chat, ChatType};

pub struct DB {
    db: Surreal<Db>,
    session: Session,
}

#[allow(unused)]
impl DB {
    pub async fn new(store: &str) -> Self {
        let db = Surreal::new::<File>(store).await.unwrap();
        Self {
            db,
            session: Session::for_kv().with_ns("bot").with_db("bot"),
        }
    }

    pub async fn get_chat(&self, chat_id: ChatId) -> anyhow::Result<Option<Chat>> {
        self.db
            .select(("chat", chat_id.to_string()))
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn get_chats(&self) -> surrealdb::Result<Vec<Chat>> {
        self.db.select("chat").await
    }

    pub async fn create_chat(&self, chat: Chat) -> surrealdb::Result<()> {
        self.db
            .create(("chat", chat.chat_id.to_string()))
            .content(chat)
            .await
    }

    pub async fn create_publisher(&self, chat_id: ChatId) -> surrealdb::Result<()> {
        self.db
            .create(("publisher", chat_id.to_string()))
            .content(chat_id)
            .await
    }

    pub async fn get_pubslishers(&self) -> surrealdb::Result<Vec<ContactId>> {
        self.db.select("publisher").await
    }

    pub async fn create_tester(&self, chat_id: ChatId) -> surrealdb::Result<()> {
        self.db
            .create(("testers", chat_id.to_string()))
            .content(chat_id)
            .await
    }

    pub async fn get_testers(&self) -> surrealdb::Result<Vec<ContactId>> {
        self.db.select("testers").await
    }
}
