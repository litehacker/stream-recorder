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
    middleware,
};
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, Any},
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use tower_cookies::CookieManagerLayer;
use tracing::info;

use crate::{
    error::AppError,
    config::Config,
    auth::Auth,
    rooms::Rooms,
    storage::Storage,
    monitoring::{MetricsStore, ResourceMonitor, ConnectionTracker},
    logging::setup_logging,
    handlers::auth::require_auth,
};

mod error;
mod config;
mod auth;
mod rooms;
mod storage;
mod monitoring;
mod logging;
mod handlers;
mod models;

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
        auth: Auth::new(config.jwt_secret.as_bytes()),
        rooms: Rooms::new(),
        storage: Storage::new().await?,
        metrics: MetricsStore::new(),
        resource_monitor: ResourceMonitor::new(),
        connection_tracker: ConnectionTracker::new(),
    });

    // Protected API routes
    let api_routes = Router::new()
        .route("/rooms", post(handlers::room::create_room))
        .route("/rooms", get(handlers::room::list_rooms))
        .route("/rooms/:id/recordings", get(handlers::room::list_recordings))
        .route("/rooms/:id/ws", get(handlers::stream::ws_handler))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/api/auth/credentials", post(handlers::auth::generate_credentials))
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_credentials(true)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CookieManagerLayer::new())
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

async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<String, AppError> {
    let mut output = String::new();
    
    // Add basic metrics for all rooms
    output.push_str(&format!("rooms_total {}\n", state.rooms.list_rooms("*").await?.len()));
    
    Ok(output)
}

