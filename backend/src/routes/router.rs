use crate::api_docs::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use crate::controllers::{
    audit_logs_controller, auth_controller, health_controller, roles_controller, users_controller, metrics_controller
};

use actix_web::http::Method;
use actix_web::web;

use crate::middleware::csrf_protection::CsrfProtection;

fn bearer_exempt_routes() -> Vec<crate::middleware::api_access_middleware::PublicRoute> {
    use crate::middleware::api_access_middleware::PublicRoute;

    vec![
        PublicRoute::method(Method::POST, "/api/v1/auth/login"),
        PublicRoute::method(Method::POST, "/api/v1/auth/login/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/register"),
        PublicRoute::method(Method::POST, "/api/v1/auth/register/"),
        PublicRoute::method(Method::GET, "/api/v1/auth/confirm"),
        PublicRoute::method(Method::GET, "/api/v1/auth/confirm/"),
        PublicRoute::method(Method::GET, "/api/v1/auth/session"),
        PublicRoute::method(Method::GET, "/api/v1/auth/session/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/refresh"),
        PublicRoute::method(Method::POST, "/api/v1/auth/refresh/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/logout"),
        PublicRoute::method(Method::POST, "/api/v1/auth/logout/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/recover"),
        PublicRoute::method(Method::POST, "/api/v1/auth/recover/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/reset"),
        PublicRoute::method(Method::POST, "/api/v1/auth/reset/"),
        PublicRoute::method(Method::GET, "/api/v1/health"),
        PublicRoute::method(Method::GET, "/api/v1/health/"),
        PublicRoute::method(Method::GET, "/api/v1/metrics"),
        PublicRoute::method(Method::GET, "/api/v1/metrics/"),
        PublicRoute::method(Method::POST, "/api/v1/webhooks/stripe"),
        PublicRoute::method(Method::POST, "/api/v1/webhooks/pix"),
        PublicRoute::method(Method::GET, "/api/v1/ws"),
    ]
}

pub fn config(cfg: &mut web::ServiceConfig, redis_pool: deadpool_redis::Pool) {
    let openapi = ApiDoc::openapi();

    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
    )
    .service(
        web::scope("/api/v1")
            .wrap(actix_web_grants::GrantsMiddleware::with_extractor(
                crate::authz::grants_extractor::extract_authorities,
            ))
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
                "/api/v1/metrics",
                "/api/v1/metrics/",
            ]))
            .wrap(crate::middleware::api_access_middleware::ApiAccessGate::new(
                bearer_exempt_routes(),
            ))
            .wrap(crate::middleware::rate_limit_middleware::RateLimiter::new(
                redis_pool.clone(),
                crate::middleware::rate_limit_middleware::RATE_API,
            ))
            // Auth routes
            .service(
                web::scope("/auth")
                    .wrap(CsrfProtection::new(vec![
                        "/api/v1/auth/login".to_string(),
                        "/api/v1/auth/register".to_string(),
                        "/api/v1/auth/recover".to_string(),
                        "/api/v1/auth/reset".to_string(),
                        "/api/v1/auth/confirm".to_string(),
                        "/api/v1/auth/logout".to_string(),
                        "/api/v1/auth/refresh".to_string(),
                    ]))
                    .wrap(crate::middleware::rate_limit_middleware::RateLimiter::new(
                        redis_pool.clone(),
                        crate::middleware::rate_limit_middleware::RATE_AUTH,
                    ))
                    .service(auth_controller::login)
                    .service(auth_controller::register)
                    .service(auth_controller::recover_password)
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
            // Admin domain routes
            .service(
                web::scope("/admin")
                    .configure(roles_controller::config)
                    .configure(users_controller::config)
                    .configure(audit_logs_controller::config),
            )
            // Health check
            .route("/health", web::get().to(health_controller::health_check))
            .route("/metrics", web::get().to(metrics_controller::metrics))
            // WebSocket route (inside /api/v1 scope)
            .service(web::resource("/ws").route(web::get().to(crate::ws::server::ws_handler))),
    );
}
