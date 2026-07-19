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
└── ssl/                     # TLS certificates (gitignored)
    ├── cert.pem             # Certificate (production)
    ├── key.pem              # Private key (production)
    └── dhparam.pem          # Diffie-Hellman parameters (gitignored, optional)
```

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
