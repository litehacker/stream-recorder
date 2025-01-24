# stream-recorder

A high-performance video streaming and recording solution with WebRTC support.

## Implementation Progress

### ✓ Core Features

- [x] Authentication and API key management
- [x] Room creation and management
- [x] WebSocket streaming support
- [x] Frame deduplication using Redis
- [x] S3-compatible storage (MinIO)
- [x] Day-based recording slicing
- [x] Analytics system with Grafana

### ✓ Database Schema

- [x] Users and API keys
- [x] Rooms and configurations
- [x] Recordings management
- [x] Analytics and metrics
- [x] Materialized views for performance

### ✓ API Endpoints

- [x] `/auth` - Authentication
- [x] `/room` - Room management
- [x] `/room/:room_id/ws` - WebSocket streaming
- [x] `/room/:room_id/recordings` - Recording management
- [x] `/analytics/*` - Analytics endpoints

### ✓ Analytics Dashboard

- [x] Real-time metrics collection
- [x] Room and user analytics
- [x] Storage utilization tracking
- [x] Performance monitoring

## Getting Started

### Prerequisites

1. **Docker and Docker Compose**

   ```bash
   # For macOS
   # Install Docker Desktop from https://www.docker.com/products/docker-desktop
   # Docker Compose is included with Docker Desktop for Mac

   # For Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install docker.io docker-compose

   # For Windows
   # Download Docker Desktop from https://www.docker.com/products/docker-desktop
   ```

   After installing Docker Desktop on macOS:

   1. Launch Docker Desktop from Applications
   2. Wait for the Docker engine to start (whale icon in menu bar)
   3. Verify installation:
      ```bash
      docker --version
      docker compose version
      ```

2. **Rust (for development)**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Node.js and npm (for web client)**

   ```bash
   # Install nvm (Node Version Manager)
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash

   # Reload shell configuration
   export NVM_DIR="$HOME/.nvm"
   [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
   [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

   # Install and use LTS version of Node.js
   nvm install --lts
   nvm use --lts

   # Verify installation
   node --version
   npm --version
   ```

### Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/yourusername/stream-recorder.git
   cd stream-recorder
   ```

2. **Create necessary directories**:

   ```bash
   mkdir -p logs data/postgres data/redis data/minio
   ```

3. **Start the services**:

   ```bash
   # Build and start all services
   docker-compose up -d

   # Wait for services to be ready
   sleep 30

   # Initialize database
   ./scripts/setup.sh
   ```

4. **Generate API credentials**:

   ```bash
   # Generate API key
   curl -X POST http://localhost:3000/auth

   # Save the API key
   export API_KEY="your_api_key_from_response"
   ```

5. **Start Web Client (Development)**:

   ```bash
   # Navigate to web client directory
   cd examples/web-client

   # Install dependencies
   npm install

   # Start development server
   npm start
   ```

### Access Points

1. **Backend Services**:

   - API Server: http://localhost:3000
   - Grafana Dashboard: http://localhost:3001
   - MinIO Console: http://localhost:9000
     - Username: minioadmin
     - Password: minioadmin

2. **Monitoring**:
   - Prometheus: http://localhost:9090
   - Grafana: http://localhost:3001
     - Username: admin
     - Password: admin

### Quick Test

1. **Create a Room**:

   ```bash
   curl -X POST http://localhost:3000/room \
     -H "Content-Type: application/json" \
     -d '{
       "api_key": "'$API_KEY'",
       "config": {
         "video_codec": "h264",
         "frame_rate": 30,
         "deduplication_enabled": true
       }
     }'

   # Save the room_id
   export ROOM_ID="room_id_from_response"
   ```

2. **Check Health**:

   ```bash
   curl http://localhost:3000/health
   ```

3. **View Metrics**:
   ```bash
   curl http://localhost:3000/metrics
   ```

### Troubleshooting

1. **Check Service Status**:

   ```bash
   docker-compose ps
   ```

2. **View Service Logs**:

   ```bash
   # All services
   docker-compose logs

   # Specific service
   docker-compose logs -f app
   ```

3. **Common Issues**:

   - Database initialization fails:
     ```bash
     docker-compose down
     docker volume prune -f
     docker-compose up -d
     ./scripts/setup.sh
     ```
   - Web client connection fails:
     - Verify API key is correct
     - Check room ID exists
     - Ensure WebSocket endpoint is accessible
   - Metrics not appearing:
     - Check Prometheus is running
     - Verify metrics endpoint is accessible
     - Review Grafana data source configuration

4. **Reset Everything**:
   ```bash
   docker-compose down
   docker volume prune -f
   rm -rf data/*
   docker-compose up -d
   ./scripts/setup.sh
   ```

## Architecture

### Backend Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL
- **Cache**: Redis
- **Storage**: MinIO (S3-compatible)
- **Streaming**: WebSocket + LiveKit
- **Analytics**: Prometheus + Grafana

### Key Features

#### Frame Deduplication

- Real-time frame hashing
- Redis-based duplicate detection
- Configurable deduplication threshold
- Storage optimization

#### Storage Management

- Day-based recording slicing
- Efficient S3 storage organization
- Automatic cleanup policies
- Quota management

#### Analytics

- Real-time metrics collection
- Performance monitoring
- Usage analytics
- Storage utilization

## Next Steps

### In Progress

1. Embedded Device Support

   - Hardware optimization
   - Resource-constrained environments
   - Protocol adaptations

2. Additional Optimizations

   - Advanced frame compression
   - Adaptive bitrate streaming
   - Network resilience

3. Grafana Dashboard Configuration
   - Custom visualization panels
   - Alert configurations
   - User-specific views

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Development

### Environment Configuration

The application supports different environments (development, testing, production) with separate database configurations:

1. **Setup Environment**:

   ```bash
   # Copy example environment file
   cp .env.example .env

   # Edit .env file with your preferred settings
   nano .env
   ```

2. **Available Environments**:

   - Development (default):
     ```
     NODE_ENV=development
     POSTGRES_DB=streamrecorder_dev
     ```
   - Testing:
     ```
     NODE_ENV=testing
     POSTGRES_DB=streamrecorder_test
     ```
   - Production:
     ```
     NODE_ENV=production
     POSTGRES_DB=streamrecorder_prod
     POSTGRES_PASSWORD=your_secure_password
     ```

3. **Switch Environments**:
   ```bash
   # Edit .env file to change environment
   # Then restart services:
   docker-compose down
   docker-compose up -d
   ```

### Making Changes to the Backend

The backend is written in Rust, which is a compiled language. When you make changes to any Rust code in the `backend/` directory, you need to rebuild the service for changes to take effect:

```bash
# From the project root directory
cd docker
docker-compose up -d --build app
```

This will:

- Rebuild the backend service with your changes
- Restart the container with the new version
- Keep all other services running

Note: The `--build` flag is required to ensure Docker rebuilds the image instead of using a cached version.
