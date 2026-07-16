use actix_web::{HttpResponse, http::StatusCode, web};
use chrono::Utc;
use diesel_async::RunQueryDsl;
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct HealthDependencyStatus {
    pub status: &'static str,
    pub error: Option<String>,
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

    let redis_result = async {
        let mut connection = state.redis.get().await.map_err(|error| error.to_string())?;
        redis::cmd("PING")
            .query_async::<String>(&mut connection)
            .await
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
    .await;

    let db_status = match db_result {
        Ok(()) => HealthDependencyStatus {
            status: "ok",
            error: None,
        },
        Err(error) => HealthDependencyStatus {
            status: "down",
            error: Some(error),
        },
    };

    let redis_status = match redis_result {
        Ok(()) => HealthDependencyStatus {
            status: "ok",
            error: None,
        },
        Err(error) => HealthDependencyStatus {
            status: "down",
            error: Some(error),
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