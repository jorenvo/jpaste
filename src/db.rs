use crate::utils::random_id;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
// use redis::aio::Connection;
// use redis::AsyncCommands;

pub type DbRef = Arc<Mutex<dyn Db + Send>>;

#[async_trait]
pub trait Db {
    async fn set(&mut self, data: Vec<u8>) -> String;
    async fn get(&self, id: &str) -> Option<&Vec<u8>>;
}

// pub struct RedisDB {
//     conn: Connection,
// }

// #[async_trait]
// impl Db for RedisDB {
//     async fn db_init() -> Self {
//         let client = redis::Client::open("redis://127.0.0.1/").unwrap();
//         RedisDB {
//             conn: client.get_async_connection().await.unwrap(),
//         }
//     }

//     async fn db_set_data(data: String) -> String {
//         "".to_string()
//     }
// }

pub struct NaiveDb {
    db: HashMap<String, Vec<u8>>,
}

impl NaiveDb {
    pub fn init() -> Self {
        NaiveDb { db: HashMap::new() }
    }
}

#[async_trait]
impl Db for NaiveDb {
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
