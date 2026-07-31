use actix_web::{HttpResponse, web};

use crate::AppState;

/// Prometheus-compatible metrics endpoint.
///
/// Exposes counters, histograms, P95/P99 gauges, DB/Redis probe timings,
/// cold-start gauge, and system resource measures (memory/CPU).
///
/// # Authentication Required
///
/// This endpoint is protected by the `RequireApiKey` middleware and requires
/// a valid internal API key on every request. Unauthenticated requests receive
/// `401 Unauthorized`.
///
/// Send the key using either header:
/// ```
/// X-API-Key: <key>
/// Authorization: ApiKey <key>
/// ```
///
/// Keys are configured via the `INTERNAL_API_KEYS` environment variable
/// (comma-separated list). See `.env.example` for details.
///
/// # Security Rationale
///
/// The metrics payload contains internal operational data — request counts,
/// latency histograms, route names, pool utilisation, memory/CPU usage, and
/// cold-start timings — that would allow an attacker to enumerate internal
/// topology, infer traffic patterns, and time attacks. Restricting access to
/// authenticated internal services (Prometheus scraper, monitoring dashboards)
/// prevents this information from leaking to unauthenticated callers.
#[utoipa::path(
    get,
    path = "/api/v1/metrics",
    tag = "metrics",
    responses(
        (
            status = 200,
            description = "Prometheus metrics payload (text/plain; version=0.0.4)",
            content_type = "text/plain; version=0.0.4; charset=utf-8"
        ),
        (status = 401, description = "Missing or invalid API key")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn metrics(state: web::Data<AppState>) -> HttpResponse {
    state.metrics.refresh_system_measures();
    let body = state.metrics.render_prometheus();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(body)
}
