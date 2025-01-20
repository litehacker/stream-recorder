mod models;
mod handlers;
mod storage;
mod error;
mod logging;
mod monitoring;

use axum::{
    routing::{get, post},
    Router,
    extract::Extension,
    http::StatusCode,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{
    trace::TraceLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
};
use crate::{
    error::{AppError, AppResult},
    logging::{setup_logging, log_performance_metrics},
    monitoring::{MetricsStore, ResourceMonitor, ConnectionTracker, monitor_gc},
};

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    redis: Arc<redis::Client>,
    storage: Arc<storage::StorageClient>,
    metrics: MetricsStore,
    resource_monitor: ResourceMonitor,
    connection_tracker: ConnectionTracker,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    setup_logging()?;

    // Initialize monitoring
    let metrics_store = MetricsStore::new();
    let resource_monitor = ResourceMonitor::new(1024, 80.0); // 1GB memory, 80% CPU
    let connection_tracker = ConnectionTracker::new(1000); // 1000 concurrent connections
    monitor_gc();

    // Initialize database connection with retry logic
    let db = connect_database().await?;

    // Initialize Redis with optimized connection pool
    let redis = Arc::new(redis::Client::open("redis://redis")
        .map_err(|e| AppError::StorageError(e.to_string()))?);

    // Initialize storage client with performance monitoring
    let storage = Arc::new(storage::StorageClient::new()
        .map_err(|e| AppError::StorageError(e.to_string()))?);

    let state = AppState {
        db,
        redis,
        storage,
        metrics: metrics_store,
        resource_monitor,
        connection_tracker,
    };

    // Create router with monitoring middleware
    let router = Router::new()
        .route("/health", get(health_check))
        .route("/auth", post(handlers::auth::generate_credentials))
        .route("/room", post(handlers::room::create_room))
        .route("/room/:room_id/ws", get(handlers::stream::ws_handler))
        .route(
            "/room/:room_id/recordings",
            get(handlers::room::list_recordings),
        )
        // Analytics routes
        .route(
            "/analytics/metrics",
            post(handlers::analytics::record_metrics),
        )
        .route(
            "/analytics/room/:room_id",
            get(handlers::analytics::get_room_analytics),
        )
        .route(
            "/analytics/user/:user_id",
            get(handlers::analytics::get_user_analytics),
        )
        .route("/metrics", get(metrics_handler))
        // Performance-optimized middleware
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<_>| {
                        tracing::info_span!(
                            "http_request",
                            method = %request.method(),
                            uri = %request.uri(),
                            version = ?request.version(),
                        )
                    })
                    .on_request(|_request: &Request<_>, _span: &Span| {
                        // Record request start
                    })
                    .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                        // Record response metrics
                        let endpoint = response.extensions().get::<String>()
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");
                        
                        metrics_store.record_latency(endpoint, latency.as_secs_f64());
                        
                        if response.status().is_server_error() {
                            metrics_store.record_error(endpoint);
                        }
                        
                        metrics_store.record_request(endpoint);
                    })
                )
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .into_inner()
        );

    // Start the server with monitoring
    let addr = "0.0.0.0:3000";
    tracing::info!("Starting server on {}", addr);
    
    axum::Server::bind(&addr.parse().unwrap())
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::StreamingError(e.to_string()))?;

    Ok(())
}

// Health check endpoint with monitoring
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    // Check system resources
    if !state.resource_monitor.check_resources() {
        return Err(AppError::ResourceExhausted);
    }

    // Check active connections
    if !state.connection_tracker.track_connection() {
        return Err(AppError::TooManyConnections);
    }

    Ok(StatusCode::OK)
}

// Metrics endpoint
async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<String, AppError> {
    let mut output = String::new();

    // Add basic metrics
    writeln!(output, "# HELP stream_recorder_requests_total Total number of requests")?;
    writeln!(output, "# TYPE stream_recorder_requests_total counter")?;
    
    // Add error rates
    writeln!(output, "# HELP stream_recorder_error_rate Error rate by endpoint")?;
    writeln!(output, "# TYPE stream_recorder_error_rate gauge")?;
    
    // Add latencies
    writeln!(output, "# HELP stream_recorder_latency_seconds Request latency in seconds")?;
    writeln!(output, "# TYPE stream_recorder_latency_seconds histogram")?;
    
    // Add resource usage
    writeln!(output, "# HELP stream_recorder_memory_bytes Memory usage in bytes")?;
    writeln!(output, "# TYPE stream_recorder_memory_bytes gauge")?;
    
    writeln!(output, "# HELP stream_recorder_cpu_percent CPU usage percentage")?;
    writeln!(output, "# TYPE stream_recorder_cpu_percent gauge")?;

    Ok(output)
}

// Database connection with retry logic
async fn connect_database() -> AppResult<PgPool> {
    use tokio::time::{sleep, Duration};
    const MAX_RETRIES: u32 = 5;
    const RETRY_DELAY: u64 = 5;

    let mut retries = 0;
    let db_url = "postgres://postgres:password@postgres/streamrecorder";

    loop {
        match PgPool::connect(db_url).await {
            Ok(pool) => {
                tracing::info!("Database connection established");
                return Ok(pool);
            }
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(AppError::DatabaseError(e));
                }
                tracing::warn!(
                    "Database connection failed, retrying in {} seconds (attempt {}/{})",
                    RETRY_DELAY, retries, MAX_RETRIES
                );
                sleep(Duration::from_secs(RETRY_DELAY)).await;
            }
        }
    }
}

// Graceful shutdown handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            tracing::info!("Received terminate signal");
        }
    }

    // Log final metrics before shutdown
    log_performance_metrics("uptime_seconds", 0.0, Some("shutdown"));
}
