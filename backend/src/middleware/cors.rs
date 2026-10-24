//! CORS middleware configuration for the API.
//!
//! `app.rs` wires this into the app factory; the construction lives here with
//! the rest of the HTTP middleware so the policy and its tests stay together.

use actix_cors::Cors;

use crate::config::AppConfig;

/// Parse CORS origins from config (validation done in `AppConfig::validate()`).
pub fn parse_cors_origins(config: &AppConfig) -> Vec<String> {
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

/// Build the CORS middleware.
///
/// Origins are explicit (`supports_credentials()` never combines with a
/// wildcard) and the header allow-list covers every header the SPA and the
/// Nitro proxy actually send: bearer credentials, the CSRF double-submit token
/// and the request-id used for cross-layer correlation.
pub fn build_cors(cors_origins: &[String]) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::ACCEPT,
            actix_web::http::header::ACCESS_CONTROL_REQUEST_HEADERS,
            // The SPA submits the CSRF double-submit header on refresh/logout,
            // and the proxy propagates the Nitro request-id for correlation.
            // Without them a cross-origin deployment fails its preflight.
            actix_web::http::header::HeaderName::from_static("csrf-token"),
            actix_web::http::header::HeaderName::from_static("x-request-id"),
        ])
        .allowed_header("x-api-key")
        .supports_credentials()
        .max_age(3600);

    for origin in cors_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}
