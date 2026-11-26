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
│   ├── prometheus.yml       # Scrape targets (goes through metrics-proxy)
│   └── alert-rules.yml      # Rules for the rejection share and the breaker
├── grafana/
│   ├── dashboards/          # JSON dashboard definitions
│   └── provisioning/        # Grafana datasource/dashboards provisioning
├── opentelemetry/
│   └── collector.yaml       # Optional collector hub (metrics + OTLP traces)
├── secrets/                 # Production secrets (gitignored)
│   ├── README.md            # Which file corresponds to which secret
│   └── *.txt                # One secret per file
└── ssl/                     # TLS certificates (gitignored)
    ├── cert.pem             # Certificate (production)
    ├── key.pem              # Private key (production)
    └── dhparam.pem          # Diffie-Hellman parameters (gitignored, optional)
```

## Monitoring

The backend exposes three signals and the operator chooses the stack that reads
them. The application has one source of metrics: the registry rendered as
Prometheus exposition at `GET /api/v1/metrics`. Traces go out as OTLP when
`OTEL_EXPORTER_OTLP_ENDPOINT` is set. Events are structured JSON logs.

### The metrics endpoint requires an API key

`/api/v1/metrics` answers only to `X-API-Key: <key>` or
`Authorization: ApiKey <key>`, from `BACKEND_API_KEYS` in the backend `.env`
(the payload carries route names, counters and system data). Prometheus itself
can only send Bearer or Basic credentials, so the local stack scrapes through
the `metrics-proxy` service in `docker-compose.yml`, which injects the header.
Tell it which key to use in your `.env`:

```bash
# .env (root): METRICS_SCRAPE_KEY must be one of the keys in BACKEND_API_KEYS
METRICS_SCRAPE_KEY=$(openssl rand -hex 32)
```

`infra/prometheus/prometheus.yml` scrapes `metrics-proxy:9091` and loads the
rules in `infra/prometheus/alert-rules.yml`. Grafana is provisioned from
`infra/grafana/provisioning` with the `backend-overview` dashboard ready on
login; no click-through setup.

### Fan-out to a vendor backend

`infra/opentelemetry/collector.yaml` runs the collector as the ingestion hub: it
scrapes the backend with the API key, accepts OTLP traces from the app,
re-exposes metrics on `:8889` for a local Prometheus, and carries commented
exporters for Grafana Cloud and Datadog. The app needs no per-vendor code for
any of it.

### Alerting the backend already does itself

No external stack is required for the first line of defence. Configured in the
backend `.env` and documented in `.env.example`:

| Variable | Default | Meaning |
|---|---|---|
| `ALERT_429_ENABLED` | `true` | Turns the rejection-share watchdog on or off |
| `ALERT_429_SHARE_PCT` | `10` | Share of 429 responses in the window that trips it |
| `ALERT_429_WINDOW_SECS` | `30` | Seconds between evaluations |

The watchdog samples the registry off the request path and emits one structured
warning per crossing event (`rate_limit.429_share_threshold_exceeded`, with the
most refused route) plus the gauges and counters
`rate_limit_429_share` / `rate_limit_429_alerts_total` /
`circuit_breaker_state` / `circuit_breaker_opened_total` /
`circuit_breaker_open_seconds_total`, which is what the rules in
`alert-rules.yml` read. A mistyped percentage or a zero window refuses to boot
rather than silently disarming the alert.

Related event names in the structured logs:
`rate_limit.circuit_breaker_opened` (Redis breaker of the limiter) and
`outbound_http.circuit_breaker_opened` (e-mail/OAuth client, event-only today).

## Service Profiles (SECURITY_AUDIT.md I1, I4)

Sensitive services are gated behind Compose `profiles:` so they never run by
default. The default `docker compose up` boots a production-shaped stack; the
flags below add dev tooling that exposes host ports or the Docker socket.

| Command | Services |
|---------|----------|
| `docker compose up` | nginx, backend, frontend (production shape — no socket, no DB/Redis host exposure) |
| `docker compose --profile dev-tools up` | + postgres/redis (host ports 5432, 6379) + portainer (Docker socket, port 9000) |

### Why Postgres/Redis are NOT exposed by default

`SECURITY_AUDIT.md I1`: In production, the database and cache are internal-only.
Mounting them on the host is fine for local development with `psql`, `redis-cli`,
or GUI tools (DBeaver, RedisInsight), but it leaks credentials if the host
firewall is misconfigured. The `dev-tools` profile makes this explicit.

### Why Portainer is NOT included by default

`SECURITY_AUDIT.md I4`: Portainer requires `/var/run/docker.sock` mounted into
the container. Any process with that socket can spawn privileged containers,
mount the host filesystem, or grant itself shell access — equivalent to root
on the host. Acceptable for local dev only.

**Production alternatives:**

1. **Run Portainer Agent remotely** — Deploy Portainer CE on a *separate* host,
   run `portainer/agent` on each Docker host, connect via TCP+TLS. No socket mount.

   ```yaml
   # On the remote Docker host (NOT in this compose):
   portainer_agent:
     image: portainer/agent
     # ... no /var/run/docker.sock mount needed ...
     environment:
       - AGENT_CLUSTER_ADDR=portainer.example.com:9001
   ```

2. **Host-side tools** — `lazydocker` (TUI), `ctop`, `dive` (image inspection).
3. **Remote-hosted** — Use a managed Kubernetes service (EKS, GKE, AKS),
   Nomad, or vendor-managed (Portainer Cloud / Portainer BE on a hardened host).

If you must ship Portainer CE in this stack for production, at minimum:
- Bind port 9000 to `127.0.0.1` on the host (`"127.0.0.1:9000:9000"`)
- Front it with nginx + mTLS + IP allowlist (cf. `infra/nginx/conf.d/app.conf`)
- Audit all `portainer_data` volume backups (they include access tokens)

## Database Migrations (SECURITY_AUDIT.md I6)

Migrations run as a **one-shot init container** rather than on every backend
start. This pattern avoids:

- Race conditions when multiple backend replicas start simultaneously
- Re-running migrations during backend rollbacks
- Coupling startup order (container up → schema migrated)

### How it works

The same backend image is used for both jobs, controlled by the
`docker-entrypoint.sh` argument:

```sh
/app/docker-entrypoint.sh migrate   # Apply pending Diesel migrations, exit
/app/docker-entrypoint.sh backend   # Default: start the HTTP API server
```

`docker-compose.yml` defines two services:

```yaml
services:
  migrate:
    command: ["migrate"]
    restart: on-failure    # Retry on non-zero exit until DB is reachable, then stop
    depends_on:
      postgres: { condition: service_healthy }

  backend:
    depends_on:
      postgres: { condition: service_healthy }
      migrate:  { condition: service_completed_successfully }
```

Diesel `migration run` is **idempotent** — re-running on an up-to-date DB exits 0.
So restarting the backend container does NOT re-run migrations.

### Kubernetes equivalent

```yaml
spec:
  initContainers:
    - name: migrate
      image: my-registry/backend:tag
      command: ["/app/docker-entrypoint.sh", "migrate"]
  containers:
    - name: backend
      image: my-registry/backend:tag
      # default CMD ["backend"]
      # No migration step here — runs in initContainer above
```

The same image is reused; no separate migration image to maintain.

### Adding new migrations

1. Generate a new migration: `diesel migration generate <name>` (creates
   `up.sql` and `down.sql` in `backend/migrations/`)
2. Commit the files — they're read from the image, not from a volume
3. On next deploy, the `migrate` init container applies them
4. The `backend` containers start only after migrations succeed

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

## Placeholder Validation (SECURITY_AUDIT.md I7)

The backend **refuses to start** if it detects well-known placeholder values
(copied verbatim from `.env.example`) when running in a production-like
environment (`ENVIRONMENT=staging|production`).

| Value pattern | Detected in |
|---------------|-------------|
| `changeme_*` | `POSTGRES_PASSWORD`, `DATABASE_URL`, `REDIS_PASSWORD` |
| `REPLACE_WITH_*` | `JWT_SECRET`, `MASTER_ENCRYPTION_KEY`, `BLIND_INDEX_KEY`, `CSRF_SECRET_KEY`, `REFRESH_TOKEN_HASH_SALT` |

Detection happens in `backend/src/config/app_config.rs::validate()` and
returns a clear error message via `validate_or_panic()` at startup. The
application will exit 1 if any placeholder is found in a production env.

**Why this matters:** A misconfigured deployment with `JWT_SECRET=REPLACE_WITH_*`
would silently authenticate tokens using a known string — anyone could mint
admin sessions. Validation at startup prevents this from leaking to prod.

Coverage: 3 unit tests in `placeholder_tests` module.

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
