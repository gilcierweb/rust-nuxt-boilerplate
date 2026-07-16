use actix_web::{HttpResponse, web};

use crate::AppState;

/// Prometheus-compatible metrics endpoint.
///
/// Exposes counters, histograms, P95/P99 gauges, DB/Redis probe timings,
/// cold-start gauge, and system resource measures (memory/CPU).
pub async fn metrics(state: web::Data<AppState>) -> HttpResponse {
    state.metrics.refresh_system_measures();
    let body = state.metrics.render_prometheus();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(body)
}
