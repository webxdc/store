//! Integration fo SurrealDB

use surrealdb::{Datastore, Session};

pub struct DB {
    db: Datastore,
    session: Session,
}

#[allow(unused)]
impl DB {
    pub async fn new(store: &str) -> Self {
        let db = Datastore::new(store).await.unwrap();
        Self {
            db,
            session: Session::for_kv().with_ns("bot").with_db("bot"),
        }
    }

    async fn execute(&self, ast: &str) -> Result<Vec<surrealdb::Response>, surrealdb::Error> {
        self.db.execute(ast, &self.session, None, false).await
    }
}
