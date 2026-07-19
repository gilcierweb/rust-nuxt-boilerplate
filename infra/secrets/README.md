# Production secrets are mounted here for each service.
# This directory MUST contain real secret files in production.
# Files in this directory are gitignored.
#
# Required file names (one secret per file, no trailing newline issues — just one value per file):
#
#   postgres_password.txt       # DB root password
#   redis_password.txt          # Redis AUTH password (if any)
#   jwt_secret.txt              # 64+ char base64 JWT signing key
#   master_encryption_key.txt   # 32-byte base64 master encryption key
#   blind_index_key.txt         # 32-byte base64 blind-index key for searchable PII
#   csrf_secret_key.txt         # 32-byte base64 CSRF HMAC key
#   refresh_token_hash_salt.txt # 32-byte base64 salt for refresh token hashes
#
# External service API keys are also kept here for production:
#
#   resend_api_key.txt          # Resend transactional email
#   stripe_secret_key.txt       # Stripe secret
#   stripe_webhook_secret.txt   # Stripe webhook signing secret
#   bunny_storage_key.txt       # Bunny.net storage
#   bunny_token_key.txt         # Bunny.net token authentication
#   bunny_stream_key.txt        # Bunny.net Stream
#   bunny_stream_webhook_secret.txt
#   b2_key_id.txt               # Backblaze B2 key ID
#   b2_application_key.txt      # Backblaze B2 application key
#
# Files are mounted as /run/secrets/<NAME> in containers, then read by
# the application code (see backend/src/config/secrets.rs).
#
# Generate secrets:
#   openssl rand -base64 48      # 64-char base64 (use for JWT_SECRET)
#   openssl rand -base64 32      # 32-byte base64 (encryption keys)
#
# Per-instance rotation:
#   docker compose -f docker-compose.yml -f docker-compose.prod.yml run --rm \
#     backend rotate-secret JWT_SECRET
