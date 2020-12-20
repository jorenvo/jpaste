use crate::utils::random_id;
use async_trait::async_trait;
// use redis::AsyncCommands;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type DbRef = Arc<Mutex<dyn Db + Send>>;

#[async_trait]
pub trait Db {
    async fn set(&mut self, data: Vec<u8>) -> String;
    async fn get(&mut self, id: &str) -> Option<Vec<u8>>;
}

const REDIS_PREFIX: &str = "jpaste:";

pub struct RedisDb {
    // Perhaps have a Client instead of sharing the same Connection everywhere?
    conn: redis::aio::Connection,
}

impl RedisDb {
    pub async fn init() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        RedisDb {
            conn: client.get_async_connection().await.unwrap(),
        }
    }

    fn redis_key(&self, id: &str) -> String {
        REDIS_PREFIX.to_string() + id
    }
}

#[async_trait]
impl Db for RedisDb {
    async fn set(&mut self, data: Vec<u8>) -> String {
        let id = random_id();
        let _: () = self.conn.set(self.redis_key(&id), data).await.unwrap();
        id
    }

    async fn get(&mut self, id: &str) -> Option<Vec<u8>> {
        self.conn.get(self.redis_key(id)).await.unwrap()
    }
}

pub struct InMemoryDb {
    db: HashMap<String, Vec<u8>>,
}

impl InMemoryDb {
    pub fn init() -> Self {
        InMemoryDb { db: HashMap::new() }
    }
}

#[async_trait]
impl Db for InMemoryDb {
    async fn set(&mut self, data: Vec<u8>) -> String {
        let id = random_id();
        self.db.insert(id.clone(), data);
        id
    }

    async fn get(&mut self, id: &str) -> Option<Vec<u8>> {
        self.db.get(id).cloned()
    }
}

// async fn db_set_data(data: String) -> String {
//     client
//         .get_async_connection()
//         .await
//         .map_err(|e| RedisClientError(e).into())
// }
