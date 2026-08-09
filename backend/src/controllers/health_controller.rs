use actix_web::http::StatusCode;
use actix_web::{HttpResponse, get, web};
use chrono::Utc;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthDependencyStatus {
    pub status: &'static str,
    pub error: Option<String>,
    pub latency_ms: Option<f64>,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: String,
    pub version: &'static str,
    pub db: HealthDependencyStatus,
    pub redis: HealthDependencyStatus,
}

/// Liveness probe — process is up and responding.
///
/// Returns a static `200` without touching any external dependency
/// (DB, Redis). Used by orchestrators that only need to know the
/// process is alive; not suitable for determining whether the service
/// is ready to handle traffic (use `/api/v1/health` for that).
///
/// # When to use this endpoint
///
/// - **Kubernetes `livenessProbe`**:LB: checks the process can answer HTTP —
///   if this fails, k8s restarts the pod.
/// - **AWS NLB/ALB health checks** (target group): balancing decision.
/// - **Docker `HEALTHCHECK`** + **Wrangler dev** probe.
///
/// Intentionally does NOT probe DB/Redis so a transient dependency
/// blip does not trigger a cascading pod restart (a slow DB is not
/// proof the process is dead).
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Process is alive", body = HealthResponse)
    )
)]
#[get("/health")]
pub async fn liveness() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION"),
        db: HealthDependencyStatus {
            status: "skipped",
            error: None,
            latency_ms: None,
        },
        redis: HealthDependencyStatus {
            status: "skipped",
            error: None,
            latency_ms: None,
        },
    })
}

/// Readiness probe — process can serve traffic (DB + Redis reachable).
///
/// Performs a live probe of the database and Redis connections and returns
/// the operational status, dependency latencies, and the service version.
///
/// # When to use this endpoint
///
/// - **Kubernetes `readinessProbe`**: LB only routes traffic to this pod
///   when this returns `200`; a `503` removes the pod from the endpoint
///   list without restarting it.
/// - **Uptime monitors** (Pingdom, BetterUptime, etc.).
/// - **Prometheus alerton** `health_status != "ok"`.
///
/// Returns `503 Service Unavailable` when any dependency is unreachable
/// so probes can react accordingly — but the pod stays alive (liveness
/// still passes), giving the dependency time to recover.
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "All dependencies healthy", body = HealthResponse),
        (status = 503, description = "One or more dependencies are unreachable", body = HealthResponse)
    )
)]
#[get("/health")]
pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    // --- DB probe with timing ---
    let db_probe_start = std::time::Instant::now();
    let db_result = async {
        let mut conn_obj = state.db.get().await.map_err(|error| error.to_string())?;
        let connection = &mut *conn_obj;
        diesel::sql_query("SELECT 1")
            .execute(connection)
            .await
            .map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    }
    .await;
    let db_latency_ms = db_probe_start.elapsed().as_secs_f64() * 1000.0;

    // Record DB probe timing in metrics registry
    state.metrics.record_db_query(db_probe_start.elapsed());

    let db_status = match db_result {
        Ok(()) => HealthDependencyStatus {
            status: "ok",
            error: None,
            latency_ms: Some(db_latency_ms),
        },
        Err(error) => HealthDependencyStatus {
            status: "down",
            error: Some(error),
            latency_ms: Some(db_latency_ms),
        },
    };

    // --- Redis probe with timing ---
    let redis_probe_start = std::time::Instant::now();
    let redis_result = async {
        let mut connection = state.redis.get().await.map_err(|error| error.to_string())?;
        redis::cmd("PING")
            .query_async::<String>(&mut connection)
            .await
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
    .await;
    let redis_latency_ms = redis_probe_start.elapsed().as_secs_f64() * 1000.0;

    // Record Redis probe timing in metrics registry
    state.metrics.record_redis_op(redis_probe_start.elapsed());

    let redis_status = match redis_result {
        Ok(()) => HealthDependencyStatus {
            status: "ok",
            error: None,
            latency_ms: Some(redis_latency_ms),
        },
        Err(error) => HealthDependencyStatus {
            status: "down",
            error: Some(error),
            latency_ms: Some(redis_latency_ms),
        },
    };

    let overall_status = if db_status.status == "ok" && redis_status.status == "ok" {
        "ok"
    } else {
        "degraded"
    };

    let response = HealthResponse {
        status: overall_status,
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION"),
        db: db_status,
        redis: redis_status,
    };

    let status_code = if overall_status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(response)
}

/// Register health-check routes.
///
/// - `liveness` is mounted at the root `/health` (no `/api/v1` prefix,
///   no middleware) by `main.rs` — see `main.rs` for the registration
///   line; it bypasses the `router::config()` chain intentionally so
///   an API-version / API-key / rate-limit failure does not make the
///   pod look dead to orchestrators.
/// - `health_check` (readiness) is mounted at `/api/v1/health` by
///   `router.rs` via `.configure(health_controller::config)`.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}
