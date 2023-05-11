//! Integration fo SurrealDBpub struct DB {
#![allow(unused)]

use deltachat::{chat::ChatId, constants::Chattype};
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

    pub async fn get_chat(&self, chat_id: ChatId) -> Chat {
        Chat {
            chat_type: ChatType::Shop,
        }
    }
}
