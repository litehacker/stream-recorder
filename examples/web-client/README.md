# Stream Recorder Web Client

A React-based web client for the Stream Recorder platform.

## Features

- Real-time video/audio streaming
- WebRTC support
- Frame deduplication
- Performance monitoring
- Adaptive quality settings

## Getting Started

### Prerequisites

- Node.js 16+
- npm or yarn
- Stream Recorder backend running

### Installation

1. Install dependencies:

```bash
npm install
# or
yarn install
```

2. Start the development server:

```bash
npm start
# or
yarn start
```

3. Open http://localhost:3000 in your browser

## Usage

### Basic Recording

```typescript
import { StreamRecorder } from "./components/StreamRecorder";

function App() {
  const handleError = (error: Error) => {
    console.error("Stream error:", error);
  };

  return <StreamRecorder apiKey="your_api_key" onError={handleError} />;
}
```

### Advanced Configuration

```typescript
const config = {
  video_codec: "h264",
  audio_codec: "opus",
  max_bitrate: 2000000,
  frame_rate: 30,
  resolution: "1280x720",
  deduplication_enabled: true,
  adaptive_bitrate: true,
  enable_frame_batching: true,
  enable_hardware_acceleration: true,

  // Advanced settings
  keyframe_interval: 2,
  min_bitrate: 500000,
  quality_degradation_limit: 30,
  batch_size: 5,
  batch_timeout_ms: 100,
  max_buffer_size: 50,
  buffer_duration_ms: 500,
  enable_error_resilience: true,
  error_correction_level: 50,
  retry_attempts: 3,
};
```

## Performance Optimization

### Network Optimization

- Frame batching for efficient transmission
- Adaptive bitrate based on network conditions
- Error resilience with automatic retries

### Resource Management

- Hardware acceleration when available
- Efficient buffer management
- Memory usage optimization

### Quality Control

- Dynamic quality adjustment
- Frame rate adaptation
- Bitrate scaling

## Troubleshooting

### Common Issues

1. **Connection Failed**

   - Check if the backend server is running
   - Verify API key is correct
   - Check WebSocket connection

2. **Poor Performance**

   - Reduce resolution or frame rate
   - Enable hardware acceleration
   - Adjust batch settings

3. **High Latency**
   - Decrease buffer duration
   - Reduce batch timeout
   - Enable frame batching

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request
