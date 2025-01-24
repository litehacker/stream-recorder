pub async fn stream_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Check connection limits
    state.connection_tracker.check_limits(&room_id).await;

    // Upgrade the connection
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, room_id, state)))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: Arc<AppState>) {
    // ... existing code ...
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