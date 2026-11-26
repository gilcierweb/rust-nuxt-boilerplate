#!/usr/bin/env bash
# Generate secure secrets and write them into .env files.
#
# Usage: ./scripts/generate-secrets.sh
#
# This script:
#   * copies .env.example to .env at the repository root (backing up any
#     existing .env first);
#   * generates fresh secrets and substitutes every placeholder value in
#     .env with the generated value;
#   * creates frontend/.env with the values the Nuxt dev server needs
#     (NUXT_BACKEND_API_BASE and NUXT_BACKEND_API_KEY), substituting the
#     generated service-to-service key.
#
# Re-running the script regenerates ALL secrets, so back up .env first if
# you need to keep the previous values.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ ! -f .env.example ]; then
    echo "Missing .env.example at $ROOT_DIR" >&2
    exit 1
fi

# ── Generate secrets ──────────────────────────────────────────────────────
JWT_SECRET="$(openssl rand -base64 48)"
# URL-safe passwords (no / + =) so they can be embedded in connection URLs.
REDIS_PASSWORD="$(openssl rand -base64 24 | tr '+/' '-_' | tr -d '=')"
POSTGRES_PASSWORD="$(openssl rand -base64 24 | tr '+/' '-_' | tr -d '=')"
MEILI_MASTER_KEY="$(openssl rand -base64 24)"
GRAFANA_PASSWORD="$(openssl rand -base64 16)"
MASTER_KEY="$(openssl rand -base64 32)"
BLIND_INDEX_KEY="$(openssl rand -base64 32)"
CSRF_SECRET_KEY="$(openssl rand -hex 32)"
REFRESH_TOKEN_HASH_SALT="$(openssl rand -base64 32)"
METRICS_SCRAPE_KEY="$(openssl rand -hex 32)"
BACKEND_API_KEY="$(openssl rand -hex 32)"

export JWT_SECRET REDIS_PASSWORD POSTGRES_PASSWORD MEILI_MASTER_KEY \
       GRAFANA_PASSWORD MASTER_KEY BLIND_INDEX_KEY CSRF_SECRET_KEY \
       REFRESH_TOKEN_HASH_SALT METRICS_SCRAPE_KEY BACKEND_API_KEY

# ── Build root .env from .env.example ──────────────────────────────────────
tmp_env="$(mktemp)"
trap 'rm -f "$tmp_env"' EXIT
cp .env.example "$tmp_env"

python3 - "$tmp_env" <<'PY'
import os
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

key_values = {
    "JWT_SECRET": os.environ["JWT_SECRET"],
    "REDIS_PASSWORD": os.environ["REDIS_PASSWORD"],
    "POSTGRES_PASSWORD": os.environ["POSTGRES_PASSWORD"],
    "MEILI_MASTER_KEY": os.environ["MEILI_MASTER_KEY"],
    "METRICS_SCRAPE_KEY": os.environ["METRICS_SCRAPE_KEY"],
    "GRAFANA_PASSWORD": os.environ["GRAFANA_PASSWORD"],
    "MASTER_KEY": os.environ["MASTER_KEY"],
    "BLIND_INDEX_KEY": os.environ["BLIND_INDEX_KEY"],
    "CSRF_SECRET_KEY": os.environ["CSRF_SECRET_KEY"],
    "REFRESH_TOKEN_HASH_SALT": os.environ["REFRESH_TOKEN_HASH_SALT"],
    "NUXT_BACKEND_API_KEY": os.environ["BACKEND_API_KEY"],
    "BACKEND_API_KEYS": os.environ["BACKEND_API_KEY"],
}

out = []
for line in lines:
    stripped = line.strip()
    if "=" in stripped and not stripped.startswith("#"):
        key, _sep, _val = stripped.partition("=")
        key = key.strip()
        if key in key_values:
            idx = line.find("=")
            prefix = line[: idx + 1]
            rest = line[idx + 1:]
            if " #" in rest:
                _v, comment = rest.split(" #", 1)
                comment = " #" + comment
            else:
                comment = ""
            newline = prefix + key_values[key] + comment
            if line.endswith("\n") and not newline.endswith("\n"):
                newline += "\n"
            out.append(newline)
            continue
        if key == "DATABASE_URL":
            idx = line.find("=")
            prefix = line[: idx + 1]
            rest = line[idx + 1:]
            if " #" in rest:
                valpart, comment = rest.split(" #", 1)
                comment = " #" + comment
            else:
                valpart = rest
                comment = ""
            valpart = valpart.replace(
                "changeme_secure_password", os.environ["POSTGRES_PASSWORD"]
            )
            newline = prefix + valpart + comment
            if line.endswith("\n") and not newline.endswith("\n"):
                newline += "\n"
            out.append(newline)
            continue
        if key == "REDIS_URL":
            idx = line.find("=")
            prefix = line[: idx + 1]
            rest = line[idx + 1:]
            if " #" in rest:
                valpart, comment = rest.split(" #", 1)
                comment = " #" + comment
            else:
                valpart = rest
                comment = ""
            valpart = valpart.replace(
                "changeme_redis_password", os.environ["REDIS_PASSWORD"]
            )
            newline = prefix + valpart + comment
            if line.endswith("\n") and not newline.endswith("\n"):
                newline += "\n"
            out.append(newline)
            continue
    out.append(line)

with open(path, "w", encoding="utf-8") as f:
    f.writelines(out)
PY

# Back up existing root .env, then install the generated one.
if [ -f .env ]; then
    backup=".env.bak.$(date -u +%Y%m%d%H%M%S)"
    cp .env "$backup"
    echo "Backed up existing .env to $backup" >&2
fi
mv "$tmp_env" .env
trap - EXIT

# ── Frontend .env ──────────────────────────────────────────────────────────
frontend_env="frontend/.env"
if [ -f "$frontend_env" ]; then
    backup="frontend/.env.bak.$(date -u +%Y%m%d%H%M%S)"
    cp "$frontend_env" "$backup"
    echo "Backed up existing frontend/.env to $backup" >&2
fi
cat > "$frontend_env" <<EOF
NUXT_BACKEND_API_BASE=http://localhost:8080/api/v1
NUXT_BACKEND_API_KEY=${BACKEND_API_KEY}
EOF

echo "Generated secrets into .env and frontend/.env" >&2
echo "Review the files before running the stack." >&2