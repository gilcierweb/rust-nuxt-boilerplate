use actix_web::{HttpResponse, http::StatusCode, web};
use chrono::Utc;
use diesel_async::RunQueryDsl;
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct HealthDependencyStatus {
    pub status: &'static str,
    pub error: Option<String>,
    pub latency_ms: Option<f64>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: String,
    pub version: &'static str,
    pub db: HealthDependencyStatus,
    pub redis: HealthDependencyStatus,
}

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
