import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// Custom metrics
export const httpReqSuccessRate = new Rate('http_req_success_rate');
export const authSuccessRate = new Rate('auth_success_rate');
export const wsConnectRate = new Rate('ws_connect_rate');
export const apiLatency = new Trend('api_latency');
export const wsLatency = new Trend('ws_latency');
export const errorCount = new Counter('errors');
export const activeUsers = new Gauge('active_vus');

// Test configuration
export const options = {
  scenarios: {
    // Auth flow: register -> login -> protected -> refresh -> logout
    auth_flow: {
      executor: 'ramping-vus',
      exec: 'authFlow',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 20 },   // Warm up
        { duration: '1m', target: 50 },    // Normal load
        { duration: '2m', target: 100 },   // Stress
        { duration: '30s', target: 0 },    // Cool down
      ],
    },
    // WebSocket connections
    websocket: {
      executor: 'ramping-vus',
      exec: 'websocketFlow',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 100 },
        { duration: '2m', target: 500 },
        { duration: '30s', target: 1000 },
        { duration: '1m', target: 0 },
      ],
    },
    // General API traffic
    api_traffic: {
      executor: 'constant-vus',
      exec: 'apiTraffic',
      vus: 30,
      duration: '3m',
    },
  },
  thresholds: {
    // HTTP thresholds
    'http_req_duration': ['p(95)<1500', 'p(99)<3000'],
    'http_req_success_rate': ['rate>0.95'],
    'auth_success_rate': ['rate>0.95'],
    'ws_connect_rate': ['rate>0.90'],
    'errors': ['count<100'],
  },
  // Export summary for CI
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
};

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080/api/v1';
const WS_BASE_URL = __ENV.WS_BASE_URL || 'ws://localhost:8080/api/v1/ws';
const TEST_USER = __ENV.TEST_USER || 'loadtest@example.com';
const TEST_PASS = __ENV.TEST_PASS || 'LoadTest123!@#';

// State
let authToken = null;
let wsConnected = false;

function randomString(len = 10) {
  return Math.random().toString(36).substring(2, 2 + len);
}

function getHeaders(token = null) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  return headers;
}

export function setup() {
  // Register/login test user
  const email = `setup_${randomString()}@test.com`;
  const password = 'Setup123!@#';
  
  // Try to register
  const regRes = http.post(`${API_BASE_URL}/auth/register`, JSON.stringify({
    email, password, password_confirmation: password
  }), { headers: getHeaders() });
  
  // Login
  const loginRes = http.post(`${API_BASE_URL}/auth/login`, JSON.stringify({ email, password }), {
    headers: getHeaders(),
  });
  
  if (loginRes.status === 200) {
    return { accessToken: loginRes.json().access_token };
  }
  return {};
}

export function authFlow(data) {
  activeUsers.add(1);
  
  group('Register', () => {
    const email = `user_${__VU}_${__ITER}_${Date.now()}@test.com`;
    const password = 'Test123!@#';
    
    const res = http.post(`${API_BASE_URL}/auth/register`, JSON.stringify({
      email, password, password_confirmation: password
    }), { headers: getHeaders() });
    
    check(res, { 'register success': (r) => r.status === 201 || r.status === 409 });
    authSuccessRate.add(res.status === 201 || res.status === 409);
  });
  
  sleep(1);
  
  group('Login', () => {
    const start = Date.now();
    const res = http.post(`${API_BASE_URL}/auth/login`, JSON.stringify({
      email: TEST_USER,
      password: TEST_PASS,
    }), { headers: getHeaders() });
    
    const latency = Date.now() - start;
    apiLatency.add(latency);
    
    if (res.status === 200) {
      authToken = res.json().access_token;
      authSuccessRate.add(1);
    } else {
      authSuccessRate.add(0);
      errorCount.add(1);
    }
  });
  
  sleep(1);
  
  group('Protected Endpoint', () => {
    if (!authToken) return;
    
    const start = Date.now();
    const res = http.get(`${API_BASE_URL}/auth/me`, { headers: getHeaders(authToken) });
    const latency = Date.now() - start;
    
    apiLatency.add(latency);
    check(res, { 'me works': (r) => r.status === 200 });
    if (res.status !== 200) errorCount.add(1);
  });
  
  sleep(1);
  
  group('Refresh Token', () => {
    if (!authToken) return;
    
    const res = http.post(`${API_BASE_URL}/auth/refresh`, null, {
      headers: getHeaders(authToken),
    });
    
    if (res.status === 200) {
      authToken = res.json().access_token;
    }
  });
  
  sleep(1);
  
  group('Logout', () => {
    if (!authToken) return;
    
    const res = http.post(`${API_BASE_URL}/auth/logout`, null, {
      headers: getHeaders(authToken),
    });
    
    authToken = null;
    check(res, { 'logout ok': (r) => r.status === 200 || r.status === 204 });
  });
  
  sleep(Math.random() * 2 + 0.5);
  activeUsers.add(-1);
}

import ws from 'k6/ws';

export function websocketFlow() {
  if (!authToken) {
    const loginRes = http.post(`${API_BASE_URL}/auth/login`, JSON.stringify({
      email: TEST_USER, password: TEST_PASS
    }), { headers: getHeaders() });
    
    if (loginRes.status === 200) {
      authToken = loginRes.json().access_token;
    } else {
      errorCount.add(1);
      return;
    }
  }
  
  const wsUrl = `${__ENV.WS_BASE_URL || 'ws://localhost:8080/api/v1/ws'}?token=${authToken}`;
  
  const start = Date.now();
  ws.connect(wsUrl, {}, (socket) => {
    socket.on('open', () => {
      wsLatency.add(Date.now() - start);
      wsConnectRate.add(1);
      wsConnected = true;
      
      socket.send(JSON.stringify({ action: 'subscribe', channels: ['test'] }));
    });
    
    socket.on('message', (msg) => {
      // Handle messages
    });
    
    socket.on('close', () => {
      wsConnected = false;
    });
    
    socket.on('error', () => {
      errorCount.add(1);
      wsConnectRate.add(0);
    });
    
    socket.setTimeout(() => socket.close(), 30000);
  });
  
  sleep(Math.random() * 20 + 10);
}

export function apiTraffic() {
  const endpoints = [
    { path: '/health', weight: 30, auth: false },
    { path: '/auth/session', weight: 20, auth: true },
    { path: '/auth/me', weight: 20, auth: true },
    { path: '/metrics', weight: 10, auth: false },
    { path: '/api-docs/openapi.json', weight: 5, auth: false },
  ];
  
  const totalWeight = endpoints.reduce((s, e) => s + e.weight, 0);
  let rand = Math.random() * totalWeight;
  let selected = endpoints[0];
  
  for (const ep of endpoints) {
    rand -= ep.weight;
    if (rand <= 0) { selected = ep; break; }
  }
  
  const url = selected.auth ? `${API_BASE_URL}${selected.path}` : `${API_BASE_URL}${selected.path}`;
  const headers = selected.auth && authToken ? getHeaders(authToken) : getHeaders();
  
  const start = Date.now();
  const res = http.get(url, { headers });
  const latency = Date.now() - start;
  
  apiLatency.add(latency);
  
  const ok = check(res, { 'status < 400': (r) => r.status < 400 });
  if (!ok) errorCount.add(1);
  
  httpReqSuccessRate.add(res.status < 400);
  
  sleep(Math.random() * 1.5 + 0.2);
}

export function teardown(data) {
  console.log('Load test completed');
}