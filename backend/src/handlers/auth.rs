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
    extract::{State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::{AppState, error::AppError};

#[derive(Debug, Serialize)]
pub struct CredentialsResponse {
    pub token: String,
}

pub async fn generate_credentials(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CredentialsResponse>, AppError> {
    let api_key = Uuid::new_v4().to_string();
    let token = state.auth.generate_token(&api_key)?;
    
    Ok(Json(CredentialsResponse { token }))
}

pub async fn require_auth<B>(
    State(state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    match auth_header {
        Some(token) if state.auth.validate_token(token).is_ok() => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
} 