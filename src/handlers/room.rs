use axum::{
    extract::{Path, State},
    Json,
};
use tower_cookies::Cookies;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    error::AppError,
    models::{CreateRoomRequest, RoomResponse},
    state::AppState,
};

let room = state.rooms.create_room(
    req.id,
    req.name,
    req.max_participants.unwrap_or(10),  // Default to 10 if not specified
    user_id,
).await?; 

pub async fn list_recordings(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<String>>, AppError> {
    let filenames = state.storage.list_recordings(&room_id).await?;
    Ok(Json(filenames))
} 