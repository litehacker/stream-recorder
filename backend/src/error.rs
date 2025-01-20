use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

// Error metrics for monitoring
static TOTAL_ERRORS: AtomicU64 = AtomicU64::new(0);
static STREAMING_ERRORS: AtomicU64 = AtomicU64::new(0);
static STORAGE_ERRORS: AtomicU64 = AtomicU64::new(0);

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Invalid request: {0}")]
    ValidationError(String),
}

// Performance-optimized error response
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

        // Map error types to status codes
        let (status, error_message) = match self {
            AppError::AuthError(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::DatabaseError(ref e) => {
                // Log database errors with more detail for debugging
                tracing::error!(error = ?e, "Database error occurred");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::StorageError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::StreamingError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::RateLimitError(_) => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            AppError::NotFoundError(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        // Create error response with minimal allocation
        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
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