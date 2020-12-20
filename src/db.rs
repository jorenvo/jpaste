use crate::utils::random_id;
use async_trait::async_trait;
// use redis::AsyncCommands;
use redis::AsyncCommands;
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
        const REDIS_EXP_S: usize = 60 * 60 * 24 * 30; // ~1 month
        let id = random_id();
        let redis_key = self.redis_key(&id);
        let _: () = self.conn.set(&redis_key, data).await.unwrap();
        let _: () = self.conn.expire(&redis_key, REDIS_EXP_S).await.unwrap();
        id
    }

    async fn get(&mut self, id: &str) -> Option<Vec<u8>> {
        self.conn.get(self.redis_key(id)).await.unwrap()
    }
}

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
pub struct InMemoryDb {
    db: HashMap<String, Vec<u8>>,
}

#[cfg(test)]
impl InMemoryDb {
    pub fn init() -> Self {
        InMemoryDb { db: HashMap::new() }
    }
}

#[cfg(test)]
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
