use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub api_key: String,
    pub quota_limit: i64,
    pub quota_used: i64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub user_id: Uuid,
    pub room_id: String,
    pub config: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_active: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recording {
    pub id: Uuid,
    pub room_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub storage_path: String,
    pub size_bytes: i64,
    pub frame_count: i64,
    pub status: RecordingStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RecordingStatus {
    Recording,
    Completed,
    Failed,
    Processing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomConfig {
    pub room_id: Uuid,
    // Video settings
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub max_bitrate: Option<i32>,
    pub frame_rate: Option<i32>,
    pub resolution: Option<String>,
    
    // Deduplication settings
    pub deduplication_enabled: bool,
    pub deduplication_threshold: Option<f32>,
    
    // Advanced optimization settings
    pub keyframe_interval: Option<i32>,      // Keyframe interval in seconds
    pub adaptive_bitrate: bool,              // Enable adaptive bitrate
    pub min_bitrate: Option<i32>,           // Minimum bitrate for adaptation
    pub quality_degradation_limit: Option<i32>, // Max quality reduction percentage
    
    // Network optimization
    pub enable_frame_batching: bool,         // Enable frame batching
    pub batch_size: Option<i32>,             // Number of frames per batch
    pub batch_timeout_ms: Option<i32>,       // Max wait time for batch
    
    // Buffer settings
    pub max_buffer_size: Option<i32>,        // Maximum buffer size in MB
    pub buffer_duration_ms: Option<i32>,     // Buffer duration in milliseconds
    
    // Error resilience
    pub enable_error_resilience: bool,       // Enable error resilience
    pub error_correction_level: Option<i32>, // FEC strength (0-100)
    pub retry_attempts: Option<i32>,         // Number of retry attempts
    
    // Hardware acceleration
    pub enable_hardware_acceleration: bool,   // Use hardware encoding/decoding
    pub preferred_hardware_vendor: Option<String>, // Preferred hardware vendor
}

// Request/Response structs
#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub max_participants: Option<u32>,
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

#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct JoinRoomResponse {
    pub access_token: String,
    pub config: Option<RoomConfig>,
}

#[derive(Debug, Serialize)]
pub struct RecordingListResponse {
    pub recordings: Vec<Recording>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketMessage {
    Frame(Frame),
    Control(ControlAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    pub timestamp: i64,
    pub frame_type: FrameType,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FrameType {
    Video,
    Audio,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlAction {
    StartRecording,
    StopRecording,
    PauseRecording,
    ResumeRecording,
}

// Analytics models
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamMetrics {
    pub room_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub bytes_transferred: i64,
    pub frames_processed: i64,
    pub frames_deduplicated: i64,
    pub current_bitrate: i32,
    pub current_fps: f32,
    pub peak_memory_mb: i32,
}

#[derive(Debug, Serialize)]
pub struct RoomAnalytics {
    pub total_storage_used: i64,
    pub total_stream_time: i64,
    pub total_recordings: i64,
    pub avg_bitrate: i32,
    pub avg_fps: f32,
    pub deduplication_ratio: f32,
}

#[derive(Debug, Serialize)]
pub struct UserAnalytics {
    pub total_rooms: i64,
    pub total_storage_used: i64,
    pub total_stream_time: i64,
    pub quota_percentage: f32,
    pub rooms_analytics: Vec<RoomAnalytics>,
}