#!/usr/bin/env bash
# Generate secure secrets for .env file
# Usage: ./scripts/generate-secrets.sh >> .env

set -euo pipefail

echo "# ──────────────────────────────────────────────"
echo "# AUTO-GENERATED SECRETS - $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo "# ──────────────────────────────────────────────"
echo

# JWT Secret (64 chars base64 = 48 bytes entropy)
JWT_SECRET=$(openssl rand -base64 48)
echo "JWT_SECRET=${JWT_SECRET}"
echo

# Redis Password (24 chars base64 = 18 bytes entropy)
REDIS_PASSWORD=$(openssl rand -base64 24)
echo "REDIS_PASSWORD=${REDIS_PASSWORD}"
echo

# PostgreSQL Password (24 chars base64)
POSTGRES_PASSWORD=$(openssl rand -base64 24)
echo "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}"
echo

# MeiliSearch Master Key (24 chars base64)
MEILI_MASTER_KEY=$(openssl rand -base64 24)
echo "MEILI_MASTER_KEY=${MEILI_MASTER_KEY}"
echo

# Grafana Admin Password (16 chars base64 = 12 bytes entropy)
GRAFANA_PASSWORD=$(openssl rand -base64 16)
echo "GRAFANA_PASSWORD=${GRAFANA_PASSWORD}"
echo

# Master Encryption Key (32 bytes base64 = 256 bits for AES-GCM)
MASTER_ENCRYPTION_KEY=$(openssl rand -base64 32)
echo "MASTER_ENCRYPTION_KEY=${MASTER_ENCRYPTION_KEY}"
echo

# Blind Index Key (32 bytes base64)
BLIND_INDEX_KEY=$(openssl rand -base64 32)
echo "BLIND_INDEX_KEY=${BLIND_INDEX_KEY}"
echo

# CSRF Secret (hex, 32 bytes = 64 chars)
CSRF_SECRET_KEY=$(openssl rand -hex 32)
echo "CSRF_SECRET_KEY=${CSRF_SECRET_KEY}"
echo

# Backend API Key (for frontend server-to-server calls, hex 32 bytes)
BACKEND_API_KEY=$(openssl rand -hex 32)
echo "BACKEND_API_KEY=${BACKEND_API_KEY}"
echo

# Optional: Stripe webhook secret prefix (if using Stripe)
# STRIPE_WEBHOOK_SECRET=whsec_$(openssl rand -hex 32)
# echo "STRIPE_WEBHOOK_SECRET=${STRIPE_WEBHOOK_SECRET}"
# echo

echo "# ──────────────────────────────────────────────"
echo "# Add the above to your .env file"
echo "# ──────────────────────────────────────────────"