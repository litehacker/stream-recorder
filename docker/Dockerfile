# Builder stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy only the dependency files first
COPY backend/Cargo.toml ./Cargo.toml
COPY backend/Cargo.lock ./Cargo.lock

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"dummy\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/stream_recorder*

# Now copy the actual source code
COPY backend/src ./src

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl-dev \
        && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/stream-recorder /usr/local/bin/

ENV RUST_LOG=info
EXPOSE 3000

CMD ["stream-recorder"]
