//! API bootstrap: observability, dependency wiring, Actix app assembly and TLS.

use std::io::BufReader;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer,
    middleware::{NormalizePath, TrailingSlash},
    web,
};
use deadpool_redis::{Config as RedisConfig, Runtime};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::AppState;
use crate::config::AppConfig;
use crate::config::app_config::{
    Environment, REDIS_POOL_MIN_PRODUCTION, REDIS_POOL_RECOMMENDED_PRODUCTION,
};
use crate::db::database::Database;
use crate::errors::AppError;

/// Default 404 handler. Delegates to `AppError::NotFound` so the response
/// shape and i18n (per-request locale via thread-local) stay consistent with
/// every other error response.
async fn not_found() -> Result<HttpResponse, AppError> {
    Err(AppError::not_found())
}

/// Initialize OpenTelemetry tracer provider with configurable sampling.
///
/// Uses `TelemetryConfig` to read `OTEL_SAMPLER`, `OTEL_SAMPLER_RATIO`,
/// `OTEL_EXPORTER_OTLP_ENDPOINT`, and `OTEL_ENABLED` env vars.
/// Returns None if OTEL is disabled or initialization fails.
fn init_opentelemetry(
    environment: &Environment,
) -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry_otlp::WithExportConfig;

    let telemetry = crate::config::telemetry::TelemetryConfig::from_env();

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
        match environment {
            Environment::Production => {
                tracing::error!(
                    event = "otel.full_coverage_production",
                    sampler = ?telemetry.sampler,
                    "OpenTelemetry is sampling 100% of traces in PRODUCTION. \
                     This will overwhelm the OTLP endpoint and incur high egress costs. \
                     Set OTEL_SAMPLER=ratio_based and OTEL_SAMPLER_RATIO=0.1 (or similar) \
                     to reduce trace volume."
                );
            },
            Environment::Staging => {
                tracing::warn!(
                    event = "otel.full_coverage_staging",
                    sampler = ?telemetry.sampler,
                    "OpenTelemetry is sampling 100% of traces in staging. \
                     Consider OTEL_SAMPLER=ratio_based OTEL_SAMPLER_RATIO=0.5 \
                     to better simulate production conditions."
                );
            },
            Environment::Development | Environment::Test => {},
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

/// Initialize the tracing subscriber, optionally with an OpenTelemetry layer.
/// Returns the tracer provider (if any) so it can be shut down before exit.
fn init_telemetry(
    environment: &Environment,
) -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    let otel_provider = init_opentelemetry(environment);

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

    if let Some(ref provider) = otel_provider {
        use opentelemetry::trace::TracerProvider;
        let tracer = provider.tracer("backend-api");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry.with(otel_layer).init();
    } else {
        registry.init();
    }

    otel_provider
}

/// Load .env from project root, parse and validate the application config.
fn load_config() -> Arc<AppConfig> {
    let env_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR parent")
        .join(".env");
    dotenvy::from_path(env_path).ok();

    let config = AppConfig::from_env().expect("Failed to load configuration");
    config.validate_or_panic();

    Arc::new(config)
}

/// Runtime handles shared with the HTTP application and background tasks.
struct AppRuntime {
    state: web::Data<AppState>,
    container: web::Data<crate::repositories::AppContainer>,
    ws_state: web::Data<crate::ws::WsRedisState>,
    translations: crate::services::translation_service::Translations,
}

/// Build the Redis connection pool from config, logging sizing guidance for
/// production workloads.
fn build_redis_pool(config: &AppConfig) -> deadpool_redis::Pool {
    let mut redis_cfg = RedisConfig::from_url(&config.redis_url);
    redis_cfg.pool = Some(deadpool_redis::PoolConfig::new(config.redis_pool_size));

    // Log Redis pool configuration for debugging
    tracing::info!(
        event = "redis.pool_config",
        pool_size = config.redis_pool_size,
        "Redis connection pool configured"
    );

    let is_production = matches!(config.environment, Environment::Production);

    // Warn if pool size is too low for production workloads
    if is_production && config.redis_pool_size < REDIS_POOL_MIN_PRODUCTION {
        tracing::error!(
            event = "redis.pool_size_below_minimum",
            pool_size = config.redis_pool_size,
            recommended = REDIS_POOL_MIN_PRODUCTION,
            "Redis pool size is below the production minimum. \
             Consider increasing REDIS_POOL_SIZE to {}+ for high-concurrency workloads \
             (rate limiting, caching, session storage, token blacklisting).",
            REDIS_POOL_MIN_PRODUCTION
        );
    } else if is_production && config.redis_pool_size < REDIS_POOL_RECOMMENDED_PRODUCTION {
        tracing::warn!(
            event = "redis.pool_size_below_recommended",
            pool_size = config.redis_pool_size,
            recommended = REDIS_POOL_RECOMMENDED_PRODUCTION,
            "Redis pool size is below the recommended production value of {}. \
             Current size ({}) may be insufficient for high-concurrency workloads \
             (rate limiting, caching, session storage, token blacklisting, WebSocket Pub/Sub). \
             Consider setting REDIS_POOL_SIZE={} or higher.",
            REDIS_POOL_RECOMMENDED_PRODUCTION,
            config.redis_pool_size,
            REDIS_POOL_RECOMMENDED_PRODUCTION
        );
    }

    redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis connection pool")
}

/// Load the per-request translation catalogues from the backend `locales/`
/// directory. On failure, falls back to key-only responses.
///
/// The resulting `Translations` handle is shared with every request via the
/// locale middleware (see `middleware::locale::LocaleMiddleware`). This is
/// the per-request translation mechanism — it does not mutate any global
/// state, so concurrent requests cannot leak locale to each other.
async fn load_translations() -> crate::services::translation_service::Translations {
    match crate::services::translation_service::Translations::load_from_dir("locales") {
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
            crate::services::translation_service::Translations::default()
        },
    }
}

/// Build the DB/Redis pools, WebSocket state, translations and the AppState.
async fn build_runtime(config: &Arc<AppConfig>, boot_start: std::time::Instant) -> AppRuntime {
    let api_db = Database::from_config(config);
    let db_pool = api_db.pool.clone();
    let db_pool_for_container = db_pool.clone();

    let redis_pool = build_redis_pool(config);
    let redis_pool_for_container = redis_pool.clone();

    // Single shared WebSocket state: the connection handler (served via
    // `web::Data`) and the Pub/Sub listener (served via `AppState.ws`) MUST
    // observe the same `local_connections` map, otherwise cross-worker
    // broadcasts are never delivered to local sockets.
    let ws_state = crate::ws::WsRedisState::new(redis_pool.clone(), crate::ws::WsLimits::default());

    let translations = load_translations().await;

    let state = web::Data::new(AppState {
        db: db_pool,
        redis: redis_pool,
        config: config.clone(),
        metrics: Arc::new(crate::services::metrics_service::MetricsRegistry::new()),
        ws: ws_state.clone(),
        // Cache Arc<Vec<JwtSecretKey>> for cheap O(1) clones in JWT middleware.
        // Avoids cloning the full Vec on every authenticated request.
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        // Per-request translation catalogues loaded above.
        translations: translations.clone(),
    });

    // Record cold-start duration (time from boot_start to AppState ready)
    state.metrics.record_cold_start(boot_start.elapsed());

    let container = web::Data::new(crate::repositories::AppContainer::new(
        db_pool_for_container,
        redis_pool_for_container,
        (**config).clone(),
    ));

    AppRuntime {
        state,
        container,
        ws_state: web::Data::new(ws_state),
        translations,
    }
}

/// Spawn the WebSocket Pub/Sub listener and the audit log chain verifier.
fn start_background_tasks(
    state: &web::Data<AppState>,
    container: &web::Data<crate::repositories::AppContainer>,
) {
    // Start WebSocket Pub/Sub listener for distributed message delivery
    let pubsub_state = std::sync::Arc::new(state.ws.clone());
    actix::spawn(async move {
        if let Err(e) = crate::ws::redis_state::run_pubsub_listener(pubsub_state).await {
            tracing::error!("WebSocket Pub/Sub listener failed: {}", e);
        }
    });

    // Start background audit log chain verifier (hourly by default)
    let audit_repo_for_verifier = container.domain_audit_logs.clone();
    actix::spawn(async move {
        crate::services::audit_log_verifier::run_audit_log_verifier(audit_repo_for_verifier).await;
    });
}

/// Parse CORS origins from config (validation done in AppConfig::validate()).
fn parse_cors_origins(config: &AppConfig) -> Vec<String> {
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

    cors_origins
}

fn build_cors(cors_origins: &[String]) -> Cors {
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

    for origin in cors_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}

/// Load and parse the TLS certificate/key pair from config.
fn load_tls_config(config: &AppConfig) -> std::io::Result<rustls::ServerConfig> {
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

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .map_err(std::io::Error::other)
}

/// Shutdown the OpenTelemetry provider to flush pending traces.
fn shutdown_otel(provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>) {
    if let Some(provider) = provider
        && let Err(e) = provider.shutdown()
    {
        tracing::warn!(error = %e, "Failed to shutdown OpenTelemetry provider");
    }
}

/// Bootstrap and run the HTTP API server until shutdown.
pub async fn run() -> std::io::Result<()> {
    let boot_start = std::time::Instant::now();

    rust_i18n::set_locale("pt-BR");

    let config = load_config();
    let otel_provider = init_telemetry(&config.environment);

    tracing::info!(
        event = "server.starting",
        version = env!("CARGO_PKG_VERSION"),
        host = %config.host,
        port = config.port,
        "Starting Backend API"
    );

    let runtime = build_runtime(&config, boot_start).await;
    start_background_tasks(&runtime.state, &runtime.container);

    let cors_origins = parse_cors_origins(&config);
    let json_limit = config.json_payload_limit;
    let form_limit = config.form_payload_limit;

    // Actix application factory shared by every worker.
    let app = move || {
        let pool_for_router = runtime.state.redis.clone();

        App::new()
            .app_data(runtime.state.clone())
            .app_data(runtime.container.clone())
            .app_data(runtime.ws_state.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(json_limit)
                    .error_handler(|_error, _request| {
                        // Use the per-request locale (populated by the locale
                        // middleware before any extractor runs) so the parser
                        // error message matches the user's UI language. Falls
                        // back to the global rust_i18n value if the thread-local
                        // is unset (should never happen in normal operation).
                        let message = crate::middleware::locale::current_request_locale()
                            .map(|rl| rl.t_blocking("errors.bad_request_payload", None))
                            .unwrap_or_else(|| {
                                t!("errors.bad_request_payload").into_owned()
                            });
                        AppError::BadRequest(message).into()
                    }),
            )
            .app_data(web::PayloadConfig::new(form_limit))
            .wrap(build_cors(&cors_origins))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(NormalizePath::new(TrailingSlash::MergeOnly))
            .wrap(crate::middleware::security_headers::SecurityHeaders)
            .wrap(crate::middleware::metrics_middleware::MetricsMiddleware)
            .wrap(crate::middleware::request_log_middleware::RequestLogMiddleware)
            // Per-request locale resolution. Must run BEFORE request_log so
            // the resolved locale appears in the structured log fields.
            .wrap(crate::middleware::locale::LocaleMiddleware::new(
                runtime.translations.clone(),
            ))
            .service(crate::controllers::health_controller::liveness)
            .configure(|cfg| crate::routes::router::config(cfg, pool_for_router.clone()))
            .default_service(web::route().to(not_found))
    };

    let server = HttpServer::new(app);

    let host = config.host.clone();
    let result = if config.tls_enabled {
        let tls_config = load_tls_config(&config)?;
        let https_port = config.https_port;
        tracing::info!(
            event = "server.bind",
            host = %host,
            port = https_port,
            scheme = "https",
            "HTTP server bound"
        );
        server
            .bind_rustls_0_23((host.clone(), https_port), tls_config)?
            .run()
            .await
    } else {
        let port = config.port;
        tracing::info!(
            event = "server.bind",
            host = %host,
            port = port,
            scheme = "http",
            "HTTP server bound"
        );
        server.bind((host, port))?.run().await
    };

    shutdown_otel(otel_provider);

    result
}
