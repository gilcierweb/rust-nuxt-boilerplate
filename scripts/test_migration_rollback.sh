#!/usr/bin/env bash
# Migration rollback test script
# Tests that all diesel migrations can be applied and rolled back successfully.
# Usage: ./scripts/test_migration_rollback.sh [DATABASE_URL]
# If DATABASE_URL is not provided, reads from .env file.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKEND_DIR="$PROJECT_ROOT/backend"
MIGRATIONS_DIR="$BACKEND_DIR/migrations"

# Load DATABASE_URL from .env if not provided
if [[ -z "${1:-}" ]]; then
    if [[ -f "$PROJECT_ROOT/.env" ]]; then
        export DATABASE_URL=$(grep '^DATABASE_URL=' "$PROJECT_ROOT/.env" | cut -d'=' -f2- | sed 's/^"//;s/"$//')
    elif [[ -f "$BACKEND_DIR/.env" ]]; then
        export DATABASE_URL=$(grep '^DATABASE_URL=' "$BACKEND_DIR/.env" | cut -d'=' -f2- | sed 's/^"//;s/"$//')
    fi
else
    export DATABASE_URL="$1"
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "ERROR: DATABASE_URL not set. Provide as argument or in .env file."
    exit 1
fi

echo "=== Migration Rollback Test ==="
echo "Database URL: ${DATABASE_URL%%@*}@****"
echo "Migrations directory: $MIGRATIONS_DIR"
echo ""

# Check diesel_cli is installed
if ! command -v diesel &> /dev/null; then
    echo "Installing diesel_cli..."
    cargo install diesel_cli --no-default-features --features postgres
fi

# Count migrations
MIGRATION_COUNT=$(ls -1 "$MIGRATIONS_DIR" | grep -E '^[0-9]+_' | wc -l | tr -d ' ')
echo "Found $MIGRATION_COUNT migrations"
echo ""

# Verify all migrations have down.sql
echo "Verifying down.sql files exist..."
for migration in "$MIGRATIONS_DIR"/*/; do
    name=$(basename "$migration")
    if [[ "$name" == ".keep" ]]; then
        continue
    fi
    if [[ ! -f "$migration/down.sql" ]]; then
        echo "ERROR: Missing down.sql for migration: $name"
        exit 1
    fi
    # Check it's not empty
    if [[ ! -s "$migration/down.sql" ]]; then
        echo "WARNING: down.sql is empty for migration: $name"
    fi
done
echo "All down.sql files present."
echo ""

# Apply all migrations
echo "Running diesel migration run..."
cd "$BACKEND_DIR"
diesel migration run --database-url "$DATABASE_URL"
echo "All migrations applied successfully."
echo ""

# Rollback all migrations
echo "Running diesel migration revert (all)..."
diesel migration revert --database-url "$DATABASE_URL" --all
echo "All migrations rolled back successfully."
echo ""

# Apply all migrations again
echo "Re-applying all migrations..."
diesel migration run --database-url "$DATABASE_URL"
echo "All migrations re-applied successfully."
echo ""

# Verify migration status
echo "=== Migration Status ==="
diesel migration list --database-url "$DATABASE_URL"
echo ""

echo "=== Migration Rollback Test PASSED ==="