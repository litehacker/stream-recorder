use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

#[derive(Clone)]
pub struct Auth {
    encoding_key: EncodingKey,
}

impl Auth {
    pub fn new(jwt_secret: &str) -> Result<Self, AppError> {
        Ok(Self {
            encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
        })
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String, AppError> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 24 * 60 * 60; // 24 hours from now

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(e.to_string()))
    }
} 