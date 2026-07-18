# Load Testing with k6

This directory contains k6 load test scripts for the Rust-Nuxt boilerplate.

## Scripts

| Script | Description |
|--------|-------------|
| `auth-flow.js` | Complete auth flow: register → login → protected endpoint → refresh → logout |
| `websocket.js` | WebSocket connection stress test (tests 10k connection limit) |
| `combined.js` | Combined scenario running auth, WS, and API traffic simultaneously |

## Quick Start

```bash
# Install k6 (macOS)
brew install k6

# Install k6 (Linux)
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install k6

# Run auth flow test
k6 run load-test/auth-flow.js

# Run websocket stress test
k6 run load-test/websocket.js

# Run combined scenario (recommended for staging)
k6 run load-test/combined.js
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BASE_URL` | `http://localhost:3000` | Frontend URL |
| `API_BASE_URL` | `http://localhost:8080/api/v1` | Backend API URL |
| `WS_BASE_URL` | `ws://localhost:8080/api/v1/ws` | WebSocket URL |
| `TEST_USER_EMAIL` | `loadtest@example.com` | Pre-created test user email |
| `TEST_USER_PASSWORD` | `LoadTest123!@#` | Pre-created test user password |

## Staging Pipeline Integration

Add to your CI/CD pipeline before production deployments:

```yaml
# .github/workflows/staging-load-test.yml
jobs:
  load-test:
    runs-on: ubuntu-latest
    needs: [deploy-staging]
    steps:
      - uses: actions/checkout@v4
      - name: Run k6 load test
        run: |
          k6 run load-test/combined.js \
            -e BASE_URL=https://staging.example.com \
            -e API_BASE_URL=https://api-staging.example.com/api/v1 \
            -e WS_BASE_URL=wss://api-staging.example.com/api/v1/ws
```

## Thresholds

| Metric | Threshold |
|--------|-----------|
| `http_req_duration` | p(95) < 1500ms |
| `http_req_success_rate` | > 95% |
| `auth_success_rate` | > 95% |
| `ws_connect_rate` | > 90% |
| `errors` | < 100 total |

## Metrics Explained

- **auth_success_rate**: Percentage of successful login/registration attempts
- **ws_connect_rate**: Percentage of successful WebSocket connections
- **api_latency**: Response time for API endpoints
- **ws_latency**: WebSocket connection establishment time
- **errors**: Total error count across all scenarios

## Interpreting Results

- **Green**: All thresholds pass → Ready for production
- **Yellow**: Some thresholds warn → Investigate bottlenecks
- **Red**: Thresholds fail → Do not deploy to production

## Capacity Planning

| Component | Tested Limit | Production Target |
|-----------|--------------|-------------------|
| WebSocket Connections | 10,000 | 5,000 (50% headroom) |
| Auth RPS | 200 | 100 |
| API RPS | 500 | 250 |
| Redis Ops/sec | 5,000 | 2,500 |