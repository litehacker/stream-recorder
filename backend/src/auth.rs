use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Clone)]
pub struct Auth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Auth {
    pub fn new(jwt_secret: &str) -> Result<Self, AppError> {
        Ok(Self {
            encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
        })
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String, AppError> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 24 * 3600; // 24 hours from now

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::JwtError(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<(), AppError> {
        decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map(|_| ())
        .map_err(|e| AppError::JwtError(e.to_string()))
    }
} 