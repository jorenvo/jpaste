use std::env;

#[derive(Clone)]
pub struct Config {
    pub redis_url: String,
    pub jpaste_public_url: String,
    pub jpaste_localhost_ip: String,
    pub jpaste_localhost_port: String,
}

impl Config {
    pub fn init() -> Self {
        Config {
            redis_url: env::var("JPASTE_REDIS")
                .unwrap_or_else(|_| "redis://127.0.0.1/".to_string()),
            jpaste_public_url: env::var("JPASTE_PUBLIC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1".to_string()),
            jpaste_localhost_ip: env::var("JPASTE_LOCALHOST_IP")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            jpaste_localhost_port: env::var("JPASTE_LOCALHOST_PORT")
                .unwrap_or_else(|_| "3030".to_string()),
        }
    }
}
