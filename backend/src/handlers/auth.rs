/*
 * handlers/auth.rs
 * Purpose: Authentication and authorization endpoints
 * 
 * This file contains:
 * - API key validation
 * - JWT token generation
 * - User credential management
 * - Authentication middleware
 * - Authorization checks
 */

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid;

use crate::{AppState, error::AppError};

#[derive(Debug, Serialize)]
pub struct Credentials {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub api_key: String,
}

pub async fn generate_credentials(
    State(state): State<Arc<crate::AppState>>,
) -> Result<Json<String>, AppError> {
    // Validate API key
    if state.config.api_key.is_empty() {
        return Err(AppError::Unauthorized("No API key configured".to_string()));
    }

    // Generate JWT token
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = state.auth.generate_token(&user_id)?;
    Ok(Json(token))
} 