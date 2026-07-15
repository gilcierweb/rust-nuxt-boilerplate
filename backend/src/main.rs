#[macro_use]
extern crate rust_i18n;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, web};
use deadpool_redis::{Config as RedisConfig, Runtime};
use serde::Serialize;
use std::borrow::Cow;
use std::io::BufReader;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::config::AppConfig;
use backend::db::database::Database;
use backend::errors::AppError;
use backend::AppState;

i18n!("locales");

#[derive(Serialize)]
pub struct Response<'a> {
    pub message: Cow<'a, str>,
}

async fn not_found() -> Result<HttpResponse, actix_web::Error> {
    let response = Response {
        message: t!("errors.not_found", resource = "Resource"),
    };
    Ok(HttpResponse::NotFound().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rust_i18n::set_locale("pt-BR");

tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend_api=debug,actix_web=info,http.request=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .pretty()
        )
        .init();

    dotenvy::dotenv().ok();
    let config = AppConfig::from_env().expect("Failed to load configuration");
    let config = Arc::new(config);
    tracing::info!(
        "Starting Backend API v{} on {}:{}",
        env!("CARGO_PKG_VERSION"),
        config.host,
        config.port
    );

    let api_db = Database::from_config(&config);
    let db_pool = api_db.pool.clone();
    let db_pool_for_container = db_pool.clone();

    let mut redis_cfg = RedisConfig::from_url(&config.redis_url);
    redis_cfg.pool = Some(deadpool_redis::PoolConfig::new(config.redis_pool_size));

    // Log Redis pool configuration for debugging
    tracing::info!(
        event = "redis.pool_config",
        pool_size = config.redis_pool_size,
        "Redis connection pool configured"
    );

    // Warn if pool size is too low for production workloads
    if config.redis_pool_size < 20 && matches!(config.environment, backend::config::app_config::Environment::Production) {
        tracing::warn!(
            event = "redis.pool_size_low",
            pool_size = config.redis_pool_size,
            recommended = 50,
            "Redis pool size may be insufficient for production. \
             Consider increasing REDIS_POOL_SIZE to 50+ for high-concurrency workloads \
             (rate limiting, caching, session storage, token blacklisting)."
        );
    }

    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis connection pool");
    let redis_pool_for_container = redis_pool.clone();

    let ws_state = web::Data::new(backend::ws::server::WsState::new());

    let state = web::Data::new(AppState {
        db: db_pool,
        redis: redis_pool,
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsState::new(),
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db_pool_for_container,
        redis_pool_for_container,
        (*config).clone(),
    ));

    let cors_origins = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001,http://localhost:4000,http://127.0.0.1:3000,http://127.0.0.1:3001,http://127.0.0.1:4000".to_string());

    // Validate CORS origins: reject wildcards when credentials are supported
    let cors_origins_list: Vec<String> = cors_origins
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Security check: reject wildcard origins when supports_credentials() is true
    for origin in &cors_origins_list {
        if origin == "*" {
            tracing::error!(
                event = "security.cors_wildcard_rejected",
                origin = %origin,
                "CORS wildcard '*' rejected: cannot use supports_credentials() with wildcard origin. \
                 Set FRONTEND_URL to specific origins (e.g., https://yourdomain.com)"
            );
            panic!(
                "CORS configuration error: wildcard '*' origin is not allowed with supports_credentials(). \
                 Set FRONTEND_URL environment variable to specific origins."
            );
        }
        // Validate origin format: must start with http:// or https://
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            tracing::warn!(
                event = "security.cors_invalid_origin",
                origin = %origin,
                "CORS origin rejected: must start with http:// or https://"
            );
            continue;
        }
    }

    // Filter to only valid origins
    let valid_origins: Vec<String> = cors_origins_list
        .into_iter()
        .filter(|o| o.starts_with("http://") || o.starts_with("https://"))
        .collect();

    if valid_origins.is_empty() {
        tracing::warn!(
            event = "security.cors_no_valid_origins",
            "No valid CORS origins configured. CORS will deny all cross-origin requests."
        );
    } else {
        tracing::info!(
            event = "security.cors_configured",
            origins = ?valid_origins,
            "CORS origins configured"
        );
    }

    let host = config.host.clone();
    let port = config.port;

    let config_json_limit = config.json_payload_limit;
    let config_form_limit = config.form_payload_limit;

    let app = move || {
        let pool_for_router = state.redis.clone();

        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::ACCESS_CONTROL_REQUEST_HEADERS,
            ])
            .allowed_header("x-api-key")
            .supports_credentials()
            .max_age(3600);

        for origin in &valid_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(state.clone())
            .app_data(container.clone())
            .app_data(ws_state.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(config_json_limit)
                    .error_handler(|_error, _request| {
                        AppError::BadRequest(t!("errors.bad_request_payload").into_owned()).into()
                    }),
            )
            .app_data(web::PayloadConfig::new(config_form_limit))
            .wrap(cors)
            .wrap(actix_web::middleware::Compress::default())
            .wrap(backend::middleware::security_headers::SecurityHeaders)
            .wrap(backend::middleware::metrics_middleware::MetricsMiddleware)
            .wrap(backend::middleware::request_log_middleware::RequestLogMiddleware)
            .route(
                "/metrics",
                web::get().to(backend::controllers::metrics_controller::metrics),
            )
            .route(
                "/health",
                web::get().to(backend::controllers::health_controller::health_check),
            )
            .configure(|cfg| backend::routes::router::config(cfg, pool_for_router.clone()))
            .default_service(web::route().to(not_found))
    };

    let server = HttpServer::new(app);

    match config.environment {
        backend::config::app_config::Environment::Staging | backend::config::app_config::Environment::Production => {
            // Initialize TLS crypto provider - falls back to default if already initialized
            let _ = rustls::crypto::CryptoProvider::get_default();

            let cert_path = config.tls_cert_path.clone();
            let key_path = config.tls_key_path.clone();

            let mut certs_file =
                BufReader::new(std::fs::File::open(&cert_path).map_err(|error| {
                    std::io::Error::other(format!(
                        "failed to open TLS certificate file '{}': {}",
                        cert_path, error
                    ))
                })?);
            let mut key_file = BufReader::new(std::fs::File::open(&key_path).map_err(|error| {
                std::io::Error::other(format!(
                    "failed to open TLS private key file '{}': {}",
                    key_path, error
                ))
            })?);

            let tls_certs = rustls_pemfile::certs(&mut certs_file)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| {
                    std::io::Error::other(format!(
                        "failed to parse TLS certificates from '{}': {}",
                        cert_path, error
                    ))
                })?;

            let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
                .next()
                .transpose()
                .map_err(|error| {
                    std::io::Error::other(format!(
                        "failed to parse TLS private key from '{}': {}",
                        key_path, error
                    ))
                })?
                .ok_or_else(|| {
                    std::io::Error::other(format!("no PKCS#8 private key found in '{}'", key_path))
                })?;

            let tls_config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
                .map_err(std::io::Error::other)?;

            let https_port = config.https_port;
            println!("Running in HTTPS on port {}", https_port);

            server
                .bind_rustls_0_23((host.clone(), https_port), tls_config)?
                .run()
                .await
        }
        backend::config::app_config::Environment::Development | backend::config::app_config::Environment::Test => {
            println!("Running in HTTP on http://localhost:{}", port);
            server.bind((host, port))?.run().await
        }
    }
}
