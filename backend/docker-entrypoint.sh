#!/bin/sh
# Entrypoint for the backend container.
#
# Modes:
#   migrate  — Apply pending Diesel migrations, then exit (CI/init-container use)
#   backend  — Start the HTTP API server (default)
#   <other>  — Passed through to exec as a shell fallback (debug only)
#
# SECURITY_AUDIT.md I6: Migrations are intentionally NOT coupled to the
# backend startup. The `migrate` command exits when done; failures are
# surfaced as a non-zero exit code so the compose `depends_on: condition:
# service_completed_successfully` will block the backend from starting
# until migrations succeed (or are a no-op).
#
# Usage in docker-compose:
#   services:
#     migrate:
#       command: ["migrate"]
#     backend:
#       depends_on:
#         migrate:
#           condition: service_completed_successfully
#
# Usage in Kubernetes:
#   initContainers:
#     - name: migrate
#       command: ["/app/docker-entrypoint.sh", "migrate"]
#   containers:
#     - name: backend
#       # ... no migration logic ...
set -eu

MODE="${1:-backend}"

case "$MODE" in
    migrate)
        echo "[entrypoint] Running database migrations..."
        if [ ! -x ./diesel ]; then
            echo "[entrypoint] FATAL: ./diesel binary not found in image" >&2
            exit 1
        fi
        if [ ! -d ./migrations ]; then
            echo "[entrypoint] FATAL: ./migrations directory not found" >&2
            exit 1
        fi
        # `diesel migration run` is idempotent — re-running on an up-to-date
        # DB exits 0 without applying any changes.
        ./diesel migration run
        echo "[entrypoint] Migrations complete."
        exit 0
        ;;
    backend)
        echo "[entrypoint] Starting backend..."
        if [ ! -x ./backend ]; then
            echo "[entrypoint] FATAL: ./backend binary not found" >&2
            exit 1
        fi
        exec ./backend
        ;;
    *)
        # Debug / exec override
        echo "[entrypoint] Unknown mode '$MODE'; passing through" >&2
        exec "$@"
        ;;
esac
