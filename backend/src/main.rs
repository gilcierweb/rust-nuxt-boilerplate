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

/// Initialize OpenTelemetry tracer provider.
///
/// Reads `OTEL_EXPORTER_OTLP_ENDPOINT` env var (default: `http://localhost:4317`).
/// Returns None if OTEL is disabled or initialization fails.
fn init_opentelemetry() -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry_otlp::WithExportConfig;

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    // Disable OTEL by setting OTEL_ENABLED=false
    if std::env::var("OTEL_ENABLED")
        .map(|v| v == "false" || v == "0")
        .unwrap_or(false)
    {
        tracing::info!("OpenTelemetry disabled via OTEL_ENABLED=false");
        return None;
    }

    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            opentelemetry::KeyValue::new("service.name", "backend-api"),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    // Build span exporter
    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()
    {
        Ok(exporter) => exporter,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to create OTLP span exporter");
            return None;
        }
    };

    // Create tracer provider
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    tracing::info!(endpoint = %endpoint, "OpenTelemetry tracing initialized");
    Some(provider)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let boot_start = std::time::Instant::now();

    rust_i18n::set_locale("pt-BR");

    // Initialize OpenTelemetry (optional)
    let otel_provider = init_opentelemetry();

    // Build tracing subscriber with optional OpenTelemetry layer
    let registry = tracing_subscriber::registry()
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
        );

    // Add OpenTelemetry layer if provider is available
    if let Some(ref provider) = otel_provider {
        use opentelemetry::trace::TracerProvider;
        let tracer = provider.tracer("backend-api");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry.with(otel_layer).init();
    } else {
        registry.init();
    }

    // Load .env from project root (parent of backend directory)
    let env_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR parent")
        .join(".env");
    dotenvy::from_path(env_path).ok();
    let config = AppConfig::from_env().expect("Failed to load configuration");
    config.validate_or_panic();
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

    // Record cold-start duration (time from boot_start to AppState ready)
    state.metrics.record_cold_start(boot_start.elapsed());

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db_pool_for_container,
        redis_pool_for_container,
        (*config).clone(),
    ));

    // Parse CORS origins from config (validation done in AppConfig::validate())
    let cors_origins: Vec<String> = config
        .frontend_url
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if cors_origins.is_empty() {
        tracing::warn!(
            event = "security.cors_no_valid_origins",
            "No valid CORS origins configured. CORS will deny all cross-origin requests."
        );
    } else {
        tracing::info!(
            event = "security.cors_configured",
            origins = ?cors_origins,
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

        for origin in &cors_origins {
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

    let result = match config.environment {
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
    };

    // Shutdown OpenTelemetry provider to flush pending traces
    if let Some(provider) = otel_provider {
        if let Err(e) = provider.shutdown() {
            tracing::warn!(error = %e, "Failed to shutdown OpenTelemetry provider");
        }
    }

    result
}
