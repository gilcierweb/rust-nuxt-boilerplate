# Infrastructure Configuration

This directory contains infrastructure-as-code configurations for the production stack.

## Structure

```
infra/
├── nginx/
│   ├── nginx.conf           # Main Nginx configuration
│   └── conf.d/
│       ├── app.conf         # Server blocks (HTTP→HTTPS, frontend, API, WS)
│       ├── upstream.conf    # Upstream backend/frontend definitions
│       ├── ssl.conf         # SSL/TLS hardening (Mozilla intermediate)
│       ├── security.conf    # Security headers (HSTS, CSP, X-Frame-Options)
│       ├── rate-limit.conf  # Rate limiting zones (per-IP, auth, API)
│       └── proxy.conf       # Reusable proxy-pass directives
├── prometheus/
│   └── prometheus.yml       # Prometheus scrape targets
├── grafana/
│   ├── dashboards/          # JSON dashboard definitions
│   └── provisioning/        # Grafana datasource/dashboards provisioning
├── secrets/                 # Production secrets (gitignored)
│   ├── README.md            # Which file corresponds to which secret
│   └── *.txt                # One secret per file
└── ssl/                     # TLS certificates (gitignored)
    ├── cert.pem             # Certificate (production)
    ├── key.pem              # Private key (production)
    └── dhparam.pem          # Diffie-Hellman parameters (gitignored, optional)
```

## Production Secrets (SECURITY_AUDIT.md I3)

In production, the backend reads critical secrets from files mounted via Docker
Compose `secrets:` (not from `env_file: .env`). Each critical secret is one file
in `infra/secrets/`. See `infra/secrets/README.md` for the list.

### Setup a new environment

```bash
# Generate secrets (do this once per environment)
mkdir -p infra/secrets
openssl rand -base64 64 > infra/secrets/jwt_secret.txt
openssl rand -base64 32 > infra/secrets/master_encryption_key.txt
openssl rand -base64 32 > infra/secrets/blind_index_key.txt
openssl rand -base64 32 > infra/secrets/csrf_secret_key.txt
openssl rand -base64 32 > infra/secrets/refresh_token_hash_salt.txt
openssl rand -hex 24 > infra/secrets/postgres_password.txt
# External services:
echo -n "re_xxxxxxxxxxxxxx" > infra/secrets/resend_api_key.txt
echo -n "sk_live_xxxxxxxxxxxxxx" > infra/secrets/stripe_secret_key.txt
echo -n "whsec_xxxxxxxxxxxxxx" > infra/secrets/stripe_webhook_secret.txt
chmod 600 infra/secrets/*.txt
```

### Resolution order (backend)

`secret_from_env_or_file()` in `backend/src/config/app_config.rs` reads:

1. `${NAME}` env var
2. `${NAME}_FILE` env var (path to file)
3. `/run/secrets/<lowercase_name>` (Docker Compose `secrets:` direct mount)
4. Default value

### Rotation

```bash
# Replace a secret file
NEW_JWT=$(openssl rand -base64 64)
echo "$NEW_JWT" > infra/secrets/jwt_secret.txt
chmod 600 infra/secrets/jwt_secret.txt

# Restart the backend to pick up the new secret
docker compose -f docker-compose.yml -f docker-compose.prod.yml restart backend
```

**Note:** `JWT_SECRET` rotation invalidates all live sessions. Plan accordingly.

## SSL Certificates

For production, place your TLS certificates in `infra/ssl/`:

```bash
# Let's Encrypt (certbot)
sudo certbot certonly --standalone -d yourdomain.com
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem infra/ssl/cert.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem infra/ssl/key.pem
sudo chown $USER:$USER infra/ssl/*.pem
```

For development, generate self-signed certs:

```bash
openssl req -x509 -newkey rsa:4096 -keyout infra/ssl/key.pem -out infra/ssl/cert.pem \
  -days 365 -nodes -subj "/CN=localhost"
```

**Never commit real certificates or keys to git!** The `infra/ssl/` directory is gitignored.

## DH Parameters (Optional)

The `nginx.conf` references `/etc/nginx/dhparam.pem`. Generate one once per environment:

```bash
# 2048 bits — adequate for ECDHE-only modern ciphers
openssl dhparam -out infra/ssl/dhparam.pem 2048
# 4096 bits — stronger, takes ~30-60 minutes to generate
openssl dhparam -out infra/ssl/dhparam.pem 4096
```

Mount it in `docker-compose.yml` (nginx service) by adding:

```yaml
volumes:
  - ./infra/ssl/dhparam.pem:/etc/nginx/dhparam.pem:ro
```

If you do not provide one, nginx falls back to a built-in 1024-bit DH group, which OpenSSL 3.x rejects with a "dh key too small" error. **For production, generate and mount your own.**

## Nginx Configs

All configs are validated with `nginx -t` and pass cleanly. To test locally:

```bash
docker run --rm \
  --add-host backend:127.0.0.1 \
  --add-host frontend:127.0.0.1 \
  -v ./infra/nginx/nginx.conf:/etc/nginx/nginx.conf:ro \
  -v ./infra/nginx/conf.d:/etc/nginx/conf.d:ro \
  -v ./infra/ssl:/etc/nginx/ssl:ro \
  -v ./infra/ssl/dhparam.pem:/etc/nginx/dhparam.pem:ro \
  nginx:alpine nginx -t
```
