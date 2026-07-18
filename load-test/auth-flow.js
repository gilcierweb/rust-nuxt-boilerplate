import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
export const loginSuccessRate = new Rate('login_success_rate');
export const registerSuccessRate = new Rate('register_success_rate');
export const wsConnectSuccessRate = new Rate('ws_connect_success_rate');
export const loginDuration = new Trend('login_duration');
export const registerDuration = new Trend('register_duration');
export const wsConnectDuration = new Trend('ws_connect_duration');
export const apiRequestDuration = new Trend('api_request_duration');
export const errors = new Counter('errors');

// Test configuration
export const options = {
  scenarios: {
    auth_flow: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 50 },   // Ramp up to 50 users
        { duration: '1m', target: 100 },   // Stay at 100 users
        { duration: '30s', target: 200 },  // Spike to 200 users
        { duration: '1m', target: 100 },   // Back to 100
        { duration: '30s', target: 0 },    // Ramp down
      ],
      exec: 'authFlow',
    },
    websocket_connections: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 100 },  // 100 WS connections
        { duration: '2m', target: 500 },   // Up to 500 (test 10k limit)
        { duration: '30s', target: 0 },
      ],
      exec: 'websocketFlow',
    },
    api_traffic: {
      executor: 'constant-vus',
      vus: 50,
      duration: '3m',
      exec: 'apiTraffic',
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<1000'],           // 95th percentile < 1s
    'login_success_rate': ['rate>0.95'],           // >95% success
    'register_success_rate': ['rate>0.95'],
    'ws_connect_success_rate': ['rate>0.95'],
    'errors': ['count<50'],                        // <50 total errors
  },
};

// Base URLs - can be overridden via environment variables
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080/api/v1';
const WS_BASE_URL = __ENV.WS_BASE_URL || 'ws://localhost:8080/api/v1';

// Generate random email for registration
function randomEmail() {
  return `loadtest_${Date.now()}_${Math.random().toString(36).substring(7)}@test.com`;
}

// Shared test user for login tests
const TEST_USER_EMAIL = __ENV.TEST_USER_EMAIL || 'loadtest@example.com';
const TEST_USER_PASSWORD = __ENV.TEST_USER_PASSWORD || 'LoadTest123!@#';

// Auth token storage (in-memory per VU)
let authToken = null;
let refreshToken = null;

export function setup() {
  // Pre-create test user if needed
  const payload = JSON.stringify({
    email: TEST_USER_EMAIL,
    password: TEST_USER_PASSWORD,
    password_confirmation: TEST_USER_PASSWORD,
  });
  
  const params = { headers: { 'Content-Type': 'application/json' } };
  
  // Try to register the test user (might already exist)
  http.post(`${API_BASE_URL}/auth/register`, payload, params);
  
  // Login to get tokens
  const loginRes = http.post(`${API_BASE_URL}/auth/login`, 
    JSON.stringify({ email: TEST_USER_EMAIL, password: TEST_USER_PASSWORD }), 
    params
  );
  
  if (loginRes.status === 200) {
    const body = loginRes.json();
    return { 
      accessToken: body.access_token,
      refreshToken: loginRes.cookies?.refresh_token?.[0]?.value 
    };
  }
  return {};
}

export function authFlow(data) {
  const params = { 
    headers: { 
      'Content-Type': 'application/json',
      ...(data.accessToken && { 'Authorization': `Bearer ${data.accessToken}` })
    },
    tags: { test_type: 'auth_flow' },
  };

  // 1. Register new user
  const registerStart = Date.now();
  const regPayload = JSON.stringify({
    email: randomEmail(),
    password: 'LoadTest123!@#',
    password_confirmation: 'LoadTest123!@#',
  });
  
  const regRes = http.post(`${API_BASE_URL}/auth/register`, regPayload, params);
  const regDuration = Date.now() - registerStart;
  
  registerSuccessRate.add(regRes.status === 201 || regRes.status === 409);
  registerDuration.add(regDuration);
  
  if (regRes.status >= 400 && regRes.status !== 409) {
    errors.add(1);
  }
  
  sleep(1);

  // 2. Login
  const loginStart = Date.now();
  const loginPayload = JSON.stringify({
    email: TEST_USER_EMAIL,
    password: TEST_USER_PASSWORD,
  });
  
  const loginRes = http.post(`${API_BASE_URL}/auth/login`, loginPayload, params);
  const loginDurationMs = Date.now() - loginStart;
  
  loginSuccessRate.add(loginRes.status === 200);
  loginDuration.add(loginDurationMs);
  
  if (loginRes.status === 200) {
    const body = loginRes.json();
    authToken = body.access_token;
    // Note: refresh token is in cookie, not in body
  } else {
    errors.add(1);
  }
  
  sleep(1);

  // 3. Access protected endpoint (if we have token)
  if (authToken) {
    const meStart = Date.now();
    const meRes = http.get(`${API_BASE_URL}/auth/me`, {
      headers: { 
        'Authorization': `Bearer ${authToken}`,
        'Content-Type': 'application/json',
      },
      tags: { test_type: 'auth_flow' },
    });
    const meDuration = Date.now() - meStart;
    
    check(meRes, { 'me endpoint works': (r) => r.status === 200 });
    apiRequestDuration.add(meDuration);
    
    if (meRes.status !== 200) errors.add(1);
  }
  
  sleep(1);

  // 4. Refresh token
  if (authToken) {
    const refreshRes = http.post(`${API_BASE_URL}/auth/refresh`, null, {
      headers: { 
        'Authorization': `Bearer ${authToken}`,
        'Content-Type': 'application/json',
      },
      tags: { test_type: 'auth_flow' },
    });
    
    if (refreshRes.status === 200) {
      const body = refreshRes.json();
      authToken = body.access_token;
    }
  }
  
  sleep(1);

  // 5. Logout
  if (authToken) {
    http.post(`${API_BASE_URL}/auth/logout`, null, {
      headers: { 
        'Authorization': `Bearer ${authToken}`,
        'Content-Type': 'application/json',
      },
      tags: { test_type: 'auth_flow' },
    });
  }
  
  sleep(Math.random() * 2);
}

export function websocketFlow() {
  if (!authToken) return;
  
  const wsUrl = `${WS_BASE_URL}/ws?token=${authToken}`;
  
  const wsStart = Date.now();
  const ws = new WebSocket(wsUrl);
  
  ws.onopen = () => {
    wsConnectDuration.add(Date.now() - wsStart);
    wsConnectSuccessRate.add(1);
    
    // Send ping to keep connection alive
    ws.send(JSON.stringify({ action: 'ping' }));
  };
  
  ws.onerror = () => {
    errors.add(1);
    wsConnectSuccessRate.add(0);
  };
  
  ws.onclose = () => {
    wsConnectSuccessRate.add(0);
  };
  
  ws.onmessage = (event) => {
    // Handle incoming messages
    try {
      const msg = JSON.parse(event.data);
      if (msg.type === 'pong') {
        ws.send(JSON.stringify({ action: 'ping' }));
      }
    } catch (e) {
      // Ignore parse errors
    }
  };
  
  // Keep connection alive for test duration
  sleep(Math.random() * 30 + 10);
  
  ws.close();
}

export function apiTraffic() {
  const endpoints = [
    { path: '/auth/session', weight: 10 },
    { path: '/auth/me', weight: 10 },
    { path: '/health', weight: 5 },
    { path: '/metrics', weight: 2 },
  ];
  
  const params = {
    headers: { 
      'Content-Type': 'application/json',
      ...(authToken && { 'Authorization': `Bearer ${authToken}` })
    },
    tags: { test_type: 'api_traffic' },
  };
  
  // Pick random endpoint based on weights
  const totalWeight = endpoints.reduce((sum, e) => sum + e.weight, 0);
  let random = Math.random() * totalWeight;
  let selectedEndpoint = endpoints[0];
  
  for (const endpoint of endpoints) {
    random -= endpoint.weight;
    if (random <= 0) {
      selectedEndpoint = endpoint;
      break;
    }
  }
  
  const start = Date.now();
  const res = http.get(`${API_BASE_URL}${selectedEndpoint.path}`, {
    headers: params.headers,
    tags: params.tags,
  });
  const duration = Date.now() - start;
  
  apiRequestDuration.add(duration);
  
  if (res.status >= 400) {
    errors.add(1);
  }
  
  sleep(Math.random() * 2 + 0.5);
}

export function teardown(data) {
  // Cleanup if needed
  console.log('Load test completed');
}