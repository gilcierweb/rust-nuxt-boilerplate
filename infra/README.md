# Infrastructure Configuration

This directory contains infrastructure-as-code configurations for the production stack.

## Structure

```
infra/
├── nginx/
│   ├── nginx.conf           # Main Nginx configuration
│   └── conf.d/
│       ├── upstream.conf    # Upstream backend/frontend definitions
│       ├── ssl.conf         # SSL/TLS settings
│       ├── security.conf    # Security headers
│       └── rate-limit.conf  # Rate limiting rules
├── prometheus/
│   └── prometheus.yml       # Prometheus scrape targets
├── grafana/
│   ├── dashboards/          # JSON dashboard definitions
│   └── provisioning/        # Grafana datasource/dashboards provisioning
└── ssl/                     # TLS certificates (gitignored)
    ├── cert.pem             # Certificate (production)
    └── key.pem              # Private key (production)
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