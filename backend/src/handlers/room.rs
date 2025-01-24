/*
 * handlers/room.rs
 * Purpose: Room management and recording endpoints
 * 
 * This file contains:
 * - Room creation and configuration
 * - Recording management (start/stop/list)
 * - Room state tracking
 * - Room capacity management
 * - Room metrics collection
 */

use crate::{
    error::AppError,
    models::{CreateRoomRequest, RoomResponse},
    AppState,
};

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

#[derive(Debug, Serialize)]
pub struct Recording {
    pub id: String,
    pub room_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub size_bytes: u64,
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, AppError> {
    let token = cookies.get("jwt_token")
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?
        .value()
        .to_string();
    
    let claims = state.auth.validate_token(&token)?;
    let user_id = claims.user_id.clone();

    let room = state.rooms.create_room(
        Uuid::new_v4().to_string(),
        req.name,
        req.max_participants.unwrap_or(10),  // Default to 10 if not specified
        user_id,
    ).await?;
    
    Ok(Json(RoomResponse {
        id: room.id,
        name: room.name,
        max_participants: room.max_participants,
        recording_enabled: room.recording_enabled,
        current_participants: room.current_participants,
        start_time: Utc::now(),
        end_time: None,
    }))
}

pub async fn list_rooms(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> Result<Json<Vec<RoomResponse>>, AppError> {
    // Get user_id from JWT token in cookie
    let token = cookies.get("jwt_token")
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?
        .value()
        .to_string();
    
    let claims = state.auth.validate_token(&token)?;
    let user_id = claims.user_id.clone();
    
    let rooms = state.rooms.list_rooms(&user_id).await?;
    
    let room_responses = rooms.into_iter()
        .map(|room| RoomResponse {
            id: room.id,
            name: room.name,
            max_participants: room.max_participants,
            recording_enabled: room.recording_enabled,
            current_participants: room.current_participants,
            start_time: Utc::now(), // This should ideally come from room creation time
            end_time: None,
        })
        .collect();

    Ok(Json(room_responses))
}

pub async fn list_recordings(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<Recording>>, AppError> {
    let filenames = state.storage.list_recordings(&room_id).await?;
    
    let recordings = filenames.into_iter()
        .map(|filename| Recording {
            id: filename.clone(),
            room_id: room_id.clone(),
            start_time: Utc::now(), // This should be parsed from filename
            end_time: None,
            size_bytes: 0, // This should be fetched from file metadata
        })
        .collect();

    Ok(Json(recordings))
} 