use std::path::Path;
use tokio::fs;
use tracing::{info, error};
use crate::error::AppError;

#[derive(Clone)]
pub struct Storage {
    base_path: String,
}

impl Storage {
    pub async fn new() -> Result<Self, AppError> {
        let base_path = "data/recordings".to_string();
        fs::create_dir_all(&base_path).await.map_err(|e| {
            error!("Failed to create storage directory: {}", e);
            AppError::StorageError(e.to_string())
        })?;

        Ok(Self { base_path })
    }

    pub async fn save_recording(&self, room_id: &str, data: &[u8]) -> Result<String, AppError> {
        let room_dir = format!("{}/{}", self.base_path, room_id);
        fs::create_dir_all(&room_dir).await.map_err(|e| {
            error!("Failed to create room directory: {}", e);
            AppError::StorageError(e.to_string())
        })?;

        let timestamp = chrono::Utc::now().timestamp();
        let filename = format!("{}.mp4", timestamp);
        let path = format!("{}/{}", room_dir, filename);

        fs::write(&path, data).await.map_err(|e| {
            error!("Failed to write recording file: {}", e);
            AppError::StorageError(e.to_string())
        })?;

        info!("Saved recording: {}", path);
        Ok(filename)
    }

    pub async fn get_recording(&self, room_id: &str, filename: &str) -> Result<Vec<u8>, AppError> {
        let path = format!("{}/{}/{}", self.base_path, room_id, filename);
        fs::read(&path).await.map_err(|e| {
            error!("Failed to read recording file: {}", e);
            AppError::StorageError(e.to_string())
        })
    }

    pub async fn list_recordings(&self, room_id: &str) -> Result<Vec<String>, AppError> {
        let room_dir = format!("{}/{}", self.base_path, room_id);
        if !Path::new(&room_dir).exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&room_dir).await.map_err(|e| {
            error!("Failed to read room directory: {}", e);
            AppError::StorageError(e.to_string())
        })?;

        let mut recordings = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            AppError::StorageError(e.to_string())
        })? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".mp4") {
                    recordings.push(filename.to_string());
                }
            }
        }

        recordings.sort();
        Ok(recordings)
    }

    pub async fn delete_recording(&self, room_id: &str, filename: &str) -> Result<(), AppError> {
        let path = format!("{}/{}/{}", self.base_path, room_id, filename);
        fs::remove_file(&path).await.map_err(|e| {
            error!("Failed to delete recording file: {}", e);
            AppError::StorageError(e.to_string())
        })
    }

    pub async fn cleanup_old_recordings(&self, days: i64) -> Result<(), AppError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let mut entries = fs::read_dir(&self.base_path).await.map_err(|e| {
            error!("Failed to read storage directory: {}", e);
            AppError::StorageError(e.to_string())
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            AppError::StorageError(e.to_string())
        })? {
            if let Some(filename) = entry.file_name().to_str() {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        let modified: chrono::DateTime<chrono::Utc> = modified.into();
                        if modified < cutoff {
                            if let Err(e) = fs::remove_file(entry.path()).await {
                                error!("Failed to delete old recording: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
} 