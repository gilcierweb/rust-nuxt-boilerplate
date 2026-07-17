# Rust + Nuxt Boilerplate

> **Production-ready full-stack boilerplate** - Rust (Actix Web) + Nuxt 4 with authentication, RBAC, admin panel, type-safe database layer, and modern developer experience.

[![Rust](https://img.shields.io/badge/Rust-1.95-orange?logo=rust)](https://www.rust-lang.org/)
[![Actix Web](https://img.shields.io/badge/Actix%20Web-4.14-blue)](https://actix.rs/)
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

## 🚀 Local Setup (Recommended)

### Prerequisites
- 4GB+ RAM available
- Ports free: 3000, 8080, 5432, 6379

### 1. Install Rust

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Verify
rustc --version
cargo --version
```

[Official docs](https://www.rust-lang.org/tools/install)

### 2. Install Node.js + pnpm

```bash
# Linux / macOS (via nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash
nvm install 20
nvm use 20

# Verify
node --version   # v20+

# Install pnpm via corepack
corepack enable
corepack prepare pnpm@9 --activate

# Verify
pnpm --version   # 9+
```

[Node.js](https://nodejs.org/) | [nvm](https://github.com/nvm-sh/nvm)

### 3. Install PostgreSQL

```bash
# Ubuntu / Debian
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql

# macOS (Homebrew)
brew install postgresql@16
brew services start postgresql@16

# Create database and user
sudo -u postgres createdb boilerplate_dev
sudo -u postgres psql -c "CREATE USER boilerplate WITH PASSWORD 'changeme';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE boilerplate_dev TO boilerplate;"

# Verify
psql -U boilerplate -d boilerplate_dev -c "SELECT version();"
```

[PostgreSQL Downloads](https://www.postgresql.org/download/)

### 4. Install Redis

```bash
# Ubuntu / Debian
sudo apt update
sudo apt install redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server

# macOS (Homebrew)
brew install redis
brew services start redis

# Verify
redis-cli ping   # PONG
```

[Redis Downloads](https://redis.io/download)

### 5. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres

# Verify
diesel --version
```

### 6. Install Extra Tools

```bash
# Hot-reload for backend
cargo install cargo-watch

# Verify
cargo watch --version
```

### 7. Install Proctor (Optional)

[Proctor](https://github.com/alecthomas/proctor) manages multiple processes with health checks and dependency ordering. Written in Rust.

```bash
curl -fsSL https://raw.githubusercontent.com/alecthomas/proctor/master/install.sh | sh

# Verify
proctor --version
```

Proctor starts these services:
- **Backend**: `cargo watch -x "run --bin backend"` → http://localhost:8080
- **Frontend**: `pnpm dev -p 4000` → http://localhost:4000

The frontend runs on port 4000 (instead of 3000) to avoid conflicts with other services.

### 8. Configure the Project

```bash
git clone https://github.com/gilcierweb/rust-nuxt-boilerplate.git
cd rust-nuxt-boilerplate

# Generate secure secrets
./scripts/generate-secrets.sh

# Review generated .env file
cat .env
```

### 9. Run Migrations

```bash
cd backend

# Run migrations
diesel migration run

# Seed demo data (admin user, roles, permissions)
cargo run --bin seed
```

### 10. Start Backend

**Option A: With Proctor (all services at once)**

```bash
cd rust-nuxt-boilerplate
proctor
```

- Backend: http://localhost:8080
- Frontend: http://localhost:4000

**Option B: Manual start**

```bash
cd backend

# With hot-reload (recommended)
cargo watch -x "run --bin backend"

# Or without hot-reload
cargo run --bin backend
```

**Backend:** http://localhost:8080

### 11. Start Frontend

```bash
cd frontend

# Install dependencies
pnpm install

# Start dev server
pnpm dev
```

**Frontend:** http://localhost:3000

### 12. Access Services

| Service | URL | Credentials |
|---------|-----|-------------|
| **Frontend** | http://localhost:3000 | - |
| **Backend API** | http://localhost:8080 | - |
| **Swagger UI** | http://localhost:8080/swagger-ui | - |
| **Scalar** | http://localhost:8080/scalar | - |
| **Health Check** | http://localhost:8080/health | - |
| **Grafana** | http://localhost:3001 | admin / (from .env) |
| **Prometheus** | http://localhost:9090 | - |
| **MeiliSearch** | http://localhost:7700 | master key from .env |
| **Portainer** | http://localhost:9000 | - |

**Default Admin** (after seeding):
- Email: `admin@example.com`
- Password: `changeme123` ⚠️ **Change immediately!**

---

## 🐳 Docker Setup (Alternative)

For those who prefer not to install dependencies locally.

### Quick Start

```bash
git clone https://github.com/gilcierweb/rust-nuxt-boilerplate.git
cd rust-nuxt-boilerplate

# Generate secure secrets
./scripts/generate-secrets.sh

# Build and start all services with hot-reload
docker compose up --build

# Or run in background
docker compose up -d --build
```

### Initialize Database

```bash
docker compose exec backend diesel migration run
docker compose exec backend cargo run --bin seed
```

### Docker Compose Profiles

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

## 🔄 Process Management

### Proctor (Recommended for Local Dev)

[Proctor](https://github.com/alecthomas/proctor) manages multiple processes with a single config, similar to Foreman/Procfile but with better log handling and health checks. Written in **Rust**.

**Install:**
```bash
curl -fsSL https://raw.githubusercontent.com/alecthomas/proctor/master/install.sh | sh

# Or install specific version to custom directory
curl -fsSL https://raw.githubusercontent.com/alecthomas/proctor/master/install.sh | INSTALL_DIR=~/.local/bin sh -s v0.1.0

# macOS via Homebrew
brew install alecthomas/tap/proctor
```

**Usage with included Procfile:**
```bash
# Start all processes (databases → backend → frontend)
proctor

# Follow logs
proctor logs -f

# Stop all
proctor stop

# Restart single process
proctor restart backend

# Check status
proctor status
```

The project includes a `Procfile` that Proctor reads directly. It defines:
- `postgres`: PostgreSQL via docker compose
- `redis`: Redis via docker compose  
- `backend`: Rust backend with hot reload → http://localhost:8080 (depends on postgres + redis)
- `frontend`: Nuxt frontend → http://localhost:4000 (depends on backend)

### Procfile (Alternative)

Create `Procfile` in project root for Foreman/Heroku-style process management (already included):

```procfile
# Procfile - already included in this repo
backend: cd backend && cargo watch -x "run --bin backend"
frontend: cd frontend && pnpm dev -p 4000
postgres: docker compose up postgres
redis: docker compose up redis
meilisearch: docker compose up meilisearch
monitoring: docker compose --profile monitoring up
```

**Usage with Foreman:**
```bash
gem install foreman
foreman start
foreman start -f Procfile
```

---

## 📦 Database Migrations

```bash
# Create new migration
diesel migration generate migration_name

# Run migrations
diesel migration run

# Revert last migration
diesel migration revert

# Redo (revert + run)
diesel migration redo

# List migrations
diesel migration list
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

- **Cargo Audit**: Rust dependency vulnerabilities on every push/PR
- **pnpm audit**: Node.js dependency vulnerabilities
- **Gitleaks**: Secret scanning on push/PR
- **Daily Scheduled Scans**: Runs automatically at 00:00 UTC

### Running Locally

```bash
cd backend && cargo audit
cd frontend && pnpm audit --prod
```

### Security Best Practices

- Password hashing uses Argon2id
- JWT tokens use HS256 with secure key management
- 2FA support with TOTP
- Rate limiting on auth endpoints
- CSRF protection on state-changing operations
- Field-level encryption with blind indexes

---

## 📁 Project Structure

### Backend

```
src/
├── config/           # AppConfig (env-driven)
├── controllers/      # HTTP handlers (thin)
├── services/         # Business logic
├── repositories/     # Data access (Diesel)
├── models/           # Domain entities
├── routes/           # Route definitions
├── middleware/       # Auth, CORS, rate limiting, metrics
├── auth/             # JWT, Paseto, PBKDF2, TOTP
├── authz/            # RBAC engine (grants/abilities)
├── security/         # Encryption, blind indexes, key mgmt
├── ws/               # WebSocket server
├── db/               # Diesel setup, connection pool
├── errors/           # Error types, handlers
├── utils/            # Helpers (pagination, validation, etc.)
└── bin/seed.rs       # Database seeder
```

### Frontend

```
app/
├── components/       # Vue components
│   ├── admin/        # Admin panel components
│   ├── ui/           # Base UI components
│   └── *.vue         # Landing/shared components
├── layouts/          # Page layouts
├── pages/            # File-based routing
│   ├── admin/        # Admin panel pages
│   ├── portal/       # User portal pages
│   └── auth/         # Auth pages
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
cargo watch -x "run --bin backend"     # Watch + run (dev)
cargo run --bin seed                    # Run seed
cargo check && cargo clippy             # Check + clippy
cargo fmt --all                         # Format
docker compose logs -f backend          # View logs (Docker)
```

### Frontend

```bash
pnpm run typecheck                      # Type check
pnpm run lint                           # Lint
pnpm run build                          # Build for production
pnpm run preview                        # Preview production build
docker compose logs -f frontend         # View logs (Docker)
```

### Database

```bash
docker compose exec postgres psql -U boilerplate -d boilerplate_dev   # Connect to Postgres
docker compose exec redis redis-cli -a "$REDIS_PASSWORD"              # Connect to Redis
```

### Connection Pool Tuning

**PostgreSQL (Diesel):**

| Workload | DB_POOL_SIZE | DB_POOL_MIN_IDLE |
|----------|--------------|------------------|
| Development | 5-10 | 1-2 |
| Staging | 10-20 | 2-5 |
| Production (Low) | 20-30 | 5-10 |
| Production (High) | 50-100 | 15-30 |

**Redis (deadpool-redis):**

| Workload | REDIS_POOL_SIZE |
|----------|-----------------|
| Development | 5-10 |
| Staging | 10-20 |
| Production | 30-100 |

---

## 🚀 Deployment

### Production Checklist

- [ ] Set `ENVIRONMENT=production`
- [ ] Use strong, unique secrets
- [ ] Configure TLS certificates
- [ ] Set up managed PostgreSQL (RDS, Cloud SQL, etc.)
- [ ] Set up managed Redis (ElastiCache, etc.)
- [ ] Configure email provider (Resend API key)
- [ ] Set up S3-compatible storage (Bunny.net, Backblaze B2)
- [ ] Configure Stripe keys (if using payments)
- [ ] Set up monitoring alerts (Grafana/Prometheus)
- [ ] Run security audit (`cargo audit`)

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

Built by [gilcierweb](https://gilcierweb.com.br) - https://gilcierweb.com.br

---
<div align="center">
  <strong>Built for developers who want to ship faster</strong>
  <br>
  <sub>Clone, configure, and deploy your first feature today.</sub>
</div>
