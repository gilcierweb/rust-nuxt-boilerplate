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
  <strong>Built with ❤️ for developers who want to ship faster</strong>
  <br>
  <sub>Clone, configure, and deploy your first feature today.</sub>
</div>