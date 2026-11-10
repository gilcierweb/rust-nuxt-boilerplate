#![allow(dead_code)]

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::Method;
use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage, HttpResponse, web};
use futures::future::{LocalBoxFuture, Ready, ready};
use serde_json::json;

use crate::AppState;
use crate::middleware::auth::{Claims, RateLimitCategory, rate_limit_category};
use crate::repositories::container::AppContainer;

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

    /// Brute-force-sensitive endpoints (login, register, password recovery).
    ///
    /// These must never bypass rate limiting — not even when Redis is down.
    /// An attacker who can take Redis offline must not gain unlimited login
    /// attempts, so AuthStrict requests always fail CLOSED (503) instead of
    /// following the circuit breaker's fail-open path.
    pub fn is_strict(&self) -> bool {
        self.key_prefix == RATE_AUTH_STRICT.key_prefix
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

/// Brute-force-sensitive endpoints charged per subnet aggregate in addition to
/// the per-IP budget. Blunts IP rotation: a bot cycling the last octet of an
/// IPv4 /24 (or roaming an IPv6 /48) hits this aggregate ceiling even though
/// every individual address stays under the per-IP limit.
pub const RATE_AUTH_STRICT_SUBNET: RateLimit = RateLimit::new(30, 60, "rl:auth_strict_mask");

/// Global daily registration ceiling. Rate limiting restricts *time*, not
/// *volume*: without an aggregate cap a distributed registration campaign can
/// still poison the users table with ~14k rows/day per IP. This bucket is
/// process-wide (single Redis key) and only charged by register paths.
pub const REGISTER_GLOBAL_KEY: &str = "rl:register_global";
pub const SECONDS_PER_DAY: u64 = 86_400;
pub const DEFAULT_REGISTER_DAILY_LIMIT: u64 = 2_000;

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

// ============================================================================
// Circuit Breaker for Redis Failures
// ============================================================================

/// Circuit breaker state for graceful degradation when Redis is unavailable.
///
/// When Redis becomes unavailable, failing closed (503) on every request
/// creates a denial-of-service vector. This circuit breaker:
///
/// 1. Tracks consecutive Redis failures
/// 2. After `failure_threshold` consecutive errors, opens the circuit
///    and starts failing OPEN (allowing requests to pass) for `reset_timeout`
/// 3. After `reset_timeout`, enters half-open state and tries Redis again
/// 4. On success, closes the circuit; on failure, reopens it
///
/// This trades rate-limiting accuracy for availability during Redis blips.
/// The impact is bounded: at most `reset_timeout` seconds of untracked requests
/// per circuit-open event.
///
/// EXCEPTION: brute-force-sensitive endpoints (`RateLimit::is_strict()`,
/// i.e. login/register/recovery) always fail CLOSED (503) while Redis is
/// unavailable, even with the circuit open. Availability must not become a
/// brute-force bypass.
#[derive(Debug)]
pub struct RedisCircuitBreaker {
    /// Number of consecutive Redis failures
    consecutive_failures: AtomicU64,
    /// Threshold to open the circuit (default: 5)
    failure_threshold: u64,
    /// When the circuit was opened (None if closed)
    opened_at: std::sync::Mutex<Option<Instant>>,
    /// How long to stay open before retrying (default: 30s)
    reset_timeout: Duration,
}

impl RedisCircuitBreaker {
    pub fn new(failure_threshold: u64, reset_timeout: Duration) -> Self {
        Self {
            consecutive_failures: AtomicU64::new(0),
            failure_threshold,
            opened_at: std::sync::Mutex::new(None),
            reset_timeout,
        }
    }

    /// Check if the circuit is open (should fail open).
    ///
    /// Returns `true` if the circuit is open and the reset timeout hasn't elapsed.
    /// After the timeout, the circuit enters half-open state and returns `false`
    /// to allow a retry.
    pub fn is_open(&self) -> bool {
        let opened_at = self.opened_at.lock().unwrap();
        if let Some(t) = *opened_at {
            // Circuit was opened — check if reset_timeout has elapsed
            if t.elapsed() < self.reset_timeout {
                return true;
            }
        }
        false
    }

    /// Record a successful Redis operation.
    ///
    /// Closes the circuit if it was open, resets the failure counter.
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        let mut opened_at = self.opened_at.lock().unwrap();
        *opened_at = None;
    }

    /// Record a Redis failure.
    ///
    /// Increments the failure counter and opens the circuit if the threshold
    /// is reached.
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.failure_threshold {
            let mut opened_at = self.opened_at.lock().unwrap();
            if opened_at.is_none() {
                tracing::warn!(
                    event = "rate_limit.circuit_breaker_opened",
                    consecutive_failures = failures,
                    threshold = self.failure_threshold,
                    reset_timeout_secs = self.reset_timeout.as_secs(),
                    "Rate limiter circuit breaker opened due to consecutive Redis failures"
                );
                *opened_at = Some(Instant::now());
            }
        }
    }
}

impl Default for RedisCircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30))
    }
}

/// Lua script for atomic fixed-window rate limiting.
///
/// Uses a single counter per window instead of sorted sets — reduces Redis
/// commands from 5 (ZREMRANGEBYSCORE, ZCARD, INCR, ZADD, EXPIRE×2) to 2
/// (INCR, EXPIRE on first request only). Trades sliding-window precision
/// for significantly lower latency under load.
///
/// Returns the counter after this request together with the TTL of the window,
/// so the caller can enforce the limit and answer the quota headers without a
/// second round trip.
const RATE_LIMIT_LUA_SCRIPT: &str = r#"
local key = KEYS[1]
local window = tonumber(ARGV[1])

local current = redis.call('INCR', key)
if current == 1 then
    redis.call('EXPIRE', key, window)
end
return {current, redis.call('TTL', key)}
"#;

pub struct RateLimiter {
    redis: deadpool_redis::Pool,
    limit: RateLimit,
    circuit_breaker: Arc<RedisCircuitBreaker>,
    charge_scope: ChargeScope,
}

impl RateLimiter {
    pub fn new(redis: deadpool_redis::Pool, limit: RateLimit) -> Self {
        Self {
            redis,
            limit,
            circuit_breaker: Arc::new(RedisCircuitBreaker::default()),
            charge_scope: ChargeScope::All,
        }
    }

    /// Rate limiter mounted on a nested scope that already lives under a
    /// scope-wide limiter (the router mounts one on `/api/v1`). Category-derived
    /// buckets (`AuthStrict`, `AuthSession`) resolve to the same Redis key
    /// regardless of which instance charges them, so charging them here would
    /// consume the outer budget twice per request. This variant charges only
    /// the `Default` category, whose key prefix comes from the base limit.
    pub fn default_only(redis: deadpool_redis::Pool, limit: RateLimit) -> Self {
        Self {
            redis,
            limit,
            circuit_breaker: Arc::new(RedisCircuitBreaker::default()),
            charge_scope: ChargeScope::DefaultOnly,
        }
    }

    /// Create a RateLimiter with a custom circuit breaker configuration.
    pub fn with_circuit_breaker(
        redis: deadpool_redis::Pool,
        limit: RateLimit,
        failure_threshold: u64,
        reset_timeout: Duration,
    ) -> Self {
        Self {
            redis,
            limit,
            circuit_breaker: Arc::new(RedisCircuitBreaker::new(failure_threshold, reset_timeout)),
            charge_scope: ChargeScope::All,
        }
    }
}

/// Which rate-limit categories a limiter instance charges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeScope {
    /// Charge every category (scope-wide / outer limiters).
    All,
    /// Charge only `Default`; higher-cost categories are left to the outer
    /// limiter (single-charge rule, see [`RateLimiter::default_only`]).
    DefaultOnly,
}

/// Whether a limiter instance with `scope` must charge `category`.
fn charges(scope: ChargeScope, category: RateLimitCategory) -> bool {
    match scope {
        ChargeScope::All => true,
        ChargeScope::DefaultOnly => category == RateLimitCategory::Default,
    }
}

/// One Redis bucket the limiter must evaluate for a request.
#[derive(Debug, Clone)]
struct Charge {
    key: String,
    limit: RateLimit,
}

/// Compact per-network bucket key: IPv4 masks the last octet (/24), IPv6
/// keeps the first three hextets (/48).
fn subnet_bucket_key(ip: &std::net::IpAddr) -> String {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            format!("{}.{}.{}.mask", octets[0], octets[1], octets[2])
        },
        std::net::IpAddr::V6(v6) => {
            let segments = v6.segments();
            format!("{:x}:{:x}:{:x}.mask", segments[0], segments[1], segments[2])
        },
    }
}

fn is_register_path(path: &str) -> bool {
    path == "/api/v1/auth/register" || path == "/api/v1/auth/register/"
}

/// Buckets charged by a brute-force-sensitive request.
///
/// 1. exact client budget — full per-IP limit (`effective_limit`);
/// 2. subnet aggregate — `RATE_AUTH_STRICT_SUBNET`, shared by rotating IPs;
/// 3. register only: the global daily ceiling configured as
///    `REGISTER_DAILY_LIMIT` (default 2000/day), one key for the whole app.
fn strict_charge_buckets(
    effective_limit: &RateLimit,
    client_ip: Option<std::net::IpAddr>,
    path: &str,
    register_daily_limit: u64,
) -> Vec<Charge> {
    let mut buckets = Vec::with_capacity(3);

    let exact_key = match client_ip {
        Some(ip) => format!("{}:{}", effective_limit.key_prefix, ip),
        None => format!("{}:unknown", effective_limit.key_prefix),
    };
    buckets.push(Charge {
        key: exact_key,
        limit: effective_limit.clone(),
    });

    if let Some(ip) = client_ip {
        buckets.push(Charge {
            key: format!(
                "{}:{}",
                RATE_AUTH_STRICT_SUBNET.key_prefix,
                subnet_bucket_key(&ip)
            ),
            limit: RATE_AUTH_STRICT_SUBNET,
        });
    }

    if is_register_path(path) {
        buckets.push(Charge {
            key: REGISTER_GLOBAL_KEY.to_string(),
            limit: RateLimit::new(register_daily_limit, SECONDS_PER_DAY, "rl:register_global"),
        });
    }

    buckets
}

/// Buckets charged by an arbitrary request: the multi-dimensional list for
/// strict endpoints, the single base bucket for everything else.
fn charges_for_request(
    effective_limit: &RateLimit,
    path: &str,
    client_key: &str,
    client_ip: Option<std::net::IpAddr>,
    register_daily_limit: u64,
) -> Vec<Charge> {
    if effective_limit.is_strict() {
        strict_charge_buckets(effective_limit, client_ip, path, register_daily_limit)
    } else {
        vec![Charge {
            key: format!("{}:{}", effective_limit.key_prefix, client_key),
            limit: effective_limit.clone(),
        }]
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
            circuit_breaker: self.circuit_breaker.clone(),
            charge_scope: self.charge_scope,
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    redis: deadpool_redis::Pool,
    limit: RateLimit,
    circuit_breaker: Arc<RedisCircuitBreaker>,
    charge_scope: ChargeScope,
}

impl<S> RateLimiterMiddleware<S> {
    fn new(
        service: Rc<S>,
        redis: deadpool_redis::Pool,
        limit: RateLimit,
        circuit_breaker: Arc<RedisCircuitBreaker>,
    ) -> Self {
        Self {
            service,
            redis,
            limit,
            circuit_breaker,
            charge_scope: ChargeScope::All,
        }
    }

    /// Redis bucket key owner for a request.
    ///
    /// Authenticated traffic is throttled per profile so a shared NAT cannot
    /// exhaust the budget of other users. Anonymous traffic falls back to the
    /// **real client IP** - the one `extract_client_ip` derives from the peer
    /// address (or `X-Forwarded-For` / `Forwarded` when the peer is a trusted
    /// proxy). Using the raw `peer_addr()` instead keyed every anonymous
    /// request behind a reverse proxy onto the proxy's own IP, collapsing the
    /// per-client limit into one shared bucket.
    fn rate_limit_client_key(
        client_ip: Option<std::net::IpAddr>,
        claims: Option<&Claims>,
    ) -> String {
        match claims {
            Some(claims) => format!("user:{}", claims.profile_id),
            None => match client_ip {
                Some(ip) => ip.to_string(),
                None => "unknown".to_string(),
            },
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

/// Configuration the middleware reads, from whichever container is registered.
///
/// The bootstrap registers `AppState` while the scoped controllers register
/// `AppContainer`. The container wins when both are present, so a narrower
/// wiring cannot shadow the process configuration. `None` means the middleware
/// was mounted outside both (unit tests), and the caller applies a safe default.
///
/// The Arc is cloned instead of borrowing because the request is moved into
/// the handler further down `call`.
fn middleware_config(req: &ServiceRequest) -> Option<Arc<crate::config::AppConfig>> {
    if let Some(container) = req.app_data::<web::Data<AppContainer>>() {
        return Some(container.config.clone());
    }
    req.app_data::<web::Data<AppState>>()
        .map(|state| state.config.clone())
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
        let circuit_breaker = self.circuit_breaker.clone();
        let charge_scope = self.charge_scope;
        let config = middleware_config(&req);
        let rate_limit_enabled = config
            .as_ref()
            .map(|config| config.rate_limit_enabled)
            .unwrap_or(true); // Default to enabled for safety

        // Trusted proxy list decides whether `X-Forwarded-For` / `Forwarded`
        // headers may override the peer address. Without it every request
        // behind a reverse proxy is throttled as if it came from the proxy.
        let trusted_proxies = config
            .as_ref()
            .map(|config| config.trusted_proxies.clone())
            .unwrap_or_default();

        // Try to extract user_id from JWT claims (set by JwtAuth middleware).
        // If authenticated, use user_id for rate limiting; otherwise fall back
        // to the real client IP.
        let is_authenticated = req.extensions().get::<Claims>().is_some();
        let client_ip = crate::utils::ip::extract_client_ip(req.request(), &trusted_proxies);
        let client_key = Self::rate_limit_client_key(client_ip, req.extensions().get::<Claims>());
        // Aggregated users-table protection: configurable ceiling for the
        // register endpoint alone (`REGISTER_DAILY_LIMIT`).
        let register_daily_limit = config
            .as_ref()
            .map(|config| config.register_daily_limit)
            .unwrap_or(DEFAULT_REGISTER_DAILY_LIMIT);

        Box::pin(async move {
            if !rate_limit_enabled {
                let res = svc.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            let category = rate_limit_category(req.method(), req.path());

            // Single-charge rule: nested scope limiters leave the category-
            // resolved buckets (AuthStrict, AuthSession) to the outer scope-wide
            // limiter, which charges the same key.
            if !charges(charge_scope, category) {
                let res = svc.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            let effective_limit = RateLimiterMiddleware::<S>::pick_effective_limit(
                &limit,
                req.method(),
                req.path(),
                is_authenticated,
            );

            // Circuit breaker check: if open, fail OPEN (allow request through)
            // instead of repeatedly hitting a dead Redis — EXCEPT for
            // AuthStrict endpoints (login/register/recovery), which always
            // fail CLOSED so a Redis outage never becomes a brute-force bypass.
            if circuit_breaker.is_open() {
                if effective_limit.is_strict() {
                    tracing::warn!(
                        event = "rate_limit.strict_fail_closed",
                        key_prefix = effective_limit.key_prefix,
                        "Circuit breaker open but AuthStrict endpoint — failing closed"
                    );
                    return Ok(service_unavailable_response(req));
                }
                tracing::debug!(
                    event = "rate_limit.circuit_breaker_bypass",
                    "Circuit breaker open — allowing request without rate limit check"
                );
                let res = svc.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            // Strict endpoints charge several dimensions (per IP, per subnet,
            // register daily aggregate); everything else keeps its single
            // category bucket.
            let charge_list = charges_for_request(
                &effective_limit,
                req.path(),
                &client_key,
                client_ip,
                register_daily_limit,
            );
            let first_charge = charge_list
                .first()
                .cloned()
                .expect("charges_for_request always returns at least one bucket");

            // Attempt rate limiting with Redis.
            // On failure, circuit breaker decides whether to fail closed or open.
            let quota = async {
                let mut conn = redis.get().await.ok()?;
                let mut first_outcome: Option<QuotaOutcome> = None;
                let mut throttled: Option<(QuotaOutcome, RateLimit)> = None;

                for charge in &charge_list {
                    // Fixed-window Lua script: 2 Redis ops per bucket, counter
                    // and window TTL arrive in one response.
                    let result: Option<(u64, i64)> = redis::cmd("EVAL")
                        .arg(RATE_LIMIT_LUA_SCRIPT)
                        .arg(1) // number of keys
                        .arg(&charge.key)
                        .arg(charge.limit.window_secs)
                        .query_async(&mut conn)
                        .await
                        .ok();
                    let result = result?;

                    let outcome = QuotaOutcome {
                        allowed: result.0 <= charge.limit.max_requests,
                        remaining: charge.limit.max_requests.saturating_sub(result.0),
                        // A negative TTL means the key lost its expiry, so the
                        // full window is the honest answer.
                        reset_secs: u64::try_from(result.1).unwrap_or(charge.limit.window_secs),
                    };

                    if first_outcome.is_none() {
                        // The per-IP budget stands for the quota headers; an
                        // aggregate bucket would only owe shared credit.
                        first_outcome = Some(outcome.clone());
                    }
                    if !outcome.allowed {
                        throttled = Some((outcome, charge.limit.clone()));
                        break;
                    }
                }

                Some((first_outcome, throttled))
            }
            .await;

            let quota = match quota {
                Some((first_outcome, throttled)) => (first_outcome, throttled),
                None => {
                    // Redis unavailable or error
                    circuit_breaker.record_failure();

                    // After recording the failure, check if circuit is now open
                    if circuit_breaker.is_open() && effective_limit.is_strict() {
                        // AuthStrict never fails open (see is_strict).
                        tracing::warn!(
                            event = "rate_limit.strict_fail_closed",
                            key = %first_charge.key,
                            "Redis failure on AuthStrict endpoint — failing closed despite open circuit"
                        );
                        return Ok(service_unavailable_response(req));
                    }
                    if circuit_breaker.is_open() {
                        // Circuit just opened — fail OPEN this request to avoid
                        // cascading failures, but log the degradation
                        tracing::warn!(
                            event = "rate_limit.circuit_breaker_opened_on_failure",
                            key = %first_charge.key,
                            "Redis failure triggered circuit breaker — allowing request through"
                        );
                        // Let the request proceed without rate limiting
                        let res = svc.call(req).await?;
                        return Ok(res.map_into_boxed_body());
                    }

                    // Circuit not yet open — fail closed with 503
                    tracing::warn!(
                        event = "rate_limit.redis_unavailable",
                        key = %first_charge.key,
                        "Rate limiter Redis unavailable, failing closed"
                    );
                    return Ok(service_unavailable_response(req));
                },
            };

            let (first_outcome, throttled): (
                Option<QuotaOutcome>,
                Option<(QuotaOutcome, RateLimit)>,
            ) = quota;

            if let Some((outcome, charge_limit)) = throttled {
                // Rate limited - also counts as success (Redis is healthy)
                circuit_breaker.record_success();
                tracing::debug!(
                    event = "rate_limit.throttled",
                    key = %first_charge.key,
                    remaining = outcome.remaining,
                    reset_secs = outcome.reset_secs,
                    "Request rejected by the rate limiter"
                );
                let mut response = HttpResponse::TooManyRequests();
                for (name, value) in quota_headers(&charge_limit, &outcome) {
                    response.insert_header((name, value));
                }
                let response = response
                    .insert_header(("Retry-After", charge_limit.window_secs.to_string()))
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

            // Success - close circuit if it was half-open
            circuit_breaker.record_success();

            let mut res = svc.call(req).await?;
            if let Some(outcome) = first_outcome {
                insert_quota_headers(res.headers_mut(), &first_charge.limit, &outcome);
            }
            Ok(res.map_into_boxed_body())
        })
    }
}

/// Quota outcome of a request that Redis could evaluate.
///
/// Carries the numbers the quota headers need, so an allowed request and a
/// throttled one answer with the same information.
#[derive(Debug, Clone)]
struct QuotaOutcome {
    /// Whether the request may proceed to the handler.
    allowed: bool,
    /// Requests left in the current window, zero once the bucket is full.
    remaining: u64,
    /// Seconds left before the current window resets.
    reset_secs: u64,
}

/// Quota header names, lowercase because `HeaderName::from_static` requires
/// it and HTTP treats the names case-insensitively on the wire.
const QUOTA_HEADER_LIMIT: &str = "x-ratelimit-limit";
const QUOTA_HEADER_REMAINING: &str = "x-ratelimit-remaining";
const QUOTA_HEADER_RESET: &str = "x-ratelimit-reset";

/// Quota header values for an outcome, ready to be inserted either on a
/// response builder or on an already built response.
fn quota_headers(limit: &RateLimit, outcome: &QuotaOutcome) -> [(&'static str, String); 3] {
    [
        (QUOTA_HEADER_LIMIT, limit.max_requests.to_string()),
        (QUOTA_HEADER_REMAINING, outcome.remaining.to_string()),
        (QUOTA_HEADER_RESET, outcome.reset_secs.to_string()),
    ]
}

/// Insert the quota headers into an already built response.
///
/// The allowed path cannot set them on a builder, since the response is
/// produced by the handler.
fn insert_quota_headers(headers: &mut HeaderMap, limit: &RateLimit, outcome: &QuotaOutcome) {
    for (name, value) in quota_headers(limit, outcome) {
        // Values are digits, so conversion cannot fail. Skip rather than
        // panic on a request path.
        if let Ok(value) = HeaderValue::from_str(&value) {
            headers.insert(HeaderName::from_static(name), value);
        }
    }
}

/// Build the 503 response used whenever the limiter must fail closed:
/// Redis is unavailable and the circuit has not opened yet, or the request
/// targets an AuthStrict endpoint (which never fails open).
fn service_unavailable_response(req: ServiceRequest) -> ServiceResponse<BoxBody> {
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
    ServiceResponse::new(http_req, response)
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use actix_web::http::Method;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn rate_limit_key_uses_authenticated_profile_id() {
        let claims = Claims {
            sub: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            role: 0,
            token_use: "access".to_string(),
            exp: 0,
            iat: 0,
            nbf: 0,
            jti: None,
        };

        assert_eq!(
            RateLimiterMiddleware::<()>::rate_limit_client_key(None, Some(&claims)),
            format!("user:{}", claims.profile_id),
            "an authenticated request must never share a bucket with anonymous traffic"
        );
    }

    #[test]
    fn rate_limit_key_falls_back_to_client_ip() {
        let ip: IpAddr = "203.0.113.9".parse().unwrap();
        assert_eq!(
            RateLimiterMiddleware::<()>::rate_limit_client_key(Some(ip), None),
            "203.0.113.9"
        );
        assert_eq!(
            RateLimiterMiddleware::<()>::rate_limit_client_key(None, None),
            "unknown"
        );
    }

    #[actix_web::test]
    async fn rate_limit_reads_client_ip_from_trusted_proxy_header() {
        // The limiter used to read `peer_addr()` directly, so every anonymous
        // request behind nginx shared the proxy's IP in one Redis bucket.
        let trusted = vec!["10.0.0.0/8".parse::<ipnet::IpNet>().unwrap()];
        let untrusted: Vec<ipnet::IpNet> = Vec::new();

        let behind_proxy = actix_web::test::TestRequest::get()
            .uri("/api/v1/users")
            .peer_addr("10.1.2.3:5000".parse().unwrap())
            .insert_header(("x-forwarded-for", "203.0.113.9, 10.1.2.3"))
            .to_request();
        let http_req = actix_web::test::TestRequest::get()
            .uri("/api/v1/users")
            .peer_addr("10.1.2.3:5000".parse().unwrap())
            .insert_header(("x-forwarded-for", "203.0.113.9, 10.1.2.3"))
            .to_http_request();

        assert_eq!(
            crate::utils::ip::extract_client_ip(&http_req, &trusted),
            Some("203.0.113.9".parse().unwrap()),
            "a trusted proxy must contribute the real client IP"
        );
        assert_eq!(
            crate::utils::ip::extract_client_ip(&http_req, &untrusted),
            Some("10.1.2.3".parse::<IpAddr>().unwrap()),
            "an untrusted peer's forwarded header must be ignored"
        );
        let _ = behind_proxy;
    }

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

    // =========================================================================
    // Multi-dimensional strict budgets
    // =========================================================================

    #[test]
    fn subnet_bucket_key_masks_the_last_ipv4_octet() {
        assert_eq!(
            subnet_bucket_key(&"198.51.100.44".parse().unwrap()),
            "198.51.100.mask",
        );
    }

    #[test]
    fn ipv4_neighbors_share_one_subnet_bucket() {
        assert_eq!(
            subnet_bucket_key(&"198.51.100.44".parse().unwrap()),
            subnet_bucket_key(&"198.51.100.99".parse().unwrap()),
            "rotating the last octet must not open a new subnet bucket",
        );
    }

    #[test]
    fn ipv4_depositors_on_different_networks_get_different_buckets() {
        assert_ne!(
            subnet_bucket_key(&"198.51.100.44".parse().unwrap()),
            subnet_bucket_key(&"198.52.100.44".parse().unwrap()),
        );
    }

    #[test]
    fn subnet_bucket_key_keeps_the_first_three_ipv6_hextets() {
        assert_eq!(
            subnet_bucket_key(&"2001:db8:1234:5678::1".parse().unwrap()),
            "2001:db8:1234.mask",
        );
    }

    #[test]
    fn ipv6_rotation_within_the_allocation_shares_one_bucket() {
        assert_eq!(
            subnet_bucket_key(&"2001:db8:1234:5678::1".parse().unwrap()),
            subnet_bucket_key(&"2001:db8:1234:9abc::2".parse().unwrap()),
            "rotating within a /48 must not open a new subnet bucket",
        );
    }

    #[test]
    fn register_charges_the_exact_subnet_and_global_daily_buckets() {
        let buckets = strict_charge_buckets(
            &RATE_AUTH_STRICT,
            Some("198.51.100.44".parse().unwrap()),
            "/api/v1/auth/register",
            2000,
        );

        assert_eq!(buckets.len(), 3, "register must charge three dimensions");
        assert_eq!(buckets[0].key, "rl:auth_strict:198.51.100.44");
        assert_eq!(buckets[0].limit.max_requests, RATE_AUTH_STRICT.max_requests);
        assert_eq!(
            buckets[1].key, "rl:auth_strict_mask:198.51.100.mask",
            "the subnet bucket must blunt last-octet rotation",
        );
        assert_eq!(
            buckets[1].limit.max_requests,
            RATE_AUTH_STRICT_SUBNET.max_requests
        );
        assert_eq!(buckets[2].key, REGISTER_GLOBAL_KEY);
        assert_eq!(buckets[2].limit.max_requests, 2000);
        assert_eq!(buckets[2].limit.window_secs, SECONDS_PER_DAY);
    }

    #[test]
    fn login_charges_no_register_daily_bucket() {
        let buckets = strict_charge_buckets(
            &RATE_AUTH_STRICT,
            Some("198.51.100.44".parse().unwrap()),
            "/api/v1/auth/login",
            2000,
        );
        assert_eq!(buckets.len(), 2, "login must charge exact + subnet only");
        assert_eq!(buckets[0].key, "rl:auth_strict:198.51.100.44");
        assert_eq!(buckets[1].key, "rl:auth_strict_mask:198.51.100.mask");
    }

    #[test]
    fn register_still_charges_daily_cap_when_the_ip_is_unknown() {
        let buckets = strict_charge_buckets(&RATE_AUTH_STRICT, None, "/api/v1/auth/register", 2000);
        assert_eq!(buckets.len(), 2, "no subnet dimension without an address");
        assert_eq!(buckets[0].key, "rl:auth_strict:unknown");
        assert_eq!(buckets[1].key, REGISTER_GLOBAL_KEY);
    }

    #[test]
    fn strict_subnet_budget_is_looser_than_the_per_ip_budget() {
        const {
            assert!(RATE_AUTH_STRICT.max_requests < RATE_AUTH_STRICT_SUBNET.max_requests);
        }
    }

    // =========================================================================
    // Charge scope (single-charge rule)
    // =========================================================================

    #[test]
    fn nested_scopes_charge_only_the_default_category() {
        // Category-resolved strict/session buckets collide between the outer
        // (scope-wide) limiter and the nested auth-scope limiter, so the nested
        // one must charge just the Default category.
        assert!(!charges(
            ChargeScope::DefaultOnly,
            RateLimitCategory::AuthStrict
        ));
        assert!(!charges(
            ChargeScope::DefaultOnly,
            RateLimitCategory::AuthSession
        ));
        assert!(charges(
            ChargeScope::DefaultOnly,
            RateLimitCategory::Default
        ));
    }

    #[test]
    fn scope_wide_limiters_charge_every_category() {
        assert!(charges(ChargeScope::All, RateLimitCategory::AuthStrict));
        assert!(charges(ChargeScope::All, RateLimitCategory::AuthSession));
        assert!(charges(ChargeScope::All, RateLimitCategory::Default));
    }

    // =========================================================================
    // Circuit Breaker Tests
    // =========================================================================

    #[test]
    fn circuit_breaker_starts_closed() {
        let cb = RedisCircuitBreaker::new(3, Duration::from_secs(30));
        assert!(!cb.is_open());
    }

    #[test]
    fn circuit_breaker_opens_after_threshold_failures() {
        let cb = RedisCircuitBreaker::new(3, Duration::from_secs(30));
        cb.record_failure();
        assert!(!cb.is_open()); // 1 failure, not yet open
        cb.record_failure();
        assert!(!cb.is_open()); // 2 failures, not yet open
        cb.record_failure();
        assert!(cb.is_open()); // 3 failures = threshold, now open
    }

    #[test]
    fn circuit_breaker_resets_on_success() {
        let cb = RedisCircuitBreaker::new(2, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_open()); // open after 2 failures
        cb.record_success();
        assert!(!cb.is_open()); // closed after success
    }

    #[test]
    fn circuit_breaker_success_resets_failure_counter() {
        let cb = RedisCircuitBreaker::new(3, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        cb.record_success(); // resets counter to 0
        cb.record_failure();
        assert!(!cb.is_open()); // only 1 failure after reset
        cb.record_failure();
        assert!(!cb.is_open()); // 2 failures after reset
        cb.record_failure();
        assert!(cb.is_open()); // 3 failures after reset, now open
    }

    #[test]
    fn circuit_breaker_closes_after_timeout() {
        let cb = RedisCircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_open());

        // Wait for reset_timeout to elapse
        std::thread::sleep(Duration::from_millis(60));
        assert!(!cb.is_open()); // now in half-open state, allows retry
    }

    // =========================================================================
    // AuthStrict fail-closed Tests
    // =========================================================================

    #[test]
    fn strict_endpoints_are_detected_for_both_buckets() {
        assert!(RATE_AUTH_STRICT.is_strict());
        assert!(RATE_AUTH_STRICT_AUTHENTICATED.is_strict());
    }

    #[test]
    fn non_strict_endpoints_are_not_detected() {
        assert!(!RATE_AUTH.is_strict());
        assert!(!RATE_AUTH_AUTHENTICATED.is_strict());
        assert!(!RATE_API.is_strict());
        assert!(!RATE_API_AUTHENTICATED.is_strict());
        assert!(!RATE_AUTH_SESSION.is_strict());
        assert!(!RATE_AUTH_SESSION_AUTHENTICATED.is_strict());
        assert!(!RATE_UPLOAD.is_strict());
        assert!(!RATE_MESSAGES.is_strict());
        assert!(!RATE_MESSAGES_AUTHENTICATED.is_strict());
    }

    #[test]
    fn resolved_strict_category_limits_are_strict() {
        // The resolved limit for brute-force-sensitive routes must carry the
        // strict flag, otherwise a Redis outage would bypass their throttling.
        for path in [
            "/api/v1/auth/login",
            "/api/v1/auth/register",
            "/api/v1/auth/recover",
            "/api/v1/auth/reset",
        ] {
            for authenticated in [false, true] {
                let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
                    &RATE_API,
                    &Method::POST,
                    path,
                    authenticated,
                );
                assert!(limit.is_strict(), "expected strict limit for {path}");
            }
        }
    }

    #[test]
    fn resolved_non_strict_limits_are_not_strict() {
        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::POST,
            "/api/v1/auth/refresh",
            false,
        );
        assert!(!limit.is_strict());

        let limit = RateLimiterMiddleware::<()>::pick_effective_limit(
            &RATE_API,
            &Method::GET,
            "/api/v1/users",
            false,
        );
        assert!(!limit.is_strict());
    }
}
