import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Metrics
export const wsConnectRate = new Rate('ws_connect_success_rate');
export const wsConnectLatency = new Trend('ws_connect_latency');
export const wsMessageLatency = new Trend('ws_message_latency');
export const wsErrors = new Counter('ws_errors');

// Configuration
export const options = {
  scenarios: {
    websocket_load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 50 },   // Warm up
        { duration: '1m', target: 200 },   // Normal load
        { duration: '2m', target: 1000 },  // Stress test - approach 10k limit
        { duration: '30s', target: 5000 }, // Push towards 10k limit
        { duration: '1m', target: 2000 },  // Sustained
        { duration: '30s', target: 0 },    // Ramp down
      ],
    },
  },
  thresholds: {
    'ws_connect_success_rate': ['rate>0.90'],
    'ws_connect_latency': ['p(95)<2000'],
    'ws_errors': ['count<100'],
  },
};

const WS_BASE_URL = __ENV.WS_BASE_URL || 'ws://localhost:8080/api/v1/ws';
const API_BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080/api/v1';
const TEST_EMAIL = __ENV.TEST_USER_EMAIL || 'wsloadtest@example.com';
const TEST_PASSWORD = __ENV.TEST_USER_PASSWORD || 'WsLoadTest123!@#';

let authToken = '';

function getAuthToken() {
  if (authToken) return authToken;
  
  const res = http.post(`${API_BASE_URL}/auth/login`, JSON.stringify({
    email: TEST_EMAIL,
    password: TEST_PASSWORD,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  if (res.status === 200) {
    const body = res.json();
    authToken = body.access_token;
    return authToken;
  }
  return null;
}

import http from 'k6/http';

export default function () {
  const token = getAuthToken();
  if (!token) {
    console.error('Failed to get auth token');
    return;
  }
  
  const wsUrl = `${__ENV.WS_BASE_URL || 'ws://localhost:8080/api/v1/ws'}?token=${token}`;
  
  const wsRes = ws.connect(wsUrl, {}, function (socket) {
    socket.on('open', () => {
      console.log('WebSocket connected');
      
      // Subscribe to channels
      socket.send(JSON.stringify({
        action: 'subscribe',
        channels: ['notifications', 'presence'],
      }));
    });
    
    socket.on('message', (msg) => {
      try {
        const data = JSON.parse(msg);
        if (data.type === 'welcome') {
          wsConnectRate.add(1);
        }
      } catch (e) {
        // Ignore
      }
    });
    
    socket.on('error', (e) => {
      console.error('WS Error:', e.error());
      wsErrors.add(1);
    });
    
    socket.on('close', () => {
      // Connection closed
    });
    
    // Send periodic ping
    const pingInterval = setInterval(() => {
      socket.send(JSON.stringify({ action: 'ping', timestamp: Date.now() }));
    }, 5000);
    
    socket.setTimeout(() => {
      clearInterval(pingInterval);
      socket.close();
    }, 60000); // Keep alive for 60 seconds
  });
  
  check(wsRes, { 'WS connected': (r) => r && r.status === 101 });
  
  sleep(Math.random() * 10 + 5);
}