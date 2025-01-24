pub async fn record_metrics(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RecordMetricsRequest>,
) -> Result<impl IntoResponse, AppError> {
    state.metrics.record_bytes(request.room_id.clone(), request.bytes_transferred);
    state.metrics.record_frames(request.room_id.clone(), request.frames_processed);
    state.metrics.record_errors(request.room_id, request.error_count);
    Ok(StatusCode::OK)
}

pub async fn get_room_metrics(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomMetrics>, AppError> {
    let metrics = state.metrics.get_room_metrics(&room_id).await?;
    Ok(Json(metrics))
}

pub async fn get_user_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserMetrics>, AppError> {
    let metrics = state.metrics.get_user_metrics().await?;
    Ok(Json(metrics))
} 