# Multi-stage Dockerfile for TitanClaw agent (cloud deployment).
#
# Build:
#   docker build --platform linux/amd64 -t titanclaw:latest .
#
# Run:
#   docker run --env-file .env -p 3000:3000 -v ./data:/data titanclaw:latest
#
# With Docker Compose:
#   docker-compose up -d

# Stage 1: Build
FROM rust:1.92-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev cmake gcc g++ \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Copy source and build artifacts
COPY src/ src/
COPY migrations/ migrations/
COPY wit/ wit/
COPY benchmarks/ benchmarks/

RUN cargo build --release --bin titanclaw

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl postgresql-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/titanclaw /usr/local/bin/titanclaw
COPY --from=builder /app/migrations /app/migrations

# Copy docker scripts
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY docker/healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/healthcheck.sh

# Create directories for data persistence (|| true to handle permission issues)
RUN mkdir -p /data /config && chmod -R 755 /data /config || true

# Non-root user
RUN useradd -m -u 1000 -s /bin/bash titanclaw
USER titanclaw

# Expose HTTP port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/healthcheck.sh

# Environment variables
ENV RUST_LOG=titanclaw=info
ENV DATA_DIR=/data
ENV CONFIG_DIR=/config

# Volume mounts for persistence
VOLUME ["/data", "/config"]

# Entrypoint with database initialization
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["titanclaw"]
