# Stream Recorder API Documentation

## Authentication

### Generate API Credentials

```http
POST /auth
```

Response:

```json
{
  "id": "uuid",
  "api_key": "string",
  "quota_limit": "number",
  "quota_used": "number"
}
```

## Room Management

### Create Room

```http
POST /room
Content-Type: application/json

{
    "api_key": "string",
    "config": {
        "video_codec": "h264",
        "audio_codec": "opus",
        "max_bitrate": 2000000,
        "frame_rate": 30,
        "resolution": "1280x720",
        "deduplication_enabled": true,
        "deduplication_threshold": 0.95
    }
}
```

Response:

```json
{
  "room_id": "string",
  "access_token": "string"
}
```

### List Room Recordings

```http
GET /room/{room_id}/recordings
Authorization: Bearer {access_token}
```

Response:

```json
{
  "recordings": [
    {
      "id": "uuid",
      "start_time": "timestamp",
      "end_time": "timestamp",
      "size_bytes": "number",
      "frame_count": "number",
      "status": "string"
    }
  ]
}
```

## WebSocket Streaming

### Connect to Room

```http
GET /room/{room_id}/ws
Authorization: Bearer {access_token}
```

### WebSocket Messages

#### Frame Message

```json
{
  "type": "Frame",
  "timestamp": "number",
  "data": "binary",
  "frame_type": "Video|Audio"
}
```

#### Control Message

```json
{
  "type": "Control",
  "action": "StartRecording|StopRecording|PauseRecording|ResumeRecording"
}
```

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Rust 1.70+ (for development)

### Quick Start

1. Clone the repository:

```bash
git clone https://github.com/yourusername/stream-recorder.git
cd stream-recorder
```

2. Start the services:

```bash
docker-compose up -d
```

3. Initialize the database:

```bash
./scripts/setup.sh
```

4. Generate API credentials:

```bash
curl -X POST http://localhost:3000/auth
```

### Example: Recording a Stream

1. Create a room:

```bash
curl -X POST http://localhost:3000/room \
  -H "Content-Type: application/json" \
  -d '{
    "api_key": "your_api_key",
    "config": {
      "video_codec": "h264",
      "frame_rate": 30,
      "deduplication_enabled": true
    }
  }'
```

2. Connect to WebSocket and start streaming:

```javascript
const ws = new WebSocket("ws://localhost:3000/room/{room_id}/ws");

// Start recording
ws.send(
  JSON.stringify({
    type: "Control",
    action: "StartRecording",
  })
);

// Send video frame
ws.send(
  JSON.stringify({
    type: "Frame",
    timestamp: Date.now(),
    data: frameData,
    frame_type: "Video",
  })
);

// Stop recording
ws.send(
  JSON.stringify({
    type: "Control",
    action: "StopRecording",
  })
);
```

## Implementation Details

### Frame Deduplication

The system uses Redis for efficient frame deduplication:

1. Each frame is hashed
2. Hash is checked against Redis
3. Only unique frames are stored
4. Duplicate frames are referenced by timestamp

### Storage Organization

Recordings are stored in MinIO (S3-compatible) with the following structure:

```
{room_id}/{date}/{timestamp}.raw
```

### Performance Optimizations

- Frame deduplication using Redis
- Day-based storage slicing
- Configurable video/audio settings
- Efficient binary WebSocket communication

## Error Handling

All endpoints return standard HTTP status codes:

- 200: Success
- 400: Bad Request
- 401: Unauthorized
- 404: Not Found
- 500: Internal Server Error

Error responses include a message:

```json
{
  "error": "Error message"
}
```

## Analytics API

### Record Stream Metrics

```http
POST /analytics/metrics
Content-Type: application/json

{
    "room_id": "uuid",
    "timestamp": "2024-01-20T20:00:00Z",
    "bytes_transferred": 1000000,
    "frames_processed": 1000,
    "frames_deduplicated": 200,
    "current_bitrate": 2000000,
    "current_fps": 30.0,
    "peak_memory_mb": 512
}
```

### Get Room Analytics

```http
GET /analytics/room/{room_id}
Authorization: Bearer {access_token}
```

Response:

```json
{
  "total_storage_used": 1000000000,
  "total_stream_time": 3600,
  "total_recordings": 10,
  "avg_bitrate": 2000000,
  "avg_fps": 29.97,
  "deduplication_ratio": 0.2
}
```

### Get User Analytics

```http
GET /analytics/user/{user_id}
Authorization: Bearer {access_token}
```

Response:

```json
{
  "total_rooms": 5,
  "total_storage_used": 5000000000,
  "total_stream_time": 18000,
  "quota_percentage": 50.0,
  "rooms_analytics": [
    {
      "total_storage_used": 1000000000,
      "total_stream_time": 3600,
      "total_recordings": 10,
      "avg_bitrate": 2000000,
      "avg_fps": 29.97,
      "deduplication_ratio": 0.2
    }
  ]
}
```

## Analytics Dashboard

The analytics dashboard is powered by Grafana and provides real-time insights into:

1. System Performance

   - CPU and memory usage
   - Network bandwidth
   - Storage utilization

2. Stream Analytics

   - Active streams
   - Frame rates
   - Bitrates
   - Deduplication efficiency

3. User Metrics
   - Storage usage
   - Quota utilization
   - Active rooms
   - Total stream time

### Accessing the Dashboard

1. Access Grafana at `http://localhost:3000/grafana`
2. Default credentials:
   - Username: admin
   - Password: admin

### Available Dashboards

1. System Overview

   - System-wide metrics
   - Resource utilization
   - Storage statistics

2. Room Analytics

   - Per-room performance metrics
   - Stream quality indicators
   - Recording statistics

3. User Analytics
   - User quotas and usage
   - Room utilization
   - Storage trends

### Metrics Collection

The system automatically collects metrics at different levels:

1. System Metrics

   - Collected every 15 seconds
   - Resource utilization
   - Network stats

2. Stream Metrics

   - Collected in real-time
   - Frame processing stats
   - Quality indicators

3. Storage Metrics
   - Updated on recording events
   - Space utilization
   - Deduplication efficiency
