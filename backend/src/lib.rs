pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod rooms;
pub mod storage;
pub mod monitoring;
pub mod logging;

use std::sync::Arc;
use auth::Auth;
use rooms::Rooms;
use storage::Storage;
use monitoring::{MetricsStore, ConnectionTracker};

pub struct AppState {
    pub auth: Auth,
    pub rooms: Rooms,
    pub storage: Storage,
    pub metrics: MetricsStore,
    pub connection_tracker: ConnectionTracker,
}

impl AppState {
    pub async fn new(jwt_secret: &[u8]) -> Result<Arc<Self>, error::AppError> {
        Ok(Arc::new(Self {
            auth: Auth::new(jwt_secret),
            rooms: Rooms::new(),
            storage: Storage::new().await?,
            metrics: MetricsStore::new(),
            connection_tracker: ConnectionTracker::new(),
        }))
    }
} 