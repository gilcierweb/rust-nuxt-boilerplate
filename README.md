# Rust + Nuxt Boilerplate

> **Production-ready full-stack boilerplate** - Rust (Actix Web) + Nuxt 4 with authentication, RBAC, admin panel, type-safe database layer, and modern developer experience.

[![Rust](https://img.shields.io/badge/Rust-1.95-orange?logo=rust)](https://www.rust-lang.org/)
[![Actix Web](https://img.shields.io/badge/Actix%20Web-4.13-blue)](https://actix.rs/)
[![Nuxt](https://img.shields.io/badge/Nuxt-4.4-green?logo=nuxt.js)](https://nuxt.com/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-4.2-teal?logo=tailwindcss)](https://tailwindcss.com/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-blue?logo=postgresql)](https://www.postgresql.org/)
[![Docker](https://img.shields.io/badge/Docker-ready-blue?logo=docker)](https://www.docker.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## ✨ Features

### Backend (Rust / Actix Web)
- **Authentication**: JWT access/refresh tokens, Paseto for email verification, PBKDF2 password hashing, TOTP 2FA, OAuth2 ready
- **Authorization**: RBAC with resource-level permissions, role hierarchies, grant-based authorization (actix-web-grants)
- **Database**: Diesel ORM with compile-time SQL verification, migrations, connection pooling, repository pattern
- **API Docs**: OpenAPI 3.1 (utoipa) → TypeScript client generation, Swagger UI + Scalar
- **Security**: Field-level encryption, blind indexes for PII, key rotation, CSRF protection, rate limiting
- **Observability**: Structured JSON logging (tracing), Prometheus metrics, Grafana dashboards, request tracing
- **Email**: Resend integration, template system, queue-based sending
- **Real-time**: WebSocket server with Redis pub/sub

### Frontend (Nuxt 4 / Vue 3)
- **Type-safe API**: Auto-generated TypeScript client from OpenAPI spec
- **Forms**: vee-validate + valibot, i18n-ready validation
- **UI**: FlyonUI + Tailwind CSS 4, Material Design 3 tokens, dark mode
- **Admin Panel**: User management, role editor, audit logs, metrics dashboard
- **User Portal**: Dashboard, support tickets, profile management
- **Auth Pages**: Login, register, email confirmation, password reset, 2FA setup
- **Internationalization**: @nuxtjs/i18n (pt-BR, en, es)

### DevOps & Infrastructure
- **Multi-stage Docker**: Optimized dev/prod images for both services
- **Docker Compose**: Full stack with hot-reload (dev) and production profiles
- **Nginx**: Reverse proxy with TLS termination, security headers
- **Monitoring**: Prometheus + Grafana (pre-configured dashboards)
- **Search**: MeiliSearch for full-text search
- **Container Mgmt**: Portainer UI
- **CI/CD Ready**: GitHub Actions compatible, health checks, graceful shutdown

---

## 🚀 Quick Start

### Prerequisites
- Docker 24+ & Docker Compose 2.20+
- 4GB+ RAM available
- Ports free: 3000, 8080, 5432, 6379, 7700, 9000, 9090, 3001

### 1. Clone & Configure

```bash
git clone https://github.com/your-org/rust-nuxt-boilerplate.git
cd rust-nuxt-boilerplate

# Generate secure secrets
./scripts/generate-secrets.sh

# commands diesel cli
diesel database reset
diesel migration run
diesel migration redo
diesel migration revert

# Review generated .env file
cat .env
```

### 2. Start Development Stack

```bash
# Build and start all services with hot-reload
docker compose up --build

# Or run in background
docker compose up -d --build
```

### 3. Initialize Database

```bash
# Run migrations (auto-runs on container start, but can run manually)
docker compose exec backend diesel migration run

# Seed demo data (admin user, roles, permissions)
docker compose exec backend cargo run --bin seed
```

### 4. Access Services

| Service | URL | Credentials |
|---------|-----|-------------|
| **Frontend** | http://localhost:3000 | - |
| **Backend API** | http://localhost:8080 | - |
| **API Docs (Swagger)** | http://localhost:8080/swagger-ui | - |
| **API Docs (Scalar)** | http://localhost:8080/scalar | - |
| **Health Check** | http://localhost:8080/health | - |
| **Grafana** | http://localhost:3001 | admin / (from .env) |
| **Prometheus** | http://localhost:9090 | - |
| **MeiliSearch** | http://localhost:7700 | master key from .env |
| **Portainer** | http://localhost:9000 | - |

**Default Admin** (after seeding):
- Email: `admin@example.com`
- Password: `changeme123` ⚠️ **Change immediately!**

### API Endpoints Reference

| Category | Endpoint | Method | Auth | Description |
|----------|----------|--------|------|-------------|
| **Auth** | `/api/v1/auth/register` | POST | No | Register new user |
| | `/api/v1/auth/login` | POST | No | Login (returns JWT + sets refresh cookie) |
| | `/api/v1/auth/refresh` | POST | No | Refresh access token (via cookie) |
| | `/api/v1/auth/logout` | POST | Yes | Invalidate refresh token |
| | `/api/v1/auth/recover` | POST | No | Request password reset |
| | `/api/v1/auth/reset` | POST | No | Reset password with token |
| | `/api/v1/auth/confirm` | GET | No | Verify email with token |
| | `/api/v1/auth/session` | GET | Yes | Get current session user |
| | `/api/v1/auth/2fa/setup` | POST | Yes | Setup TOTP 2FA |
| | `/api/v1/auth/2fa/enable` | POST | Yes | Enable TOTP 2FA |
| | `/api/v1/auth/2fa/disable` | POST | Yes | Disable TOTP 2FA |
| | `/api/v1/auth/2fa/verify` | POST | Yes | Verify TOTP code |
| | `/api/v1/auth/change-password` | POST | Yes | Change password |
| **Admin** | `/api/v1/admin/users` | GET | Admin | List users (paginated) |
| | `/api/v1/admin/users` | POST | Admin | Create user |
| | `/api/v1/admin/users/{id}` | GET | Admin | Get user |
| | `/api/v1/admin/users/{id}` | PATCH | Admin | Update user |
| | `/api/v1/admin/users/{id}` | DELETE | Admin | Delete user |
| | `/api/v1/admin/roles` | GET | Admin | List roles |
| | `/api/v1/admin/roles` | POST | Admin | Create role |
| | `/api/v1/admin/roles/{id}` | GET | Admin | Get role |
| | `/api/v1/admin/roles/{id}` | PATCH | Admin | Update role |
| | `/api/v1/admin/roles/{id}` | DELETE | Admin | Delete role |
| | `/api/v1/admin/audit-logs` | GET | Admin | List audit logs (paginated) |
| | `/api/v1/admin/audit-logs` | POST | Admin | Create audit log |
| | `/api/v1/admin/audit-logs/{id}` | GET | Admin | Get audit log |
| **Health** | `/health` | GET | No | Service health |
| | `/metrics` | GET | No | Prometheus metrics |

---

## 🛠️ Local Development (Without Docker)

### Backend

```bash
cd backend

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install tools
cargo install diesel_cli --no-default-features --features postgres
cargo install cargo-watch

# Configure environment
cp ../.env.example .env
# Edit .env with local DB/Redis credentials

# Run migrations
diesel migration run

# Start with hot-reload
cargo watch -x "run --bin backend"

# Or run seed directly
cargo run --bin seed
```

**Backend:** http://localhost:8080

### Frontend

```bash
cd frontend

# Enable corepack (for pnpm)
corepack enable

# Install dependencies
pnpm install

# Start dev server
pnpm dev
```

**Frontend:** http://localhost:3000

---

## 🔐 Environment Variables

All configuration via `.env` (from `.env.example`). **Never commit `.env` to git.**

### Generate Secrets

```bash
# Automated (recommended)
./scripts/generate-secrets.sh

# Manual
# JWT Secret (64 chars base64)
openssl rand -base64 48

# Database/Redis passwords
openssl rand -base64 24 | tr -d '/+=' | cut -c1-24

# MeiliSearch / Grafana
openssl rand -base64 16
```

### Required Variables

| Variable | Description | Required |
|----------|-------------|:--------:|
| `ENVIRONMENT` | `development` \| `staging` \| `production` | ✅ |
| `DATABASE_URL` | PostgreSQL connection string | ✅ |
| `REDIS_URL` | Redis connection string | ✅ |
| `JWT_SECRET` | 64-char base64 for JWT signing | ✅ |
| `MASTER_KEY` | Base64 encryption key (32 bytes) | ✅ (prod) |
| `BLIND_INDEX_KEY` | Base64 blind index key (32 bytes) | ✅ (prod) |
| `FRONTEND_URL` | CORS origin for frontend | ✅ |

See [`.env.example`](.env.example) for complete list.

---

## 🔧 Backend & Frontend Environment Setup (Step by Step)

### Backend Environment Variables

Create `backend/.env` (or use root `.env` with Docker Compose):

```bash
# ──────────────────────────────────────────────
# APPLICATION
# ──────────────────────────────────────────────
ENVIRONMENT=development                 # development | staging | production
FRONTEND_URL=http://localhost:3000

# ──────────────────────────────────────────────
# SERVER
# ──────────────────────────────────────────────
HOST=0.0.0.0
PORT=8080
HTTPS_PORT=8443
TLS_CERT_PATH=./infra/ssl/cert.pem
TLS_KEY_PATH=./infra/ssl/key.pem

# ──────────────────────────────────────────────
# DATABASE (PostgreSQL)
# ──────────────────────────────────────────────
POSTGRES_USER=boilerplate
POSTGRES_PASSWORD=changeme_secure_password
POSTGRES_DB=boilerplate_dev
DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB}

# Connection Pool Settings (Diesel r2d2)
# Tune these for high-concurrency scenarios
DB_POOL_SIZE=10                          # Max connections (default: 10)
DB_POOL_MIN_IDLE=2                       # Min idle connections (optional)
DB_POOL_MAX_LIFETIME_SECS=1800           # Max connection lifetime: 30 min (optional)
DB_POOL_IDLE_TIMEOUT_SECS=600            # Idle timeout: 10 min (optional)
DB_POOL_CONNECTION_TIMEOUT_SECS=10       # Connection timeout: 10 sec (default: 10)

# High-concurrency example (uncomment and adjust for production):
# DB_POOL_SIZE=50
# DB_POOL_MIN_IDLE=10
# DB_POOL_MAX_LIFETIME_SECS=3600
# DB_POOL_IDLE_TIMEOUT_SECS=300
# DB_POOL_CONNECTION_TIMEOUT_SECS=30

# ──────────────────────────────────────────────
# REDIS
# ──────────────────────────────────────────────
REDIS_PASSWORD=changeme_redis_password
REDIS_URL=redis://:${REDIS_PASSWORD}@redis:6379

# Connection Pool Settings
REDIS_POOL_SIZE=10                       # Max connections (default: 10)

# High-concurrency example (uncomment and adjust for production):
# REDIS_POOL_SIZE=50

# ──────────────────────────────────────────────
# JWT / AUTHENTICATION
# ──────────────────────────────────────────────
# Generate with: openssl rand -base64 48
JWT_SECRET=REPLACE_WITH_GENERATED_64_CHAR_BASE64_SECRET
JWT_ACCESS_EXPIRY_SECS=900           # 15 minutes
JWT_REFRESH_EXPIRY_SECS=2592000      # 30 days

# ──────────────────────────────────────────────
# SECURITY / ENCRYPTION
# ──────────────────────────────────────────────
# Master key for field-level encryption (AES-GCM, 32 bytes base64)
# Generate with: openssl rand -base64 32
MASTER_ENCRYPTION_KEY=REPLACE_WITH_GENERATED_32_BYTE_BASE64_KEY

# Blind index key for PII search (32 bytes base64)
# Generate with: openssl rand -base64 32
BLIND_INDEX_KEY=REPLACE_WITH_GENERATED_32_BYTE_BASE64_KEY

CURRENT_ENCRYPTION_KEY_VERSION=1

# Internal API keys (comma-separated) for service-to-service auth
INTERNAL_API_KEYS=

# ──────────────────────────────────────────────
# EMAIL (Resend)
# ──────────────────────────────────────────────
RESEND_API_KEY=
EMAIL_FROM=noreply@boilerplate-rust-nuxt.com
EMAIL_FROM_NAME=Boilerplate Rust Nuxt

# SMTP fallback
SMTP_HOST=
SMTP_PORT=587
SMTP_USER=
SMTP_PASSWORD=
SMTP_FROM=

# ──────────────────────────────────────────────
# BUNNY.NET (CDN / Storage / Stream)
# ──────────────────────────────────────────────
BUNNY_STORAGE_ZONE=
BUNNY_STORAGE_KEY=
BUNNY_CDN_URL=https://cdn.boilerplate-rust-nuxt.com
BUNNY_TOKEN_KEY=

BUNNY_STREAM_LIBRARY_ID=
BUNNY_STREAM_KEY=
BUNNY_STREAM_WEBHOOK_SECRET=

# ──────────────────────────────────────────────
# BACKBLAZE B2 (Alternative Storage)
# ──────────────────────────────────────────────
B2_KEY_ID=
B2_APPLICATION_KEY=
B2_BUCKET_ID=
B2_BUCKET_NAME=
B2_ENDPOINT=https://s3.us-west-004.backblazeb2.com

# ──────────────────────────────────────────────
# STRIPE (Payments / Subscriptions)
# ──────────────────────────────────────────────
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
STRIPE_PUBLISHABLE_KEY=

PLATFORM_COMMISSION_PERCENT=20.0
MIN_SUBSCRIPTION_PRICE_CENTS=500
MAX_SUBSCRIPTION_PRICE_CENTS=50000
MIN_WITHDRAWAL_AMOUNT_CENTS=2000

# ──────────────────────────────────────────────
# 2FA / TOTP
# ──────────────────────────────────────────────
TOTP_ISSUER=Boilerplate Rust Nuxt

# ──────────────────────────────────────────────
# MEILISEARCH
# ──────────────────────────────────────────────
MEILI_MASTER_KEY=REPLACE_WITH_GENERATED_KEY
MEILI_URL=http://meilisearch:7700

# ──────────────────────────────────────────────
# OBSERVABILITY
# ──────────────────────────────────────────────
GRAFANA_PASSWORD=REPLACE_WITH_GENERATED_PASSWORD

# ──────────────────────────────────────────────
# FILE UPLOAD LIMITS
# ──────────────────────────────────────────────
MAX_VIDEO_SIZE_BYTES=10737418240    # 10 GB
MAX_PHOTO_SIZE_BYTES=52428800       # 50 MB
MAX_AUDIO_SIZE_BYTES=524288000      # 500 MB
```

#### Generate Backend Secrets

```bash
# JWT Secret (64 chars base64 = 48 bytes entropy)
openssl rand -base64 48

# Database/Redis passwords (24 chars base64 = 18 bytes)
openssl rand -base64 24 | tr -d '/+=' | cut -c1-24

# Master Encryption Key (32 bytes base64 = 256 bits for AES-GCM)
openssl rand -base64 32

# Blind Index Key (32 bytes base64)
openssl rand -base64 32

# MeiliSearch Master Key
openssl rand -base64 24

# Grafana Admin Password
openssl rand -base64 16

# CSRF Secret (hex, 32 bytes = 64 chars)
openssl rand -hex 32
```

---

### Frontend Environment Variables

Create `frontend/.env` or set in Nuxt config:

```bash
# ──────────────────────────────────────────────
# BACKEND API
# ──────────────────────────────────────────────
# Base URL for backend API (used by Nitro proxy & generated client)
NUXT_PUBLIC_API_BASE=http://localhost:8080/api/v1

# Direct backend API URL for client-side (CSR) requests
# When set, client-side fetches will go directly to the backend instead of
# going through the Nitro proxy. This reduces latency for non-SSR requests.
# The Nitro proxy is still used for SSR fetches to attach cookies/headers.
# Leave empty to use default proxy behavior for all requests.
# Example: http://localhost:8080/api/v1
NUXT_PUBLIC_API_BASE=http://localhost:8080/api/v1

# ──────────────────────────────────────────────
# CSRF PROTECTION (must match backend)
# ──────────────────────────────────────────────
# Generate with: openssl rand -hex 32
CSRF_SECRET_KEY=REPLACE_WITH_GENERATED_CSRF_SECRET

# ──────────────────────────────────────────────
# BACKEND API KEY (for server-to-server calls from frontend)
# ──────────────────────────────────────────────
# Must match one of INTERNAL_API_KEYS in backend
BACKEND_API_KEY=REPLACE_WITH_GENERATED_API_KEY

# ──────────────────────────────────────────────
# APP CONFIG
# ──────────────────────────────────────────────
NUXT_PUBLIC_APP_NAME=RustNuxt Boilerplate
NUXT_PUBLIC_APP_URL=http://localhost:3000

# ──────────────────────────────────────────────
# FEATURE FLAGS
# ──────────────────────────────────────────────
NUXT_PUBLIC_ENABLE_2FA=true
NUXT_PUBLIC_ENABLE_PWA=true
```

#### Frontend Required Variables

| Variable | Description | Required |
|----------|-------------|:--------:|
| `NUXT_PUBLIC_API_BASE` | Backend API base URL | ✅ |
| `CSRF_SECRET_KEY` | CSRF secret (must match backend) | ✅ |
| `BACKEND_API_KEY` | Internal API key for server-to-server calls | ✅ |
| `NUXT_PUBLIC_APP_NAME` | Display name | - |
| `NUXT_PUBLIC_APP_URL` | Frontend URL | - |

---

### CSRF Configuration

**Backend** (`backend/.env` or root `.env`):
```bash
export CSRF_SECRET_KEY=REPLACE_WITH_GENERATED_CSRF_SECRET
```

**Frontend** (`frontend/.env`):
```bash
export CSRF_SECRET_KEY=REPLACE_WITH_GENERATED_CSRF_SECRET
```

**Exclusions** (paths that skip CSRF validation):
- Auth endpoints: `/api/v1/auth/*`
- Webhooks: `/api/v1/webhooks/*`
- WebSocket: `/api/v1/ws/*`
- Swagger/Scalar docs: `/swagger-ui`, `/scalar`, `/openapi.json`

---

### Quick Setup Script

```bash
#!/usr/bin/env bash
# setup-env.sh - Run from project root

set -euo pipefail

echo "🔐 Generating secrets..."

# Backend secrets
JWT_SECRET=$(openssl rand -base64 48)
POSTGRES_PASSWORD=$(openssl rand -base64 24 | tr -d '/+=' | cut -c1-24)
REDIS_PASSWORD=$(openssl rand -base64 24 | tr -d '/+=' | cut -c1-24)
MASTER_KEY=$(openssl rand -base64 32)
BLIND_INDEX_KEY=$(openssl rand -base64 32)
MEILI_MASTER_KEY=$(openssl rand -base64 24)
GRAFANA_PASSWORD=$(openssl rand -base64 16)
CSRF_SECRET=$(openssl rand -hex 32)
INTERNAL_API_KEY=$(openssl rand -hex 32)
BACKEND_API_KEY=$(openssl rand -hex 32)

cat > .env <<EOF
# ──────────────────────────────────────────────
# GENERATED - $(date -u +"%Y-%m-%d %H:%M:%S UTC")
# ──────────────────────────────────────────────

# APPLICATION
ENVIRONMENT=development
FRONTEND_URL=http://localhost:3000

# SERVER
HOST=0.0.0.0
PORT=8080
HTTPS_PORT=8443
TLS_CERT_PATH=./infra/ssl/cert.pem
TLS_KEY_PATH=./infra/ssl/key.pem

# DATABASE
POSTGRES_USER=boilerplate
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=boilerplate_dev
DATABASE_URL=postgres://boilerplate:${POSTGRES_PASSWORD}@postgres:5432/boilerplate_dev

# Connection Pool Settings
DB_POOL_SIZE=10
DB_POOL_MIN_IDLE=2
DB_POOL_MAX_LIFETIME_SECS=1800
DB_POOL_IDLE_TIMEOUT_SECS=600
DB_POOL_CONNECTION_TIMEOUT_SECS=10

# REDIS
REDIS_PASSWORD=${REDIS_PASSWORD}
REDIS_URL=redis://:${REDIS_PASSWORD}@redis:6379
REDIS_POOL_SIZE=10

# JWT / AUTH
JWT_SECRET=${JWT_SECRET}
JWT_ACCESS_EXPIRY_SECS=900
JWT_REFRESH_EXPIRY_SECS=2592000

# SECURITY / ENCRYPTION
MASTER_ENCRYPTION_KEY=${MASTER_KEY}
BLIND_INDEX_KEY=${BLIND_INDEX_KEY}
CURRENT_ENCRYPTION_KEY_VERSION=1
INTERNAL_API_KEYS=${INTERNAL_API_KEY}

# EMAIL
RESEND_API_KEY=
EMAIL_FROM=noreply@boilerplate-rust-nuxt.com
EMAIL_FROM_NAME=Boilerplate Rust Nuxt

# BUNNY.NET
BUNNY_STORAGE_ZONE=
BUNNY_STORAGE_KEY=
BUNNY_CDN_URL=https://cdn.boilerplate-rust-nuxt.com
BUNNY_TOKEN_KEY=${CSRF_SECRET}

# STRIPE
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
STRIPE_PUBLISHABLE_KEY=

# MEILISEARCH
MEILI_MASTER_KEY=${MEILI_MASTER_KEY}
MEILI_URL=http://meilisearch:7700

# OBSERVABILITY
GRAFANA_PASSWORD=${GRAFANA_PASSWORD}

# FILE UPLOAD LIMITS
MAX_VIDEO_SIZE_BYTES=10737418240
MAX_PHOTO_SIZE_BYTES=52428800
MAX_AUDIO_SIZE_BYTES=524288000

# CSRF (shared)
CSRF_SECRET_KEY=${CSRF_SECRET}

# BACKEND API KEY (for frontend server calls)
BACKEND_API_KEY=${BACKEND_API_KEY}
EOF

# Frontend .env
cat > frontend/.env <<EOF
# Generated $(date -u +"%Y-%m-%d %H:%M:%S UTC")

# Backend API
NUXT_PUBLIC_API_BASE=http://localhost:8080/api/v1

# CSRF (must match backend)
CSRF_SECRET_KEY=${CSRF_SECRET}

# Backend API Key (for server-to-server)
BACKEND_API_KEY=${BACKEND_API_KEY}

# App Config
NUXT_PUBLIC_APP_NAME=RustNuxt Boilerplate
NUXT_PUBLIC_APP_URL=http://localhost:3000

# Features
NUXT_PUBLIC_ENABLE_2FA=true
NUXT_PUBLIC_ENABLE_PWA=true
EOF

echo "✅ .env files created"
echo ""
echo "📋 Generated values (save these!):"
echo "  JWT_SECRET:           ${JWT_SECRET}"
echo "  POSTGRES_PASSWORD:    ${POSTGRES_PASSWORD}"
echo "  REDIS_PASSWORD:       ${REDIS_PASSWORD}"
echo "  MASTER_KEY:           ${MASTER_KEY}"
echo "  BLIND_INDEX_KEY:      ${BLIND_INDEX_KEY}"
echo "  MEILI_MASTER_KEY:     ${MEILI_MASTER_KEY}"
echo "  GRAFANA_PASSWORD:     ${GRAFANA_PASSWORD}"
echo "  CSRF_SECRET_KEY:      ${CSRF_SECRET}"
echo "  INTERNAL_API_KEY:     ${INTERNAL_API_KEY}"
echo "  BACKEND_API_KEY:      ${BACKEND_API_KEY}"
echo ""
echo "🚀 Next steps:"
echo "  1. Review .env and frontend/.env"
echo "  2. docker compose up --build"
echo "  3. docker compose exec backend diesel migration run"
echo "  4. docker compose exec backend cargo run --bin seed"
```

---

## 🐳 Docker Compose Profiles

```bash
# Development (default) - hot reload, debug logs
docker compose up

# Production - optimized images, no volumes
docker compose --env-file .env.production -f docker-compose.yml -f docker-compose.prod.yml up -d

# Specific services only
docker compose up postgres redis backend

# With monitoring stack
docker compose --profile monitoring up

# With search
docker compose --profile search up
```

### Production Overrides (`docker-compose.prod.yml`)

```yaml
services:
  backend:
    build:
      target: production
    env_file: .env.production
    volumes: []
  
  frontend:
    build:
      target: production
    env_file: .env.production
    volumes: []
  
  nginx:
    volumes:
      - ./infra/ssl:/etc/nginx/ssl:ro
```

---

## 📦 Database Migrations

```bash
# Create new migration
docker compose exec backend diesel migration generate migration_name

# Run migrations
docker compose exec backend diesel migration run

# Revert last migration
docker compose exec backend diesel migration revert

# Redo (revert + run)
docker compose exec backend diesel migration redo

# List migrations
docker compose exec backend diesel migration list
```

### Migration Structure

```
backend/migrations/
├── 20240101000001_create_users/
│   ├── up.sql
│   └── down.sql
├── 20240101000002_create_roles/
│   ├── up.sql
│   └── down.sql
└── ...
```

---

## 🧪 Testing

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
pnpm test          # vitest
pnpm test:e2e      # playwright

# Integration tests (requires Docker)
docker compose -f docker-compose.test.yml up --abort-on-container-exit
```

---

## 🔒 Security

### Automated Security Scanning

This project uses automated security scanning via GitHub Actions:

- **Cargo Audit**: Runs on every push/PR to `main` branch
- **Daily Scheduled Scans**: Runs automatically at 00:00 UTC
- **Dependency Monitoring**: Checks for known vulnerabilities in Rust dependencies

### Running Security Audits Locally

```bash
# Using the convenience script
./scripts/security-audit.sh

# Or directly
cd backend
cargo audit
```

### CI/CD Security Checks

The CI pipeline includes:
- Security vulnerability scanning with `cargo audit`
- Code linting with `cargo clippy`
- Format checking with `cargo fmt`
- Unit and integration tests

All security checks must pass before merging PRs.

### Security Best Practices

- All dependencies are regularly audited
- Secrets are managed via environment variables
- Password hashing uses Argon2id
- JWT tokens use HS256 with secure key management
- 2FA support with TOTP
- Rate limiting on auth endpoints
- CSRF protection on state-changing operations

---

## 📁 Project Structure Details

### Backend Architecture

```
src/
├── config/           # AppConfig (env-driven)
├── controllers/      # HTTP handlers (thin)
├── services/         # Business logic
├── repositories/     # Data access (Diesel)
├── models/           # Domain entities
├── routes/           # Route definitions
├── middleware/       # Auth, CORS, rate-logs, CORS, metrics, logging
├── auth/             # JWT, Paseto, PBKDF2, TOTP
├── authz/            # RBAC engine (grants/abilities)
├── security/         # Encryption, blind indexes, key mgmt
├── ws/               # WebSocket server
├── db/               # Diesel setup, connection pool
├── errors/           # Error types, handlers
├── utils/            # Helpers (pagination, validation, etc.)
└── bin/seed.rs       # Database seeder
```

### Frontend Architecture

```
app/
├── components/       # Vue components
│   ├── admin/        # Admin panel components
│   ├── ui/           # Base UI components
│   └── *.vue         # Landing/shared components
├── layouts/          # Page layouts
│   ├── landing.vue   # Public pages
│   ├── default.vue   # Authenticated portal
│   ├── admin.vue     # Admin panel
│   ├── portal.vue    # User portal
│   └── auth.vue      # Auth pages
├── pages/            # File-based routing
│   ├── admin/        # Admin panel pages
│   ├── portal/       # User portal pages
│   ├── auth/         # Auth pages
│   └── *.vue         # Public pages
├── composables/      # Vue composables
├── plugins/          # Nuxt plugins
├── stores/           # Pinia stores
├── middleware/       # Route middleware
└── utils/            # Helpers
```

---

## 🔧 Useful Commands

### Backend

```bash
# Watch + run (dev)
cargo watch -x "run --bin backend"

# Run seed
cargo run --bin seed

# Check + clippy
cargo check && cargo clippy

# Format
cargo fmt --all

# Generate OpenAPI spec
cargo run --example openapi_gen

# View logs (Docker)
docker compose logs -f backend
```

### Frontend

```bash
# Type check
pnpm run typecheck

# Lint
pnpm run lint

# Build for production
pnpm run build

# Preview production build
pnpm run preview

# Generate API client (after backend changes)
pnpm run generate:api

# View logs (Docker)
docker compose logs -f frontend
```

### Database

```bash
# Connect to Postgres
docker compose exec postgres psql -U boilerplate -d boilerplate_dev

# Connect to Redis
docker compose exec redis redis-cli -a "$REDIS_PASSWORD"

# Backup database
docker compose exec postgres pg_dump -U boilerplate boilerplate_dev > backup.sql

# Restore database
cat backup.sql | docker compose exec -T postgres psql -U boilerplate -d boilerplate_dev
```

### Connection Pool Tuning

Connection pools can be tuned for high-concurrency scenarios:

#### PostgreSQL (Diesel)

**Environment Variables:**
- `DB_POOL_SIZE`: Maximum connections (default: 10)
- `DB_POOL_MIN_IDLE`: Minimum idle connections (default: auto, 20% of pool size)
- `DB_POOL_MAX_LIFETIME_SECS`: Max connection lifetime (default: 1800 = 30 min)
- `DB_POOL_IDLE_TIMEOUT_SECS`: Idle timeout (default: 600 = 10 min)
- `DB_POOL_CONNECTION_TIMEOUT_SECS`: Connection timeout (default: 10 sec)

**Recommended Settings by Workload:**

| Workload | DB_POOL_SIZE | DB_POOL_MIN_IDLE | Notes |
|----------|--------------|------------------|-------|
| Development | 5-10 | 1-2 | Minimal resource usage |
| Staging | 10-20 | 2-5 | Moderate load testing |
| Production (Low) | 20-30 | 5-10 | < 100 req/s |
| Production (Medium) | 30-50 | 10-15 | 100-500 req/s |
| Production (High) | 50-100 | 15-30 | > 500 req/s, monitor closely |

#### Redis (deadpool-redis)

**Environment Variables:**
- `REDIS_POOL_SIZE`: Maximum connections (default: 10)

**Recommended Settings by Workload:**

| Workload | REDIS_POOL_SIZE | Notes |
|----------|-----------------|-------|
| Development | 5-10 | Minimal resource usage |
| Staging | 10-20 | Moderate load testing |
| Production (Low) | 20-30 | < 100 req/s |
| Production (Medium) | 30-50 | 100-500 req/s |
| Production (High) | 50-100 | > 500 req/s, monitor closely |

**Tips:**
- Start with defaults and monitor connection usage
- Increase pool sizes gradually based on metrics
- For PostgreSQL: set `DB_POOL_MAX_LIFETIME_SECS` to prevent connection leaks
- For Redis: deadpool-redis handles connection recycling automatically
- Use connection pooling at the database level (PgBouncer) for very high concurrency
- Monitor: active connections, idle connections, wait time

### Request Payload Limits

Request payload limits can be tuned based on your API requirements:

**Environment Variables:**
- `JSON_PAYLOAD_LIMIT`: JSON body size limit in bytes (default: 16 MB)
- `FORM_PAYLOAD_LIMIT`: Form/multipart body size limit in bytes (default: 20 MB)

**When to increase:**
- **JSON_PAYLOAD_LIMIT**: When endpoints accept large JSON payloads (e.g., file upload metadata, batch operations, complex nested data)
- **FORM_PAYLOAD_LIMIT**: When endpoints accept large multipart forms (e.g., multiple files, large form submissions)

**Example (High-upload scenarios):**
```bash
# Allow larger JSON payloads (e.g., file upload metadata with thumbnails)
JSON_PAYLOAD_LIMIT=33554432       # 32 MB

# Allow larger multipart forms
FORM_PAYLOAD_LIMIT=104857600      # 100 MB
```

> **Note**: These limits apply to metadata only. Actual file uploads should use dedicated file storage (S3, Backblaze B2, Bunny.net) and not be sent through the JSON/Form payload.

---

## 🚀 Deployment

### Production Checklist

- [ ] Set `ENVIRONMENT=production`
- [ ] Use strong, unique secrets (run `./scripts/generate-secrets.sh`)
- [ ] Configure TLS certificates (`infra/ssl/cert.pem`, `key.pem`)
- [ ] Set up managed PostgreSQL (RDS, Cloud SQL, etc.)
- [ ] Set up managed Redis (ElastiCache, etc.)
- [ ] Configure email provider (Resend API key)
- [ ] Set up S3-compatible storage (Bunny.net, Backblaze B2)
- [ ] Configure Stripe keys (if using payments)
- [ ] Set up monitoring alerts (Grafana/Prometheus)
- [ ] Configure backup strategy
- [ ] Run security audit (`cargo audit` or `./scripts/security-audit.sh`)
- [ ] Verify CI/CD workflows are passing (GitHub Actions)

### Kubernetes / Cloud Deploy

The Docker images are multi-arch (amd64/arm64) and ready for:
- AWS ECS/Fargate / EKS
- Google Cloud Run / GKE
- Azure Container Apps / AKS
- DigitalOcean App Platform / Kubernetes
- Fly.io / Railway / Render

---

## 🤝 Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feat/amazing-feature`
3. Commit changes: `git commit -m 'feat: add amazing feature'`
4. Push branch: `git push origin feat/amazing-feature`
5. Open Pull Request

### Commit Convention

Follows [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new admin dashboard widget
fix: resolve JWT refresh token rotation bug
refactor: simplify repository query builder
docs: update API documentation
chore: update dependencies
```

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- [Actix Web](https://actix.rs/) - Powerful Rust web framework
- [Nuxt](https://nuxt.com/) - The Intuitive Vue Framework
- [Diesel](https://diesel.rs/) - Safe, extensible ORM
- [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS
- [FlyonUI](https://flyonui.com/) - Tailwind component library
- [utoipa](https://github.com/juhaku/utoipa) - OpenAPI for Rust
- [actix-web-grants](https://github.com/ivan-k/cargo-grant) - RBAC for Actix

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/rust-nuxt-boilerplate/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/rust-nuxt-boilerplate/discussions)
- **Email**: support@boilerplate-rust-nuxt.com

---

Built by [gilcierweb](https://gilcierweb.com.br) - https://gilcierweb.com.br

---
<div align="center">
  <strong>Built for developers who want to ship faster</strong>
  <br>
  <sub>Clone, configure, and deploy your first feature today.</sub>
</div>