# Embedded Device Integration Guide

This guide explains how to connect embedded devices to the Stream Recorder platform.

## Supported Protocols

1. WebSocket (Primary)

   - Lightweight binary protocol
   - Low latency
   - Bi-directional communication

2. MQTT (Alternative)

   - Pub/sub architecture
   - Better for constrained networks
   - QoS levels support

3. RTSP (Legacy Devices)
   - Standard streaming protocol
   - Wide device compatibility
   - Automatic transcoding

## Resource Optimization

### Memory Usage

```rust
// Example configuration for memory-constrained devices
let config = RoomConfig {
    video_codec: Some("h264".to_string()),
    audio_codec: Some("opus".to_string()),
    max_bitrate: Some(500_000),  // 500 Kbps
    frame_rate: Some(15),        // Reduced frame rate
    resolution: Some("640x480".to_string()),
    deduplication_enabled: true,
    deduplication_threshold: Some(0.98),  // More aggressive deduplication
};
```

### Network Bandwidth

```rust
// Example frame batching for network efficiency
let batch_size = 5;
let mut frame_batch = Vec::with_capacity(batch_size);

for frame in frames.take(batch_size) {
    frame_batch.push(frame);
}

// Send batch in single WebSocket message
ws.send(Message::Binary(serialize_batch(&frame_batch))).await?;
```

## Example Implementations

### 1. Raspberry Pi (Python)

```python
import asyncio
import websockets
import json
import cv2
from datetime import datetime

async def connect_stream():
    # Connect to stream-recorder
    uri = "ws://localhost:3000/room/{room_id}/ws"
    async with websockets.connect(uri) as ws:
        # Initialize camera
        cap = cv2.VideoCapture(0)
        cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
        cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)

        # Start recording
        await ws.send(json.dumps({
            "type": "Control",
            "action": "StartRecording"
        }))

        while True:
            ret, frame = cap.read()
            if ret:
                # Compress frame
                _, buffer = cv2.imencode('.jpg', frame, [
                    cv2.IMWRITE_JPEG_QUALITY, 80
                ])

                # Send frame
                await ws.send(json.dumps({
                    "type": "Frame",
                    "timestamp": int(datetime.now().timestamp() * 1000),
                    "data": buffer.tobytes(),
                    "frame_type": "Video"
                }))

            await asyncio.sleep(1/15)  # 15 FPS

asyncio.run(connect_stream())
```

### 2. ESP32 (Arduino)

```cpp
#include <WiFi.h>
#include <WebSocketsClient.h>
#include <ArduinoJson.h>

WebSocketsClient webSocket;

void setup() {
    WiFi.begin("SSID", "PASSWORD");

    // Connect to stream-recorder
    webSocket.begin("localhost", 3000, "/room/{room_id}/ws");
    webSocket.onEvent(webSocketEvent);

    // Configure camera
    camera_config_t config;
    config.frame_size = FRAMESIZE_VGA;
    config.jpeg_quality = 12;
    config.fb_count = 2;
    esp_camera_init(&config);
}

void loop() {
    if (WiFi.status() == WL_CONNECTED) {
        camera_fb_t * fb = esp_camera_fb_get();
        if (fb) {
            // Create frame message
            StaticJsonDocument<200> doc;
            doc["type"] = "Frame";
            doc["timestamp"] = millis();
            doc["frame_type"] = "Video";

            // Send frame
            webSocket.sendBIN(fb->buf, fb->len);
            esp_camera_fb_return(fb);
        }
        delay(66); // ~15 FPS
    }
    webSocket.loop();
}
```

### 3. RTSP Camera Integration

```python
import asyncio
import cv2
import websockets
import json
from datetime import datetime

async def rtsp_to_websocket():
    # Connect to RTSP stream
    rtsp_url = "rtsp://camera_ip:554/stream"
    cap = cv2.VideoCapture(rtsp_url)

    # Connect to stream-recorder
    uri = "ws://localhost:3000/room/{room_id}/ws"
    async with websockets.connect(uri) as ws:
        while True:
            ret, frame = cap.read()
            if ret:
                # Resize and compress
                frame = cv2.resize(frame, (640, 480))
                _, buffer = cv2.imencode('.jpg', frame, [
                    cv2.IMWRITE_JPEG_QUALITY, 80
                ])

                # Send frame
                await ws.send(json.dumps({
                    "type": "Frame",
                    "timestamp": int(datetime.now().timestamp() * 1000),
                    "data": buffer.tobytes(),
                    "frame_type": "Video"
                }))

            await asyncio.sleep(1/15)

asyncio.run(rtsp_to_websocket())
```

## Best Practices

1. **Resource Management**

   - Use appropriate buffer sizes
   - Implement frame skipping under load
   - Monitor memory usage

2. **Error Handling**

   - Implement reconnection logic
   - Handle network interruptions
   - Buffer important frames

3. **Performance Optimization**

   - Use hardware acceleration when available
   - Implement frame batching
   - Adaptive quality based on network

4. **Security**
   - Secure storage of credentials
   - TLS for all connections
   - Regular token rotation

## Troubleshooting

1. **High Latency**

   - Reduce frame rate
   - Lower resolution
   - Enable frame batching

2. **Memory Issues**

   - Decrease buffer sizes
   - Enable aggressive frame skipping
   - Monitor heap fragmentation

3. **Network Problems**
   - Implement exponential backoff
   - Use QoS for critical frames
   - Enable connection pooling
