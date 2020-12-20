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
    async fn get(&self, id: &str) -> Option<&Vec<u8>>;
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
}

#[async_trait]
impl Db for RedisDb {
    async fn set(&mut self, data: Vec<u8>) -> String {
        let id = random_id();
        let key = REDIS_PREFIX.to_string() + &id;
        let _: () = self.conn.set(key, data).await.unwrap();
        id
    }

    async fn get<'a>(&'a self, id: &str) -> Option<&'a Vec<u8>> {
        unimplemented!()
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

    async fn get<'a>(&'a self, id: &str) -> Option<&'a Vec<u8>> {
        self.db.get(id)
    }
}

// async fn db_set_data(data: String) -> String {
//     client
//         .get_async_connection()
//         .await
//         .map_err(|e| RedisClientError(e).into())
// }
