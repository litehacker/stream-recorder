/*
 * handlers/stream.rs
 * Purpose: WebSocket streaming handler implementation
 * 
 * This file contains:
 * - WebSocket connection handling and upgrade
 * - Stream message processing and broadcasting
 * - Room connection management
 * - Recording functionality for streams
 */

use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
};
use futures::{stream::StreamExt, SinkExt};
use tokio::sync::broadcast;
use tracing::{error, info};
use crate::{
    AppState,
    error::AppError,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Validate room exists
    let _room = state.rooms.get_room(&room_id).await?;

    // Check connection limits
    state.connection_tracker.check_limits(&room_id).await;

    // Get stream for room
    let tx = state.rooms.get_stream(&room_id).await?;

    info!("New WebSocket connection for room {}", room_id);

    // Upgrade connection
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, room_id, state, tx)))
}

async fn handle_socket(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState>,
    tx: broadcast::Sender<Vec<u8>>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(msg) => {
                    // Process message
                    if let Err(e) = process_message(msg, &room_id, &state).await {
                        error!("Error processing message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Handle outgoing messages
    let mut rx = tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if let Err(e) = sender.send(Message::Binary(msg)).await {
            error!("Error sending message: {}", e);
            break;
        }
    }
}

async fn process_message(msg: Message, room_id: &str, state: &AppState) -> Result<(), AppError> {
    match msg {
        Message::Binary(data) => {
            // Store frame
            state.storage.save_recording(room_id, &data).await?;
            Ok(())
        }
        Message::Close(_) => {
            info!("Client disconnected from room {}", room_id);
            Ok(())
        }
        _ => Ok(()),
    }
} 