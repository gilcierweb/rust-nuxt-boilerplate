# Procfile - Process definitions for Foreman/Proctor/Overmind
# Usage: proctor
# Usage: foreman start -f Procfile
# Usage: overmind start -f Procfile

# proctor: watch=backend/**/*.rs debounce=300ms
backend: cd backend && cargo watch -x "run --bin backend"

# proctor: after=backend probe=http://localhost:8080/health
frontend: cd frontend && pnpm dev -p 4000

# Database (PostgreSQL) - run via docker compose
# postgres: docker compose up postgres

# Cache (Redis) - run via docker compose
# redis: docker compose up redis

# Search (MeiliSearch) - optional, run via docker compose
# meilisearch: docker compose up meilisearch

# Monitoring (Prometheus + Grafana) - optional, run via docker compose
# monitoring: docker compose --profile monitoring up

# Full stack (all services via docker compose)
# docker: docker compose up