/*
 * logging.rs
 * Purpose: Logging configuration and management
 * 
 * This file contains:
 * - Logging setup with file and console output
 * - Log rotation and cleanup functionality
 * - Performance metrics logging helpers
 * - Structured logging for errors and metrics
 * - Custom log filtering for high-volume events
 */

use tracing_subscriber::{
    fmt,
    EnvFilter,
    Layer,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::{io, time::Duration};
use tokio::time::interval;
use serde_json;
use crate::error::AppError;

const LOG_DIR: &str = "logs";
const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const MAX_LOG_FILES: usize = 5;

pub fn setup_logging() -> Result<(), AppError> {
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(LOG_DIR)?;

    // Configure formatting
    let format = fmt::format()
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(false) // Disable for performance
        .with_target(true)
        .with_line_number(true)
        .with_file(true)
        .compact();

    // Configure file appender
    let file_appender = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/stream-recorder.log", LOG_DIR))?;

    // Create subscribers
    let file_layer = fmt::Layer::new()
        .with_writer(file_appender)
        .with_ansi(false)
        .event_format(format.clone());

    let stdout_layer = fmt::Layer::new()
        .with_writer(io::stdout)
        .with_ansi(true)
        .event_format(format);

    // Create filters
    let file_filter = EnvFilter::new("debug");
    let stdout_filter = EnvFilter::new("info");

    // Combine layers with filters
    tracing_subscriber::registry()
        .with(file_layer.with_filter(file_filter))
        .with(stdout_layer.with_filter(stdout_filter))
        .init();

    Ok(())
}

pub fn cleanup_old_logs() -> Result<(), AppError> {
    let mut log_files: Vec<_> = std::fs::read_dir(LOG_DIR)?
        .filter_map(|entry| entry.ok())
        .collect();

    // Sort by modified time
    log_files.sort_by_key(|entry| {
        entry.metadata()
            .unwrap()
            .modified()
            .unwrap()
    });

    // Remove old files if we have too many
    while log_files.len() > MAX_LOG_FILES {
        if let Some(file) = log_files.first() {
            std::fs::remove_file(file.path())?;
            log_files.remove(0);
        }
    }

    // Check sizes and rotate if needed
    for file in log_files {
        let metadata = file.metadata()?;
        if metadata.len() > MAX_LOG_SIZE {
            let path = file.path();
            let new_path = path.with_extension("log.old");
            std::fs::rename(&path, &new_path)?;
        }
    }

    Ok(())
}

// Structured logging helper for performance metrics
pub fn log_performance_metrics(metrics: &str, value: f64, context: Option<&str>) {
    tracing::info!(
        metric = metrics,
        value = value,
        context = context,
        "Performance metric recorded"
    );
}

// Helper function to log errors with context
pub fn log_error_with_context(error: &str, context: &str, data: Option<&serde_json::Value>) {
    tracing::error!(
        error = error,
        context = context,
        data = ?data,
        "Error occurred with context"
    );
}

// Performance monitoring for logging itself
#[cfg(debug_assertions)]
pub fn monitor_logging_performance() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static LOG_COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let count = LOG_COUNTER.swap(0, Ordering::Relaxed);
            tracing::info!(
                logs_per_minute = count,
                "Logging performance metrics"
            );
        }
    });
}

// Custom log filter for high-volume events
pub fn should_log(level: tracing::Level, module: &str) -> bool {
    match level {
        tracing::Level::ERROR | tracing::Level::WARN => true,
        tracing::Level::INFO => {
            // Rate limit info logs from high-volume modules
            !matches!(module, "stream" | "metrics")
        }
        tracing::Level::DEBUG | tracing::Level::TRACE => {
            // Only log debug/trace for specific modules
            matches!(module, "auth" | "storage")
        }
    }
} 