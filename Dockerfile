# Stage 1: Build Rust server
FROM rust:1.87-slim AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY engine/ engine/
COPY server/ server/
COPY client-wasm/ client-wasm/
RUN cargo build --release --bin penguin-soccer-server

# Stage 2: Build frontend
FROM node:22-slim AS web-builder
WORKDIR /app
COPY client-web/package*.json ./
RUN npm ci
COPY client-web/ ./
RUN npm run build

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/penguin-soccer-server ./
COPY --from=web-builder /app/dist ./public/
EXPOSE 3000
CMD ["./penguin-soccer-server"]
