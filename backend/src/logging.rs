use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::time::Duration;
use tokio::time::interval;
use crate::error::ErrorMetrics;

// Performance-optimized log buffer size
const LOG_BUFFER_SIZE: usize = 8192;
// Log rotation interval in seconds
const LOG_ROTATION_INTERVAL: u64 = 3600;

pub fn setup_logging() {
    // Create a custom format that includes essential information
    let format = fmt::format()
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(false) // Disable for performance
        .with_target(false)       // Disable for performance
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE) // Only log span close events
        .compact();

    // Create a buffered layer for better performance
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::RollingFileAppender::builder()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix("stream-recorder")
            .filename_suffix("log")
            .build()
            .unwrap(),
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up the subscriber with multiple layers
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,stream_recorder=debug".into())
        )
        // Console output for development
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_filter(EnvFilter::new("info"))
                .event_format(format.clone())
        )
        // File output with buffering
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking)
                .with_filter(EnvFilter::new("debug"))
                .event_format(format)
                .with_ansi(false)
        )
        .init();

    // Start log rotation and cleanup task
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(LOG_ROTATION_INTERVAL));
        loop {
            interval.tick().await;
            cleanup_old_logs().await;
            ErrorMetrics::reset(); // Reset error metrics periodically
        }
    });
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

// Cleanup old log files
async fn cleanup_old_logs() {
    use tokio::fs;
    use chrono::{DateTime, Utc, Duration};

    let log_dir = "logs";
    let retention_days = 7;

    if let Ok(mut entries) = fs::read_dir(log_dir).await {
        let now = Utc::now();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    let modified: DateTime<Utc> = modified.into();
                    if now - modified > Duration::days(retention_days) {
                        let _ = fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
    }
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