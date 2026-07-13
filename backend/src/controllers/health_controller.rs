use actix_web::{HttpResponse, http::StatusCode, web};
use chrono::Utc;
use diesel::RunQueryDsl;
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
    let db_pool = state.db.clone();
    let db_result = tokio::task::spawn_blocking(move || {
        let mut connection = db_pool.get().map_err(|error| error.to_string())?;
        diesel::sql_query("SELECT 1")
            .execute(&mut connection)
            .map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result);

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
            status: "error",
            error: Some(error),
        },
    };

    let redis_status = match redis_result {
        Ok(()) => HealthDependencyStatus {
            status: "ok",
            error: None,
        },
        Err(error) => HealthDependencyStatus {
            status: "error",
            error: Some(error),
        },
    };

    let overall_status = if db_status.status == "ok" && redis_status.status == "ok" {
        "ok"
    } else {
        "degraded"
    };

    let response_status = if overall_status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = HealthResponse {
        status: overall_status,
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION"),
        db: db_status,
        redis: redis_status,
    };

    HttpResponse::build(response_status).json(body)
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use actix_web::{App, body::to_bytes, http::StatusCode, test, web};
    use diesel::{PgConnection, r2d2::ConnectionManager};
    use serde_json::Value;

    use crate::{
        AppState, config::app_config::Environment,
        repositories::test_utils::mocks::mock_app_config,
        services::metrics_service::MetricsRegistry, ws::WsState,
    };

    use super::health_check;

    fn unavailable_state() -> AppState {
        let db_manager = ConnectionManager::<PgConnection>::new(
            "postgres://invalid:invalid@127.0.0.1:1/invalid_db",
        );
        let db_pool = diesel::r2d2::Pool::builder()
            .max_size(1)
            .min_idle(Some(0))
            .connection_timeout(Duration::from_millis(100))
            .build_unchecked(db_manager);

        let redis_cfg = deadpool_redis::Config::from_url("redis://127.0.0.1:1");
        let redis_pool = redis_cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("failed to create redis test pool");

        let mut config = mock_app_config();
        config.environment = Environment::Test;

        AppState {
            db: db_pool,
            redis: redis_pool,
            config: Arc::new(config),
            metrics: Arc::new(MetricsRegistry::new()),
            ws: WsState::new(),
        }
    }

    #[actix_web::test]
    async fn health_endpoint_returns_service_unavailable_when_dependencies_fail() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(unavailable_state()))
                .route("/health", web::get().to(health_check)),
        )
        .await;

        let request = test::TestRequest::get().uri("/health").to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = to_bytes(response.into_body()).await.expect("response body");
        let json: Value = serde_json::from_slice(&body).expect("valid json response");

        assert_eq!(json["status"], "degraded");
        assert_eq!(json["db"]["status"], "error");
        assert_eq!(json["redis"]["status"], "error");
        assert!(json["db"]["error"].is_string());
        assert!(json["redis"]["error"].is_string());
    }
}
