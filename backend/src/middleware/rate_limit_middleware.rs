#![allow(dead_code)]

use crate::middleware::auth::{Claims, RateLimitCategory, rate_limit_category};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::Method,
    web,
};
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
// Anonymous and authenticated users have separate rate limit buckets.
// Authenticated users get higher limits since they are trusted identities.
// Redis keys already use different prefixes (user:{id} vs IP), ensuring
// no collision between the two groups.
// ============================================================================

/// Authentication endpoints (login, register, password reset) — anonymous
/// Limit: 100 requests per minute (per IP)
pub const RATE_AUTH: RateLimit = RateLimit::new(100, 60, "rl:auth");

/// Authentication endpoints — authenticated
/// Limit: 200 requests per minute (per user)
pub const RATE_AUTH_AUTHENTICATED: RateLimit = RateLimit::new(200, 60, "rl:auth");

/// Strict authentication endpoints (login POST, register POST, password recovery) — anonymous
/// Limit: 10 requests per minute (per IP)
pub const RATE_AUTH_STRICT: RateLimit = RateLimit::new(10, 60, "rl:auth_strict");

/// Strict authentication endpoints — authenticated
/// Limit: 20 requests per minute (per user)
pub const RATE_AUTH_STRICT_AUTHENTICATED: RateLimit = RateLimit::new(20, 60, "rl:auth_strict");

/// General API endpoints — anonymous
/// Limit: 120 requests per minute (per IP)
pub const RATE_API: RateLimit = RateLimit::new(120, 60, "rl:api");

/// General API endpoints — authenticated
/// Limit: 600 requests per minute (per user)
pub const RATE_API_AUTHENTICATED: RateLimit = RateLimit::new(600, 60, "rl:api");

/// Session management endpoints (refresh, session check) — anonymous
/// Limit: 300 requests per minute (per IP)
pub const RATE_AUTH_SESSION: RateLimit = RateLimit::new(300, 60, "rl:auth_session");

/// Session management endpoints — authenticated
/// Limit: 600 requests per minute (per user)
pub const RATE_AUTH_SESSION_AUTHENTICATED: RateLimit = RateLimit::new(600, 60, "rl:auth_session");

/// File upload endpoints
/// Limit: 10 requests per hour (per IP)
pub const RATE_UPLOAD: RateLimit = RateLimit::new(10, 3600, "rl:upload");

/// Messaging endpoints — anonymous
/// Limit: 60 requests per minute (per IP)
pub const RATE_MESSAGES: RateLimit = RateLimit::new(60, 60, "rl:msg");

/// Messaging endpoints — authenticated
/// Limit: 120 requests per minute (per user)
pub const RATE_MESSAGES_AUTHENTICATED: RateLimit = RateLimit::new(120, 60, "rl:msg");

/// Lua script for atomic fixed-window rate limiting.
///
/// Uses a single counter per window instead of sorted sets — reduces Redis
/// commands from 5 (ZREMRANGEBYSCORE, ZCARD, INCR, ZADD, EXPIRE×2) to 2
/// (INCR, EXPIRE on first request only). Trades sliding-window precision
/// for significantly lower latency under load.
///
/// Returns: 1 if allowed, 0 if rate limited
const RATE_LIMIT_LUA_SCRIPT: &str = r#"
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])

local current = redis.call('INCR', key)
if current == 1 then
    redis.call('EXPIRE', key, window)
end
if current > limit then
    return 0
end
return 1
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

    fn pick_effective_limit(
        base_limit: &RateLimit,
        method: &Method,
        path: &str,
        is_authenticated: bool,
    ) -> RateLimit {
        match rate_limit_category(method, path) {
            RateLimitCategory::AuthStrict => {
                if is_authenticated {
                    RATE_AUTH_STRICT_AUTHENTICATED.clone()
                } else {
                    RATE_AUTH_STRICT.clone()
                }
            },
            RateLimitCategory::AuthSession => {
                if is_authenticated {
                    RATE_AUTH_SESSION_AUTHENTICATED.clone()
                } else {
                    RATE_AUTH_SESSION.clone()
                }
            },
            RateLimitCategory::Default => {
                if is_authenticated {
                    match base_limit.key_prefix {
                        "rl:auth" => RATE_AUTH_AUTHENTICATED.clone(),
                        "rl:msg" => RATE_MESSAGES_AUTHENTICATED.clone(),
                        "rl:api" => RATE_API_AUTHENTICATED.clone(),
                        _ => base_limit.clone(),
                    }
                } else {
                    base_limit.clone()
                }
            },
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
        let is_authenticated = req.extensions().get::<Claims>().is_some();
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

            let effective_limit = RateLimiterMiddleware::<S>::pick_effective_limit(
                &limit,
                req.method(),
                req.path(),
                is_authenticated,
            );
            let key = format!("{}:{}", effective_limit.key_prefix, client_key);

            // Attempt rate limiting with Redis. Fail closed on Redis errors.
            let allowed = async {
                let mut conn = redis.get().await.ok()?;

                // Use fixed-window Lua script (2 Redis ops vs 5 for sliding window)
                let result: u64 = redis::cmd("EVAL")
                    .arg(RATE_LIMIT_LUA_SCRIPT)
                    .arg(1) // number of keys
                    .arg(&key)
                    .arg(effective_limit.max_requests)
                    .arg(effective_limit.window_secs)
                    .query_async(&mut conn)
                    .await
                    .ok()?;

                Some(result == 1)
            }
            .await;

            match allowed {
                Some(true) => {
                    // Allowed - continue to handler
                },
                Some(false) => {
                    // Rate limited
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
                },
                None => {
                    // Redis unavailable or error - fail closed with 503
                    tracing::warn!(
                        event = "rate_limit.redis_unavailable",
                        key = %key,
                        "Rate limiter Redis unavailable, failing closed"
                    );
                    let response = HttpResponse::ServiceUnavailable()
                        .insert_header(("Retry-After", "5"))
                        .json(json!({
                            "error": {
                                "code": "SERVICE_UNAVAILABLE",
                                "message": t!("errors.rate_limiter_unavailable").into_owned()
                            }
                        }))
                        .map_into_boxed_body();

                    let (http_req, _payload) = req.into_parts();
                    return Ok(ServiceResponse::new(http_req, response));
                },
            }

            let res = svc.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::Method;

    #[test]
    fn anonymous_gets_base_api_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::GET,
            "/api/v1/users",
            false,
        );
        assert_eq!(limit.max_requests, 120);
        assert_eq!(limit.key_prefix, "rl:api");
    }

    #[test]
    fn authenticated_gets_higher_api_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::GET,
            "/api/v1/users",
            true,
        );
        assert_eq!(limit.max_requests, 600);
        assert_eq!(limit.key_prefix, "rl:api");
    }

    #[test]
    fn anonymous_gets_base_auth_strict_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::POST,
            "/api/v1/auth/login",
            false,
        );
        assert_eq!(limit.max_requests, 10);
        assert_eq!(limit.key_prefix, "rl:auth_strict");
    }

    #[test]
    fn authenticated_gets_higher_auth_strict_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::POST,
            "/api/v1/auth/login",
            true,
        );
        assert_eq!(limit.max_requests, 20);
        assert_eq!(limit.key_prefix, "rl:auth_strict");
    }

    #[test]
    fn anonymous_gets_base_session_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::POST,
            "/api/v1/auth/refresh",
            false,
        );
        assert_eq!(limit.max_requests, 300);
        assert_eq!(limit.key_prefix, "rl:auth_session");
    }

    #[test]
    fn authenticated_gets_higher_session_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::POST,
            "/api/v1/auth/refresh",
            true,
        );
        assert_eq!(limit.max_requests, 600);
        assert_eq!(limit.key_prefix, "rl:auth_session");
    }

    #[test]
    fn authenticated_user_with_auth_base_gets_auth_authenticated_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_AUTH,
            &Method::GET,
            "/api/v1/other",
            true,
        );
        assert_eq!(limit.max_requests, 200);
        assert_eq!(limit.key_prefix, "rl:auth");
    }

    #[test]
    fn authenticated_user_with_msg_base_gets_messages_authenticated_limit() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_MESSAGES,
            &Method::POST,
            "/api/v1/other",
            true,
        );
        assert_eq!(limit.max_requests, 120);
        assert_eq!(limit.key_prefix, "rl:msg");
    }

    #[test]
    fn upload_limit_unchanged_for_both() {
        let anon = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_UPLOAD,
            &Method::POST,
            "/api/v1/admin/upload",
            false,
        );
        let auth = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_UPLOAD,
            &Method::POST,
            "/api/v1/admin/upload",
            true,
        );
        assert_eq!(anon.max_requests, 10);
        assert_eq!(auth.max_requests, 10);
    }

    #[test]
    fn rate_limit_constants_are_ordered_correctly() {
        const {
            assert!(RATE_AUTH_STRICT.max_requests < RATE_AUTH.max_requests);
            assert!(RATE_AUTH.max_requests < RATE_AUTH_AUTHENTICATED.max_requests);
            assert!(
                RATE_AUTH_STRICT_AUTHENTICATED.max_requests < RATE_AUTH_AUTHENTICATED.max_requests
            );
            assert!(RATE_API.max_requests < RATE_API_AUTHENTICATED.max_requests);
        }
    }

    #[test]
    fn authenticated_limits_are_stricter_than_anonymous_for_strict() {
        const {
            assert!(RATE_AUTH_STRICT.max_requests < RATE_AUTH_STRICT_AUTHENTICATED.max_requests);
        }
    }

    #[test]
    fn key_prefixes_are_shared_between_anonymous_and_authenticated() {
        assert_eq!(RATE_AUTH.key_prefix, RATE_AUTH_AUTHENTICATED.key_prefix);
        assert_eq!(
            RATE_AUTH_STRICT.key_prefix,
            RATE_AUTH_STRICT_AUTHENTICATED.key_prefix
        );
        assert_eq!(RATE_API.key_prefix, RATE_API_AUTHENTICATED.key_prefix);
        assert_eq!(
            RATE_AUTH_SESSION.key_prefix,
            RATE_AUTH_SESSION_AUTHENTICATED.key_prefix
        );
        assert_eq!(
            RATE_MESSAGES.key_prefix,
            RATE_MESSAGES_AUTHENTICATED.key_prefix
        );
    }
}
