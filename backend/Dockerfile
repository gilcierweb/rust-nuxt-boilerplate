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

# Cache dependencies: copy manifests first (layer caching).
# Stub all `src/main.rs` and `src/bin/*.rs` so `cargo build` can resolve every
# [[bin]] target declared in Cargo.toml (seed, cleanup_refresh_tokens, backend).
# Each stub is overwritten once the real src/ is COPYd below.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && \
    printf 'fn main() {}\n' > src/main.rs && \
    printf 'fn main() {}\n' > src/bin/seed.rs && \
    printf 'fn main() {}\n' > src/bin/cleanup_refresh_tokens.rs
RUN cargo build --release --bins
RUN rm -f target/release/deps/backend* target/release/deps/seed* target/release/deps/cleanup_refresh_tokens*

# Copy actual source
# `templates/` and `locales/` are referenced via include_str!/rust_i18n! macros
# at compile time, so they MUST be present in the build context alongside src/.
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY locales ./locales

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
RUN groupadd -r rust_nuxt_boilerplate && useradd -r -g rust_nuxt_boilerplate rust_nuxt_boilerplate

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/backend ./backend
# The diesel_cli was installed by `cargo install` in the builder stage into
# $CARGO_HOME/bin (which is /usr/local/cargo/bin for rust:official images —
# not /root/.cargo/bin). Use $CARGO_HOME so this works regardless of base
# image changes.
COPY --from=builder /usr/local/cargo/bin/diesel ./diesel
COPY --from=builder /app/migrations ./migrations
# `templates/` and `locales/` are NOT needed at runtime — they're bundled
# into the binary via `include_str!` / `rust_i18n!` at compile time. The
# production image only needs ./backend, ./diesel, and ./migrations.

# Ownership
RUN chown -R rust_nuxt_boilerplate:rust_nuxt_boilerplate /app
USER rust_nuxt_boilerplate

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Entrypoint wrapper:
# - `./docker-entrypoint.sh migrate` runs database migrations and exits.
# - `./docker-entrypoint.sh backend` (default) starts the HTTP server.
#
# Why an entrypoint wrapper:
# - Migrations no longer run on every container start (SECURITY_AUDIT.md I6):
#   the migration step is now a separate compose service ("migrate") that
#   must complete before the backend comes up.
# - The same image is reused for both the migration job and the running
#   service: the migration job runs `migrate`, then exits 0; the backend
#   runs `backend`.
# - In Kubernetes, this image is suitable for use as an initContainer that
#   runs `migrate`, plus a regular Deployment that runs `backend`.
COPY --chown=rust_nuxt_boilerplate:rust_nuxt_boilerplate docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod 755 /app/docker-entrypoint.sh

# Default: start the HTTP server.
ENTRYPOINT ["/app/docker-entrypoint.sh"]
CMD ["backend"]
