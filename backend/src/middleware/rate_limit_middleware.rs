#![allow(dead_code)]

use actix_web::{
    Error, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::Method,
    web,
    HttpMessage,
};
use crate::middleware::auth::{Claims, RateLimitCategory, rate_limit_category};
use futures::future::{LocalBoxFuture, Ready, ready};
use serde_json::json;
use std::rc::Rc;

use crate::{AppState, repositories::container::AppContainer};

const CSRF_COOKIE_NAME: &str = "csrf_token";

/// Rate limit configuration.
/// 
/// # Fields
/// * `max_requests` - Maximum number of requests allowed within the window
/// * `window_secs` - Time window in seconds for the rate limit
/// * `key_prefix` - Prefix for Redis keys used to track requests
#[derive(Clone, Debug)]
pub struct RateLimit {
    pub max_requests: u64,
    pub window_secs: u64,
    pub key_prefix: &'static str,
}

impl RateLimit {
    pub const fn new(max_requests: u64, window_secs: u64, key_prefix: &'static str) -> Self {
        Self {
            max_requests,
            window_secs,
            key_prefix,
        }
    }
}

// ============================================================================
// Rate Limit Configurations
// ============================================================================
// These constants define the rate limits for different parts of the API.
// All limits are enforced using Redis-backed sliding window counters.
// ============================================================================

/// Authentication endpoints (login, register, password reset)
/// Limit: 100 requests per minute (per IP)
/// Use case: Standard auth operations with moderate protection
pub const RATE_AUTH: RateLimit = RateLimit::new(100, 60, "rl:auth");

/// Strict authentication endpoints (login POST, register POST, password recovery)
/// Limit: 10 requests per minute (per IP)
/// Use case: Write operations that are susceptible to brute force attacks
pub const RATE_AUTH_STRICT: RateLimit = RateLimit::new(10, 60, "rl:auth_strict");

/// General API endpoints
/// Limit: 120 requests per minute (per IP)
/// Use case: Standard API operations with relaxed limits
pub const RATE_API: RateLimit = RateLimit::new(120, 60, "rl:api");

/// Session management endpoints (refresh, session check)
/// Limit: 300 requests per minute (per IP)
/// Use case: High-frequency session operations with minimal restriction
pub const RATE_AUTH_SESSION: RateLimit = RateLimit::new(300, 60, "rl:auth_session");

/// File upload endpoints
/// Limit: 10 requests per hour (per IP)
/// Use case: Resource-intensive operations requiring strict limits
pub const RATE_UPLOAD: RateLimit = RateLimit::new(10, 3600, "rl:upload");

/// Messaging endpoints
/// Limit: 60 requests per minute (per IP)
/// Use case: User messaging with spam prevention
pub const RATE_MESSAGES: RateLimit = RateLimit::new(60, 60, "rl:msg");

/// Lua script for atomic sliding window rate limiting
/// Uses sorted sets to track request timestamps
/// Returns: 1 if allowed, 0 if rate limited
const RATE_LIMIT_LUA_SCRIPT: &str = r#"
local key = KEYS[1]
local window = tonumber(ARGV[1])
local max_requests = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

-- Remove expired entries from the sorted set
redis.call('ZREMRANGEBYSCORE', key, 0, now - window)

-- Count current requests in window
local current = redis.call('ZCARD', key)

if current < max_requests then
    -- Add current request with unique member using atomic INCR counter
    local seq = redis.call('INCR', key .. ':seq')
    redis.call('ZADD', key, now, now .. ':' .. seq)
    redis.call('EXPIRE', key, window)
    redis.call('EXPIRE', key .. ':seq', window)
    return 1
else
    -- Rate limited, still set expiry for cleanup
    redis.call('EXPIRE', key, window)
    return 0
end
"#;

pub struct RateLimiter {
    redis: deadpool_redis::Pool,
    limit: RateLimit,
}

impl RateLimiter {
    pub fn new(redis: deadpool_redis::Pool, limit: RateLimit) -> Self {
        Self { redis, limit }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service: Rc::new(service),
            redis: self.redis.clone(),
            limit: self.limit.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    redis: deadpool_redis::Pool,
    limit: RateLimit,
}

impl<S> RateLimiterMiddleware<S> {
    fn new(service: Rc<S>, redis: deadpool_redis::Pool, limit: RateLimit) -> Self {
        Self {
            service,
            redis,
            limit,
        }
    }

    fn pick_effective_limit(base_limit: &RateLimit, method: &Method, path: &str) -> RateLimit {
        match rate_limit_category(method, path) {
            RateLimitCategory::AuthStrict => RATE_AUTH_STRICT.clone(),
            RateLimitCategory::AuthSession => RATE_AUTH_SESSION.clone(),
            RateLimitCategory::Default => base_limit.clone(),
        }
    }
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let redis = self.redis.clone();
        let limit = self.limit.clone();
        let rate_limit_enabled = req
            .app_data::<web::Data<AppContainer>>()
            .map(|container| container.config.rate_limit_enabled)
            .or_else(|| {
                req.app_data::<web::Data<AppState>>()
                    .map(|state| state.config.rate_limit_enabled)
            })
            .unwrap_or(true); // Default to enabled for safety

        // Try to extract user_id from JWT claims (set by JwtAuth middleware)
        // If authenticated, use user_id for rate limiting; otherwise fall back to IP
        let client_key = req
            .extensions()
            .get::<Claims>()
            .map(|claims| format!("user:{}", claims.profile_id))
            .unwrap_or_else(|| {
                req.peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            });

        Box::pin(async move {
            if !rate_limit_enabled {
                let res = svc.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            let effective_limit =
                RateLimiterMiddleware::<S>::pick_effective_limit(&limit, req.method(), req.path());
            let key = format!("{}:{}", effective_limit.key_prefix, client_key);

            let allowed = async {
                let mut conn = redis.get().await.ok()?;

                // Use Lua script for atomic sliding window rate limiting
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();

                let result: u64 = redis::cmd("EVAL")
                    .arg(RATE_LIMIT_LUA_SCRIPT)
                    .arg(1) // number of keys
                    .arg(&key)
                    .arg(effective_limit.window_secs)
                    .arg(effective_limit.max_requests)
                    .arg(now)
                    .query_async(&mut conn)
                    .await
                    .ok()?;

                Some(result == 1)
            }
            .await
            .unwrap_or(true);

            if !allowed {
                let response = HttpResponse::TooManyRequests()
                    .insert_header(("Retry-After", effective_limit.window_secs.to_string()))
                    .insert_header((
                        "X-RateLimit-Limit",
                        effective_limit.max_requests.to_string(),
                    ))
                    .json(json!({
                        "error": {
                            "code": "RATE_LIMITED",
                            "message": t!("errors.rate_limited")
                        }
                    }))
                    .map_into_boxed_body();

                let (http_req, _payload) = req.into_parts();
                return Ok(ServiceResponse::new(http_req, response));
            }

            let res = svc.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}
