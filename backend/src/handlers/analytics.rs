/*
 * handlers/analytics.rs
 * Purpose: Analytics and metrics endpoints
 * 
 * This file contains:
 * - Room analytics collection and reporting
 * - User analytics aggregation
 * - Performance metrics recording
 * - Resource usage tracking
 * - Analytics data structures
 */

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{AppState, error::AppError};

#[derive(Debug, Deserialize)]
pub struct MetricsRequest {
    pub room_id: String,
    pub bytes_transferred: u64,
    pub frames_processed: u64,
    pub error_count: u64,
}

#[derive(Debug, Serialize)]
pub struct RoomAnalytics {
    pub total_bytes: u64,
    pub total_frames: u64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct UserAnalytics {
    pub total_rooms: u64,
    pub total_storage: u64,
    pub total_bandwidth: u64,
}

pub async fn record_metrics(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MetricsRequest>,
) -> Result<(), AppError> {
    info!("Recording metrics for room {}", request.room_id);

    // Update metrics store
    state.metrics.record_bytes(request.room_id.clone(), request.bytes_transferred);
    state.metrics.record_frames(request.room_id.clone(), request.frames_processed);
    state.metrics.record_errors(request.room_id, request.error_count);

    Ok(())
}

pub async fn get_room_analytics(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomAnalytics>, AppError> {
    let metrics = state.metrics.get_room_metrics(&room_id).await?;
    
    Ok(Json(RoomAnalytics {
        total_bytes: metrics.bytes_transferred,
        total_frames: metrics.frames_processed,
        error_rate: metrics.error_rate,
        avg_latency_ms: metrics.avg_latency,
    }))
}

pub async fn get_user_analytics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserAnalytics>, AppError> {
    let metrics = state.metrics.get_user_metrics().await?;
    
    Ok(Json(UserAnalytics {
        total_rooms: metrics.total_rooms,
        total_storage: metrics.total_storage,
        total_bandwidth: metrics.total_bandwidth,
    }))
} 