#!/bin/bash

# Wait for PostgreSQL to be ready
until PGPASSWORD=password psql -h localhost -U postgres -d streamrecorder -c '\q'; do
  echo "Postgres is unavailable - sleeping"
  sleep 1
done

# Create tables
PGPASSWORD=password psql -h localhost -U postgres -d streamrecorder << 'EOF'

-- Users table for client management
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    api_key TEXT UNIQUE NOT NULL,
    quota_limit BIGINT NOT NULL,
    quota_used BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Rooms table for managing streaming rooms
CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    room_id TEXT UNIQUE NOT NULL,
    config JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP WITH TIME ZONE
);

-- Recordings table for managing recorded sessions
CREATE TABLE IF NOT EXISTS recordings (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES rooms(id),
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    storage_path TEXT NOT NULL,
    size_bytes BIGINT DEFAULT 0,
    frame_count BIGINT DEFAULT 0,
    status TEXT NOT NULL
);

-- Room configurations for optimization settings
CREATE TABLE IF NOT EXISTS room_configs (
    room_id UUID PRIMARY KEY REFERENCES rooms(id),
    video_codec TEXT,
    audio_codec TEXT,
    max_bitrate INTEGER,
    frame_rate INTEGER,
    resolution TEXT,
    deduplication_enabled BOOLEAN DEFAULT true,
    deduplication_threshold FLOAT
);

-- Stream metrics for analytics
CREATE TABLE IF NOT EXISTS stream_metrics (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES rooms(id),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    bytes_transferred BIGINT NOT NULL,
    frames_processed BIGINT NOT NULL,
    frames_deduplicated BIGINT NOT NULL,
    current_bitrate INTEGER NOT NULL,
    current_fps FLOAT NOT NULL,
    peak_memory_mb INTEGER NOT NULL
);

-- Create indexes for analytics queries
CREATE INDEX IF NOT EXISTS idx_stream_metrics_room_time 
ON stream_metrics (room_id, timestamp);

CREATE INDEX IF NOT EXISTS idx_recordings_room_time 
ON recordings (room_id, start_time);

-- Create materialized view for room analytics
CREATE MATERIALIZED VIEW IF NOT EXISTS room_analytics AS
SELECT 
    r.id as room_id,
    r.user_id,
    COUNT(DISTINCT rec.id) as total_recordings,
    COALESCE(SUM(rec.size_bytes), 0) as total_storage_used,
    COALESCE(
        SUM(EXTRACT(EPOCH FROM (rec.end_time - rec.start_time))), 
        0
    ) as total_stream_time,
    COALESCE(AVG(sm.current_bitrate), 0) as avg_bitrate,
    COALESCE(AVG(sm.current_fps), 0) as avg_fps,
    CASE 
        WHEN SUM(sm.frames_processed) > 0 
        THEN CAST(SUM(sm.frames_deduplicated) AS FLOAT) / SUM(sm.frames_processed)
        ELSE 0 
    END as deduplication_ratio
FROM rooms r
LEFT JOIN recordings rec ON r.id = rec.room_id
LEFT JOIN stream_metrics sm ON r.id = sm.room_id
GROUP BY r.id, r.user_id;

-- Create function to refresh analytics
CREATE OR REPLACE FUNCTION refresh_analytics()
RETURNS TRIGGER AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY room_analytics;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create triggers to refresh analytics
CREATE TRIGGER refresh_analytics_on_recording
AFTER INSERT OR UPDATE ON recordings
FOR EACH STATEMENT
EXECUTE FUNCTION refresh_analytics();

CREATE TRIGGER refresh_analytics_on_metrics
AFTER INSERT ON stream_metrics
FOR EACH STATEMENT
EXECUTE FUNCTION refresh_analytics();

EOF

# Make the script executable
chmod +x setup.sh
