# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/app
COPY backend/ .
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /usr/local/bin
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /var/log/stream-recorder
COPY --from=builder /usr/src/app/target/release/stream-recorder .
CMD ["./stream-recorder"]
