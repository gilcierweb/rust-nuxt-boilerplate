#[macro_use]
extern crate rust_i18n;

use std::borrow::Cow;
use std::io::BufReader;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer,
    middleware::{NormalizePath, TrailingSlash},
    web,
};
use backend::AppState;
use backend::config::AppConfig;
use backend::db::database::Database;
use backend::errors::AppError;
use deadpool_redis::{Config as RedisConfig, Runtime};
use serde::Serialize;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

i18n!("locales");

#[derive(Serialize)]
pub struct Response<'a> {
    pub message: Cow<'a, str>,
}

async fn not_found(req: actix_web::HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    // Fall back to the global rust_i18n (set_locale at startup) when the
    // request extensions don't carry a RequestLocale (e.g., very early
    // middleware failures). In normal operation the locale middleware has
    // already populated this.
    let message = backend::middleware::locale::locale_from_request(&req)
        .map(|rl| rl.t_blocking("errors.not_found", None))
        .unwrap_or_else(|| String::from("Resource not found"));
    let response = Response {
        message: Cow::from(message),
    };
    Ok(HttpResponse::NotFound().json(response))
}

/// Root-level liveness probe mounted at `/health` (outside `/api/v1`).
///
/// Returns a static 200 without touching DB/Redis — used by load balancers
/// and orchestrators that only need to know the process is up. The deep
/// dependency probe lives at `/api/v1/health` (see `health_controller`).
async fn root_health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

/// Initialize OpenTelemetry tracer provider with configurable sampling.
///
/// Uses `TelemetryConfig` to read `OTEL_SAMPLER`, `OTEL_SAMPLER_RATIO`,
/// `OTEL_EXPORTER_OTLP_ENDPOINT`, and `OTEL_ENABLED` env vars.
/// Returns None if OTEL is disabled or initialization fails.
fn init_opentelemetry() -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry_otlp::WithExportConfig;

    let telemetry = backend::config::telemetry::TelemetryConfig::from_env();

    if !telemetry.enabled {
        tracing::info!("OpenTelemetry disabled via OTEL_ENABLED=false");
        return None;
    }

    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes(vec![
            opentelemetry::KeyValue::new("service.name", "backend-api"),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&telemetry.endpoint)
        .build()
    {
        Ok(exporter) => exporter,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to create OTLP span exporter");
            return None;
        },
    };

    let sampler = telemetry.build_sampler();

    // Warn if sampling 100% of traces in production — will overwhelm OTLP endpoint
    if telemetry.is_full_coverage() {
        let env_var = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        if env_var.eq_ignore_ascii_case("production") {
            tracing::error!(
                event = "otel.full_coverage_production",
                sampler = ?telemetry.sampler,
                "OpenTelemetry is sampling 100% of traces in PRODUCTION. \
                 This will overwhelm the OTLP endpoint and incur high egress costs. \
                 Set OTEL_SAMPLER=ratio_based and OTEL_SAMPLER_RATIO=0.1 (or similar) \
                 to reduce trace volume."
            );
        } else if env_var.eq_ignore_ascii_case("staging") {
            tracing::warn!(
                event = "otel.full_coverage_staging",
                sampler = ?telemetry.sampler,
                "OpenTelemetry is sampling 100% of traces in staging. \
                 Consider OTEL_SAMPLER=ratio_based OTEL_SAMPLER_RATIO=0.5 \
                 to better simulate production conditions."
            );
        }
    }

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .with_sampler(sampler)
        .build();

    telemetry.log_config();
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
                .unwrap_or_else(|_| "backend=info,actix_web=info,http.request=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .pretty(),
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
    if config.redis_pool_size < backend::config::app_config::REDIS_POOL_MIN_PRODUCTION
        && matches!(
            config.environment,
            backend::config::app_config::Environment::Production
        )
    {
        tracing::error!(
            event = "redis.pool_size_below_minimum",
            pool_size = config.redis_pool_size,
            recommended = backend::config::app_config::REDIS_POOL_MIN_PRODUCTION,
            "Redis pool size is below the production minimum. \
             Consider increasing REDIS_POOL_SIZE to {}+ for high-concurrency workloads \
             (rate limiting, caching, session storage, token blacklisting).",
            backend::config::app_config::REDIS_POOL_MIN_PRODUCTION
        );
    } else if config.redis_pool_size
        < backend::config::app_config::REDIS_POOL_RECOMMENDED_PRODUCTION
        && matches!(
            config.environment,
            backend::config::app_config::Environment::Production
        )
    {
        tracing::warn!(
            event = "redis.pool_size_below_recommended",
            pool_size = config.redis_pool_size,
            recommended = backend::config::app_config::REDIS_POOL_RECOMMENDED_PRODUCTION,
            "Redis pool size is below the recommended production value of {}. \
             Current size ({}) may be insufficient for high-concurrency workloads \
             (rate limiting, caching, session storage, token blacklisting, WebSocket Pub/Sub). \
             Consider setting REDIS_POOL_SIZE={} or higher.",
            backend::config::app_config::REDIS_POOL_RECOMMENDED_PRODUCTION,
            config.redis_pool_size,
            backend::config::app_config::REDIS_POOL_RECOMMENDED_PRODUCTION
        );
    }

    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis connection pool");
    let redis_pool_for_container = redis_pool.clone();
    let redis_pool_for_ws = redis_pool.clone();

    let ws_state = web::Data::new(backend::ws::WsRedisState::new(
        redis_pool_for_ws,
        backend::ws::WsLimits::default(),
    ));

    // Load translation catalogues from the backend `locales/` directory. The
    // resulting `Translations` handle is shared with every request via the
    // locale middleware (see `middleware::locale::LocaleMiddleware`). This is
    // the per-request translation mechanism — it does not mutate any global
    // state, so concurrent requests cannot leak locale to each other.
    let translations =
        match backend::services::translation_service::Translations::load_from_dir("locales") {
            Ok(t) => {
                let locales = t.available_locales().await;
                tracing::info!(
                    event = "i18n.translations_loaded",
                    locales = ?locales,
                    "per-request translation catalogues loaded"
                );
                t
            },
            Err(error) => {
                tracing::error!(
                    event = "i18n.translations_load_failed",
                    error = %error,
                    "failed to load translation catalogues; falling back to key-only responses"
                );
                backend::services::translation_service::Translations::default()
            },
        };

    let state = web::Data::new(AppState {
        db: db_pool,
        redis: redis_pool,
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(
            redis_pool_for_container.clone(),
            backend::ws::WsLimits::default(),
        ),
        // Cache Arc<Vec<JwtSecretKey>> for cheap O(1) clones in JWT middleware.
        // Avoids cloning the full Vec on every authenticated request.
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        // Per-request translation catalogues loaded above.
        translations: translations.clone(),
    });

    // Record cold-start duration (time from boot_start to AppState ready)
    state.metrics.record_cold_start(boot_start.elapsed());

    // Start WebSocket Pub/Sub listener for distributed message delivery
    let pubsub_state = std::sync::Arc::new(state.ws.clone());
    actix::spawn(async move {
        if let Err(e) = backend::ws::redis_state::run_pubsub_listener(pubsub_state).await {
            tracing::error!("WebSocket Pub/Sub listener failed: {}", e);
        }
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db_pool_for_container,
        redis_pool_for_container,
        (*config).clone(),
    ));

    // Start background audit log chain verifier (hourly by default)
    let audit_repo_for_verifier = container.domain_audit_logs.clone();
    actix::spawn(async move {
        backend::services::audit_log_verifier::run_audit_log_verifier(audit_repo_for_verifier)
            .await;
    });

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
                        // Use the per-request locale (populated by the locale
                        // middleware before any extractor runs) so the parser
                        // error message matches the user's UI language. Falls
                        // back to the global rust_i18n value if the thread-local
                        // is unset (should never happen in normal operation).
                        let message = backend::middleware::locale::current_request_locale()
                            .map(|rl| rl.t_blocking("errors.bad_request_payload", None))
                            .unwrap_or_else(|| {
                                t!("errors.bad_request_payload").into_owned()
                            });
                        AppError::BadRequest(message).into()
                    }),
            )
            .app_data(web::PayloadConfig::new(config_form_limit))
            .wrap(cors)
            // middleware
            // .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::MergeOnly))
            .wrap(backend::middleware::security_headers::SecurityHeaders)
            .wrap(backend::middleware::metrics_middleware::MetricsMiddleware)
            .wrap(backend::middleware::request_log_middleware::RequestLogMiddleware)
            // Per-request locale resolution. Must run BEFORE request_log so
            // the resolved locale appears in the structured log fields.
            .wrap(backend::middleware::locale::LocaleMiddleware::new(
                translations.clone(),
            ))
            .route(
                "/health",
                web::get().to(root_health_check),
            )
            .configure(|cfg| backend::routes::router::config(cfg, pool_for_router.clone()))
            .default_service(web::route().to(not_found))
    };

    let server = HttpServer::new(app);

    let result = if config.tls_enabled {
        // Initialize TLS crypto provider - falls back to default if already initialized
        let _ = rustls::crypto::CryptoProvider::get_default();

        let cert_path = config.tls_cert_path.clone();
        let key_path = config.tls_key_path.clone();

        let mut certs_file = BufReader::new(std::fs::File::open(&cert_path).map_err(|error| {
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
    } else {
        println!("Running in HTTP on {}:{}", host, port);
        server.bind((host, port))?.run().await
    };

    // Shutdown OpenTelemetry provider to flush pending traces
    if let Some(provider) = otel_provider
        && let Err(e) = provider.shutdown()
    {
        tracing::warn!(error = %e, "Failed to shutdown OpenTelemetry provider");
    }

    result
}
