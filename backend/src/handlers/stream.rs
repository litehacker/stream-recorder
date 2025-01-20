use std::sync::Arc;
use axum::{
    extract::{ws::{WebSocket, Message}, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{stream::StreamExt, SinkExt};
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::{
    AppState,
    models::{WebSocketMessage, FrameType, ControlAction},
};
use redis::AsyncCommands;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    room_id: String,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, room_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, room_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(tx);
    
    // Get Redis connection
    let mut redis = state.redis.get_async_connection().await.unwrap();
    
    // Frame processing task
    let process_frames = async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Binary(data) => {
                    if let Ok(ws_msg) = serde_json::from_slice::<WebSocketMessage>(&data) {
                        match ws_msg {
                            WebSocketMessage::Frame { timestamp, data, frame_type } => {
                                process_frame(
                                    &mut redis,
                                    &room_id,
                                    timestamp,
                                    data,
                                    frame_type
                                ).await;
                            }
                            WebSocketMessage::Control { action } => {
                                handle_control_action(action, &room_id, &state).await;
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = process_frames => {}
    }
}

async fn process_frame(
    redis: &mut redis::aio::Connection,
    room_id: &str,
    timestamp: i64,
    data: Vec<u8>,
    frame_type: FrameType,
) {
    match frame_type {
        FrameType::Video => {
            // Generate frame hash for deduplication
            let frame_hash = calculate_frame_hash(&data);
            
            // Check if this frame is a duplicate
            let key = format!("frame:{}:{}", room_id, frame_hash);
            let exists: bool = redis.exists(&key).await.unwrap_or(false);
            
            if !exists {
                // Store new frame
                let _: () = redis.set_ex(&key, timestamp, 3600).await.unwrap_or(());
                
                // Store frame in MinIO/S3
                store_frame(room_id, timestamp, &data).await;
            } else {
                // Update timestamp for duplicate frame
                let _: () = redis.set_ex(&key, timestamp, 3600).await.unwrap_or(());
            }
        }
        FrameType::Audio => {
            // Always store audio frames
            store_frame(room_id, timestamp, &data).await;
        }
    }
}

async fn handle_control_action(action: ControlAction, room_id: &str, state: &AppState) {
    match action {
        ControlAction::StartRecording => {
            let recording_id = Uuid::new_v4();
            sqlx::query!(
                "INSERT INTO recordings (id, room_id, start_time, status) 
                 VALUES ($1, $2, CURRENT_TIMESTAMP, 'Recording')",
                recording_id,
                Uuid::parse_str(room_id).unwrap(),
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        ControlAction::StopRecording => {
            sqlx::query!(
                "UPDATE recordings 
                 SET end_time = CURRENT_TIMESTAMP, status = 'Completed' 
                 WHERE room_id = $1 AND status = 'Recording'",
                Uuid::parse_str(room_id).unwrap(),
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        _ => {}
    }
}

fn calculate_frame_hash(data: &[u8]) -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish().to_string()
}

async fn store_frame(room_id: &str, timestamp: i64, data: &[u8]) {
    // TODO: Implement MinIO/S3 storage
    // This will be implemented in the storage module
} 