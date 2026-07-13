# -- Stage 1: Builder
FROM rust:1.95-slim-bookworm AS builder

# Install system dependencies for Diesel + OpenSSL
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install diesel_cli for migrations
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /app

# Cache dependencies: copy manifests first (layer caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/backend*

# Copy actual source
COPY src ./src
COPY migrations ./migrations

# Build release binary
RUN cargo build --release --bin backend

# -- Stage 2: Development (with hot-reload via cargo-watch)
FROM rust:1.95-slim-bookworm AS development

RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch diesel_cli --no-default-features --features postgres

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

EXPOSE 8080

# Run migrations then start with hot-reload
CMD diesel migration run && cargo watch -x "run --bin backend"

# -- Stage 3: Production Runtime
FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r contifya && useradd -r -g contifya contifya

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/backend ./backend
COPY --from=builder /root/.cargo/bin/diesel ./diesel
COPY --from=builder /app/migrations ./migrations

# Ownership
RUN chown -R contifya:contifya /app
USER contifya

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run migrations then start
CMD ./diesel migration run && ./backend
