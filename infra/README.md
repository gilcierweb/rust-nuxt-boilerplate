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
