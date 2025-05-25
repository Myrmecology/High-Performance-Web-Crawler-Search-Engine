# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -s /bin/bash crawler

# Copy the binaries from builder
COPY --from=builder /app/target/release/crawler /usr/local/bin/crawler
COPY --from=builder /app/target/release/search-server /usr/local/bin/search-server

# Copy configuration files
COPY config /app/config

# Create data directory
RUN mkdir -p /app/data && chown -R crawler:crawler /app

# Switch to app user
USER crawler
WORKDIR /app

# Expose the API port
EXPOSE 8080

# Default command
CMD ["search-server"]