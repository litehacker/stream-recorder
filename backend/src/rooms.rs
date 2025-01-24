use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub max_participants: u32,
    pub recording_enabled: bool,
    pub current_participants: u32,
}

#[derive(Clone)]
pub struct Rooms {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    streams: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
}

impl Rooms {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_room(&self, id: String, name: String, max_participants: u32) -> Result<Room, AppError> {
        let mut rooms = self.rooms.write().unwrap();
        let room = Room {
            id: id.clone(),
            name,
            max_participants,
            recording_enabled: true,
            current_participants: 0,
        };
        rooms.insert(id, room.clone());
        Ok(room)
    }

    pub async fn list_rooms(&self) -> Result<Vec<Room>, AppError> {
        let rooms = self.rooms.read().unwrap();
        Ok(rooms.values().cloned().collect())
    }

    pub async fn get_room(&self, room_id: &str) -> Result<Room, AppError> {
        let rooms = self.rooms.read().unwrap();
        rooms
            .get(room_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Room {} not found", room_id)))
    }

    pub async fn get_stream(&self, room_id: &str) -> Result<broadcast::Sender<Vec<u8>>, AppError> {
        let streams = self.streams.read().unwrap();
        if let Some(tx) = streams.get(room_id) {
            Ok(tx.clone())
        } else {
            let mut streams = self.streams.write().unwrap();
            let (tx, _) = broadcast::channel(100);
            streams.insert(room_id.to_string(), tx.clone());
            Ok(tx)
        }
    }

    pub async fn add_participant(&self, room_id: &str) -> Result<(), AppError> {
        let mut rooms = self.rooms.write().unwrap();
        if let Some(room) = rooms.get_mut(room_id) {
            if room.current_participants >= room.max_participants {
                return Err(AppError::TooManyConnections(format!("Room {} is full", room_id)));
            }
            room.current_participants += 1;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Room {} not found", room_id)))
        }
    }

    pub async fn remove_participant(&self, room_id: &str) -> Result<(), AppError> {
        let mut rooms = self.rooms.write().unwrap();
        if let Some(room) = rooms.get_mut(room_id) {
            if room.current_participants > 0 {
                room.current_participants -= 1;
            }
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Room {} not found", room_id)))
        }
    }
} 