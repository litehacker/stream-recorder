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

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{AppState, error::AppError};

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub max_participants: Option<u32>,
    pub recording_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: String,
    pub name: String,
    pub max_participants: u32,
    pub recording_enabled: bool,
    pub current_participants: u32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

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
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, AppError> {
    let room_id = Uuid::new_v4().to_string();
    let room = state.rooms.create_room(
        room_id,
        request.name,
        request.max_participants.unwrap_or(10),
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
) -> Result<Json<Vec<RoomResponse>>, AppError> {
    info!("Listing rooms...");
    let rooms = state.rooms.list_rooms().await?;
    info!("Found {} rooms", rooms.len());
    
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
            id: Uuid::new_v4().to_string(),
            room_id: room_id.clone(),
            start_time: Utc::now(), // This should be parsed from filename
            end_time: None,
            size_bytes: 0, // This should be fetched from file metadata
        })
        .collect();

    Ok(Json(recordings))
} 