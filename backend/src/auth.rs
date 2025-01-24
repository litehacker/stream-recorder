use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::error::AppError;

const JWT_SECRET: &[u8] = b"your-secret-key";  // In production, this should be properly configured

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (API key)
    pub user_id: String,  // Unique user ID
    pub exp: i64,         // Expiration time
}

#[derive(Clone)]
pub struct Auth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Auth {
    pub fn new(jwt_secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(jwt_secret),
            decoding_key: DecodingKey::from_secret(jwt_secret),
        }
    }

    pub fn generate_token(&self, api_key: &str) -> Result<String, AppError> {
        let user_id = Uuid::new_v4().to_string();
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: api_key.to_string(),
            user_id,
            exp: expiration,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Unauthorized(format!("Failed to create token: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )
        .map(|token_data| token_data.claims)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    }
} 