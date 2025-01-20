/*
 * main.rs
 * Purpose: Entry point for the Stream Recorder application
 * 
 * This file contains:
 * - Application state and configuration setup
 * - Router configuration with all API endpoints
 * - Server initialization and startup
 * - Health check and metrics endpoints
 * - Middleware configuration (tracing, CORS, compression, timeout)
 */

use std::{sync::Arc, time::Duration};
use axum::{
    Router,
    routing::{get, post},
    extract::State,
};
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use tracing::info;

use crate::{
    error::AppError,
    config::Config,
    auth::Auth,
    rooms::Rooms,
    storage::Storage,
    monitoring::{MetricsStore, ResourceMonitor, ConnectionTracker},
    logging::setup_logging,
};

mod error;
mod config;
mod auth;
mod rooms;
mod storage;
mod monitoring;
mod logging;
mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub auth: Auth,
    pub rooms: Rooms,
    pub storage: Storage,
    pub metrics: MetricsStore,
    pub resource_monitor: ResourceMonitor,
    pub connection_tracker: ConnectionTracker,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Setup logging
    setup_logging()?;

    info!("Starting Stream Recorder server...");

    // Load config
    let config = Config::load()?;

    // Initialize state
    let state = Arc::new(AppState {
        config: config.clone(),
        auth: Auth::new(&config.jwt_secret)?,
        rooms: Rooms::new(),
        storage: Storage::new().await?,
        metrics: MetricsStore::new(),
        resource_monitor: ResourceMonitor::new(),
        connection_tracker: ConnectionTracker::new(),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/api/auth/credentials", post(handlers::auth::generate_credentials))
        .route("/api/rooms", post(handlers::room::create_room))
        .route("/api/rooms/:id/recordings", get(handlers::room::list_recordings))
        .route("/api/rooms/:id/ws", get(handlers::stream::ws_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
        )
        .with_state(state);

    // Start server
    let addr = "0.0.0.0:3000";
    info!("Listening on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics_handler(State(state): State<Arc<AppState>>) -> String {
    let mut output = String::new();
    
    // Add metrics sections
    output.push_str("# HELP stream_recorder_total_requests Total number of requests\n");
    output.push_str("# TYPE stream_recorder_total_requests counter\n");
    output.push_str(&format!("stream_recorder_total_requests {}\n", state.metrics.get_total_requests().await));
    
    output.push_str("# HELP stream_recorder_error_rate Error rate percentage\n");
    output.push_str("# TYPE stream_recorder_error_rate gauge\n");
    output.push_str(&format!("stream_recorder_error_rate {}\n", state.metrics.get_error_rate().await));
    
    output.push_str("# HELP stream_recorder_avg_latency Average request latency in milliseconds\n");
    output.push_str("# TYPE stream_recorder_avg_latency histogram\n");
    output.push_str(&format!("stream_recorder_avg_latency {}\n", state.metrics.get_avg_latency().await));
    
    output.push_str("# HELP stream_recorder_memory_usage Memory usage in bytes\n");
    output.push_str("# TYPE stream_recorder_memory_usage gauge\n");
    output.push_str(&format!("stream_recorder_memory_usage {}\n", state.resource_monitor.get_memory_usage().await));
    
    output.push_str("# HELP stream_recorder_cpu_usage CPU usage percentage\n");
    output.push_str("# TYPE stream_recorder_cpu_usage gauge\n");
    output.push_str(&format!("stream_recorder_cpu_usage {}\n", state.resource_monitor.get_cpu_usage().await));
    
    output
}

