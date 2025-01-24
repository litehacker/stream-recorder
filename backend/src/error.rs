/*
 * error.rs
 * Purpose: Error handling and error type definitions
 * 
 * This file contains:
 * - Custom AppError type with various error variants
 * - Error response formatting and serialization
 * - Error metrics collection and monitoring
 * - Error conversion implementations for standard error types
 * - Performance-optimized logging macros for errors
 */

use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::Serialize;
use std::{fmt, io};

// Error metrics for monitoring
static TOTAL_ERRORS: AtomicU64 = AtomicU64::new(0);
static STREAMING_ERRORS: AtomicU64 = AtomicU64::new(0);
static STORAGE_ERRORS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    NotFound(String),
    ResourceExhausted(String),
    TooManyConnections(String),
    StorageError(String),
    StreamingError(String),
    InternalError(String),
    JwtError(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            AppError::TooManyConnections(msg) => write!(f, "Too many connections: {}", msg),
            AppError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            AppError::StreamingError(msg) => write!(f, "Streaming error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::JwtError(msg) => write!(f, "JWT error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Increment error metrics atomically
        TOTAL_ERRORS.fetch_add(1, Ordering::Relaxed);
        
        // Update specific error metrics
        match &self {
            AppError::StreamingError(_) => {
                STREAMING_ERRORS.fetch_add(1, Ordering::Relaxed);
            }
            AppError::StorageError(_) => {
                STORAGE_ERRORS.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        let (status, message) = match self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::ResourceExhausted(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            AppError::TooManyConnections(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::StreamingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::JwtError(msg) => (StatusCode::UNAUTHORIZED, msg),
        };

        let body = Json(ErrorResponse {
            code: status.as_u16(),
            message,
        });

        (status, body).into_response()
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<std::net::AddrParseError> for AppError {
    fn from(err: std::net::AddrParseError) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<hyper::Error> for AppError {
    fn from(err: hyper::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<std::fmt::Error> for AppError {
    fn from(err: std::fmt::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

// Error metrics collection
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub streaming_errors: u64,
    pub storage_errors: u64,
}

impl ErrorMetrics {
    pub fn get_current() -> Self {
        Self {
            total_errors: TOTAL_ERRORS.load(Ordering::Relaxed),
            streaming_errors: STREAMING_ERRORS.load(Ordering::Relaxed),
            storage_errors: STORAGE_ERRORS.load(Ordering::Relaxed),
        }
    }

    pub fn reset() {
        TOTAL_ERRORS.store(0, Ordering::Relaxed);
        STREAMING_ERRORS.store(0, Ordering::Relaxed);
        STORAGE_ERRORS.store(0, Ordering::Relaxed);
    }
}

// Result type alias for application
pub type AppResult<T> = Result<T, AppError>;

// Performance-optimized logging macros
#[macro_export]
macro_rules! log_error {
    ($err:expr) => {{
        tracing::error!(
            error = ?$err,
            error_type = std::any::type_name_of_val(&$err),
            "Error occurred"
        );
    }};
    ($err:expr, $msg:expr) => {{
        tracing::error!(
            error = ?$err,
            error_type = std::any::type_name_of_val(&$err),
            message = $msg
        );
    }};
}

#[macro_export]
macro_rules! log_warning {
    ($msg:expr) => {{
        tracing::warn!(message = $msg);
    }};
    ($msg:expr, $data:expr) => {{
        tracing::warn!(message = $msg, data = ?$data);
    }};
} 