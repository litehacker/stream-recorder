use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use crate::{
    AppState,
    models::{StreamMetrics, RoomAnalytics, UserAnalytics},
};

pub async fn record_metrics(
    State(state): State<AppState>,
    Json(metrics): Json<StreamMetrics>,
) -> Json<()> {
    sqlx::query!(
        "INSERT INTO stream_metrics 
         (id, room_id, timestamp, bytes_transferred, frames_processed, 
          frames_deduplicated, current_bitrate, current_fps, peak_memory_mb)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        Uuid::new_v4(),
        metrics.room_id,
        metrics.timestamp,
        metrics.bytes_transferred,
        metrics.frames_processed,
        metrics.frames_deduplicated,
        metrics.current_bitrate,
        metrics.current_fps,
        metrics.peak_memory_mb,
    )
    .execute(&state.db)
    .await
    .unwrap();

    Json(())
}

pub async fn get_room_analytics(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Json<RoomAnalytics> {
    let analytics = sqlx::query_as!(
        RoomAnalytics,
        "SELECT 
            total_storage_used,
            total_stream_time,
            total_recordings,
            avg_bitrate,
            avg_fps,
            deduplication_ratio
         FROM room_analytics
         WHERE room_id = $1",
        room_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(analytics)
}

pub async fn get_user_analytics(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Json<UserAnalytics> {
    let user = sqlx::query!(
        "SELECT quota_limit, quota_used FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    let rooms_analytics = sqlx::query_as!(
        RoomAnalytics,
        "SELECT 
            total_storage_used,
            total_stream_time,
            total_recordings,
            avg_bitrate,
            avg_fps,
            deduplication_ratio
         FROM room_analytics
         WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    let total_storage: i64 = rooms_analytics
        .iter()
        .map(|r| r.total_storage_used)
        .sum();

    let total_stream_time: i64 = rooms_analytics
        .iter()
        .map(|r| r.total_stream_time)
        .sum();

    Json(UserAnalytics {
        total_rooms: rooms_analytics.len() as i64,
        total_storage_used: total_storage,
        total_stream_time,
        quota_percentage: (user.quota_used as f32 / user.quota_limit as f32) * 100.0,
        rooms_analytics,
    })
} 