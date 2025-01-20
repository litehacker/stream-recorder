use serde::Deserialize;
use std::env;
use crate::error::AppError;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub jwt_secret: String,
    pub max_connections: u32,
    pub max_room_size: u32,
    pub storage_path: String,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        Ok(Self {
            api_key: env::var("API_KEY").unwrap_or_else(|_| "default_key".to_string()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "your-256-bit-secret".to_string()),
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            max_room_size: env::var("MAX_ROOM_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "data/recordings".to_string()),
        })
    }
} 