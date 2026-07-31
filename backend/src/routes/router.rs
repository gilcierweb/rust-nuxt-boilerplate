use actix_web::web;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::api_docs::ApiDoc;
pub use crate::controllers::{
    audit_logs_controller, auth_controller, health_controller, metrics_controller,
    roles_controller, upload_controller, users_controller,
};
use crate::middleware::csrf_protection::CsrfProtection;
use crate::middleware::stripe_webhook_verifier::StripeWebhookVerifier;

/// Configure API routes with middleware chain.
///
/// # Middleware Order (Critical)
///
/// The middleware stack order is critical and follows this sequence:
///
/// 1. **GrantsMiddleware (RBAC)** - Extracts user authorities for role-based access control
///    - Runs first to populate request extensions with user roles
///    - Used by `#[authorize("role:admin")]` guards on handlers
///
/// 2. **RequireApiKey (API Key Auth)** - Validates API key for service-to-service communication
///    - Runs after RBAC to allow API key authentication
///    - Exempt paths (webhooks, auth endpoints) skip this middleware
///    - Note: Webhook endpoints need API key AND are in public routes list
///
/// 3. **RateLimiter** - Enforces rate limits per IP/client
///    - Runs after auth to avoid rate limiting auth failures
///    - Uses Redis-backed sliding window algorithm
///    - Different limits for auth vs API endpoints
///
/// 4. **CsrfProtection (CSRF)** - Validates CSRF tokens for state-changing operations
///    - Applied only to `/auth` and `/admin` scopes
///    - Skips Bearer token requests (allows API-first apps)
///    - Uses signed cookies for CSRF token validation
///
/// 5. **JwtAuth (JWT Validation)** - Validates JWT tokens for protected routes
///    - Applied only to `/admin` scope (global application deferred)
///    - Handler-level `AuthUser` extractor used for other protected routes
///    - Public routes exempted via `bearer_exempt_routes()`
///
/// # Design Decisions
///
/// - **JwtAuth on admin scope only**: Reduces overhead for public/auth routes
///   while ensuring admin routes are always protected. Handler-level extractors
///   provide flexibility for routes that need different auth strategies.
///
/// - **CSRF skips Bearer tokens**: Allows API-first applications to use the API
///   without CSRF tokens while still protecting browser-based form submissions.
///
/// - **Rate limiting after auth**: Prevents brute force attacks from consuming
///   rate limit quota before authentication is attempted.
///
/// - **Webhook routes public but verified**: Webhooks are in the public routes
///   list (skip API key) but have their own signature verification middleware
///   (`StripeWebhookVerifier`).
///
/// # Swagger UI access policy
///
/// The Swagger UI (`/swagger-ui/`) and the OpenAPI JSON (`/api-docs/openapi.json`)
/// are always mounted at the application root, outside the authenticated
/// `/api/v1` scope, so they are reachable without the JWT/API-key middleware
/// stack that guards business endpoints.
///
/// - **Development & Test**: The UI and JSON are public. Developers can open
///   `http://localhost:8080/swagger-ui/` directly in a browser.
/// - **Staging & Production**: The UI and JSON are protected by the
///   `RequireApiKey` middleware. Browsers must send a valid `X-API-Key` header
///   (use a browser extension such as ModHeader, or open the JSON via an
///   authenticated `curl` and import it into Swagger Editor/Insomnia/Postman).
///   This prevents leaking internal route documentation, request/response
///   schemas, and auth scheme details to unauthenticated callers in
///   internet-facing environments.
pub fn config(cfg: &mut web::ServiceConfig, redis_pool: deadpool_redis::Pool) {
    // Resolve environment-aware auth policy for the Swagger UI / OpenAPI docs.
    //
    // `actix_web::web::ServiceConfig` exposes no public accessor for the
    // already-registered `AppContainer`, so we re-read the `ENVIRONMENT`
    // variable here — the same source `AppConfig::from_env` uses — to decide
    // whether the docs mount requires an API key. This keeps the public
    // `config` signature unchanged.
    let require_api_key_for_docs = matches!(
        std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_ascii_lowercase()
            .as_str(),
        "staging" | "production"
    );

    let openapi = ApiDoc::openapi();

    // Public docs mount (development & test):
    // Serves the Swagger UI assets and the OpenAPI JSON without any auth.
    if !require_api_key_for_docs {
        cfg.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
        )
        .service(
            Scalar::with_url("/scalar", openapi.clone()),
        );
    }

    cfg.service(
        web::scope("/api/v1")
            // 0. API version guard (first) — version negotiation + deprecation headers
            .wrap(crate::middleware::api_version::ApiVersionGuard::new(
                crate::middleware::api_version::ApiVersionConfig::new(),
            ))
            // Middleware order (outermost first, executes first on request):
            // 1. Rate limiting (global) - first line of defense
            .wrap(crate::middleware::rate_limit_middleware::RateLimiter::new(
                redis_pool.clone(),
                crate::middleware::rate_limit_middleware::RATE_API,
            ))
            // 2. API Key auth - for service-to-service
            // NOTE: /metrics is intentionally NOT in this exempt list.
            // The Prometheus metrics endpoint must require a valid API key
            // (X-API-Key or Authorization: ApiKey <key>) to prevent exposing
            // internal counters, latencies, route names, and system resource
            // information to unauthenticated callers.
            .wrap(crate::middleware::api_key_middleware::RequireApiKey::new(vec![
                "/api/v1/webhooks/*",
                "/api/v1/ws",
                "/api/v1/auth/login",
                "/api/v1/auth/login/",
                "/api/v1/auth/register",
                "/api/v1/auth/register/",
                "/api/v1/auth/session",
                "/api/v1/auth/session/",
                "/api/v1/auth/refresh",
                "/api/v1/auth/refresh/",
                "/api/v1/auth/logout",
                "/api/v1/auth/logout/",
                "/api/v1/auth/recover",
                "/api/v1/auth/recover/",
                "/api/v1/auth/reset",
                "/api/v1/auth/reset/",
                "/api/v1/auth/confirm",
                "/api/v1/auth/confirm/",
                "/api/v1/health",
                "/api/v1/health/",
            ]))
            // Auth routes - no RBAC needed
            .service(
                web::scope("/auth")
                    .wrap(CsrfProtection::new(vec![]))
                    .wrap(crate::middleware::rate_limit_middleware::RateLimiter::new(
                        redis_pool.clone(),
                        crate::middleware::rate_limit_middleware::RATE_AUTH,
                    ))
                    .service(auth_controller::login)
                    .service(auth_controller::register)
                    .service(auth_controller::recover_password)
                    .service(auth_controller::forgot_password)
                    .service(auth_controller::reset_password)
                    .service(auth_controller::confirm)
                    .service(auth_controller::me)
                    .service(auth_controller::session)
                    .service(auth_controller::session_trailing)
                    .service(auth_controller::refresh)
                    .service(auth_controller::logout)
                    .service(auth_controller::setup_2fa)
                    .service(auth_controller::enable_2fa)
                    .service(auth_controller::disable_2fa)
                    .service(auth_controller::change_password),
            )
            // Webhook routes
            .service(
                web::scope("/webhooks")
                    .wrap(StripeWebhookVerifier::new())
                    .route("/stripe", web::post().to(auth_controller::stripe_webhook))
                    .route("/pix", web::post().to(auth_controller::pix_webhook)),
            )
            // Admin domain routes
            // Middleware order (outermost first on request):
            // 1. CsrfProtection (outermost) - checks CSRF for browser forms
            // 2. JwtAuth - validates JWT, inserts Claims and AuthDetails in extensions
            // 3. Admin handlers use AuthDetails from extensions for RBAC
            .service(
                web::scope("/admin")
                    .wrap(CsrfProtection::new(vec![]))
                    .wrap(crate::middleware::auth::JwtAuth::new(
                        crate::middleware::auth::JwtAuthConfig::new(vec![]),
                    ))
                    .configure(roles_controller::config)
                    .configure(users_controller::config)
                    .configure(audit_logs_controller::config)
                    .service(upload_controller::upload_file),
            )
            .route("/health", web::get().to(health_controller::health_check))
            // NOTE: /metrics requires API key — protected by RequireApiKey middleware on this scope.
            // Access via: X-API-Key: <key> or Authorization: ApiKey <key>
            .route("/metrics", web::get().to(metrics_controller::metrics))
            // WebSocket route (inside /api/v1 scope)
            .service(web::resource("/ws").route(web::get().to(crate::ws::redis_handler::ws_handler))),
    );

    // Authenticated docs mount (staging & production):
    // The same Swagger UI + OpenAPI JSON, but inside a scope guarded by the
    // `RequireApiKey` middleware. Browsers/clients must send a valid
    // `X-API-Key` header to load the UI or the spec. In dev/test the public
    // mount above is registered instead and this one is skipped.
    if require_api_key_for_docs {
        cfg.service(
            web::scope("")
                .wrap(crate::middleware::api_key_middleware::RequireApiKey::new(vec![
                    // Everything else under this scope requires the API key,
                    // so we only exempt the health route that the API exposes
                    // for liveness probes (already protected elsewhere).
                    "/health",
                    "/health/",
                    "/api/v1/health",
                    "/api/v1/health/",
                ]))
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", openapi.clone()),
                )
                .service(
                    Scalar::with_url("/scalar", openapi.clone()),
                ),
        );
    }
}
