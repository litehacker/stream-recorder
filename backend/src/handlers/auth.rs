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
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::{AppState, error::AppError};
use tower_cookies::{Cookie, Cookies};

#[derive(Debug, Serialize)]
pub struct CredentialsResponse {
    pub message: String,
}

pub async fn generate_credentials(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let api_key = Uuid::new_v4().to_string();
    let token = state.auth.generate_token(&api_key)?;
    
    // Create a secure HTTP-only cookie with the JWT token
    let cookie = Cookie::build("jwt_token", token)
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .finish();
    
    cookies.add(cookie);
    
    Ok(Json(CredentialsResponse { 
        message: "Authentication successful".to_string() 
    }))
}

pub async fn require_auth<B>(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // First try to get token from cookie
    if let Some(cookie) = cookies.get("jwt_token") {
        if state.auth.validate_token(cookie.value()).is_ok() {
            return Ok(next.run(req).await);
        }
    }
    
    // Fallback to Authorization header
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