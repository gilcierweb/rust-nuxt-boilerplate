use actix_web::{HttpResponse, get, post, web};
use actix_web_grants::authorities::AuthDetails;
use uuid::Uuid;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, authorize};
use crate::errors::{AppError, AppResult};
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::container::AppContainer;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::validation::first_validation_error_message;

/// List audit logs with pagination and optional sorting.
///
/// Requires the `audit_logs:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs",
    tag = "audit_logs",
    params(
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (max 100)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by"),
        ("sort_dir" = Option<String>, Query, description = "Sort direction: `asc` or `desc`")
    ),
    responses(
        (status = 200, description = "Paginated list of audit logs", body = PaginatedResponse<AuditLog>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `audit_logs:read` authority")
    ),
    security(("bearer_auth" = []))
)]
#[get("/audit-logs")]
pub async fn list_audit_logs(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::AuditLogs, AbilityAction::Read)?;
    let pagination = pagination.into_inner().validated();
    let response = container
        .domain_audit_logs
        .list_paginated(&pagination)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(response))
}

/// Fetch a single audit log entry by its UUID.
///
/// Requires the `audit_logs:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs/{id}",
    tag = "audit_logs",
    params(
        ("id" = Uuid, Path, description = "Audit log identifier")
    ),
    responses(
        (status = 200, description = "Audit log found", body = AuditLog),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `audit_logs:read` authority"),
        (status = 404, description = "Audit log not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/audit-logs/{id}")]
pub async fn get_audit_log(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::AuditLogs, AbilityAction::Read)?;
    let item = container
        .domain_audit_logs
        .find(&id.into_inner())
        .await
        .map_err(|e| AppError::from_diesel(e, "AuditLog"))?;
    Ok(HttpResponse::Ok().json(item))
}

/// Create a new audit log entry.
///
/// Requires the `audit_logs:create` authority.
#[utoipa::path(
    post,
    path = "/api/v1/admin/audit-logs",
    tag = "audit_logs",
    request_body = NewAuditLog,
    responses(
        (status = 201, description = "Audit log created", body = AuditLog),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `audit_logs:create` authority")
    ),
    security(("bearer_auth" = []))
)]
#[post("/audit-logs")]
pub async fn create_audit_log(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    body: web::Json<NewAuditLog>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::AuditLogs, AbilityAction::Create)?;
    let payload = body.into_inner();
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;
    let created = container
        .domain_audit_logs
        .create(&payload)
        .await
        .map_err(AppError::Database)?;
    Ok(HttpResponse::Created().json(created))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_audit_logs)
        .service(get_audit_log)
        .service(create_audit_log);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    use crate::middleware::test_authorities::TestAuthorities;

    cfg.service(web::scope("/admin").wrap(TestAuthorities).configure(config));
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use actix_web::body::to_bytes;
    use actix_web::dev::ServiceRequest;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use chrono::Utc;
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::test_config;
    use crate::middleware::auth::create_token;
    use crate::models::audit_log::AuditLog;
    use crate::repositories::audit_logs_repository::MockIAuditLogRepository;
    use crate::repositories::test_utils::mocks::mock_container;

    #[allow(dead_code)]
    async fn test_extract_authorities(
        req: &ServiceRequest,
    ) -> Result<HashSet<String>, actix_web::Error> {
        let authorities = req
            .headers()
            .get("x-test-authorities")
            .and_then(|value| value.to_str().ok())
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .collect::<HashSet<String>>()
            })
            .unwrap_or_default();

        Ok(authorities)
    }

    fn test_token() -> String {
        create_token(Uuid::new_v4(), Uuid::new_v4(), 1, "", 3600).unwrap()
    }

    #[actix_web::test]
    async fn list_audit_logs_returns_forbidden_without_read_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/audit-logs")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", test_token()),
            ))
            .insert_header(("x-test-authorities", "ROLE_ADMIN"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn list_audit_logs_returns_ok_with_read_authority() {
        use crate::utils::pagination::PaginatedResponse;

        let mut container = mock_container();
        let mut repo = MockIAuditLogRepository::new();
        repo.expect_list_paginated().times(1).returning(|_| {
            Ok(PaginatedResponse::new(
                vec![AuditLog {
                    id: Uuid::new_v4(),
                    actor_user_id: Some(Uuid::new_v4()),
                    actor_role_snapshot: Some("admin".to_string()),
                    action: "create".to_string(),
                    resource_type: "User".to_string(),
                    resource_id: Some(Uuid::new_v4()),
                    ip_address: None,
                    user_agent: None,
                    request_id: None,
                    changes: json!({}),
                    metadata: json!({}),
                    created_at: Utc::now(),
                    prev_hash: None,
                    hash: "a".repeat(64),
                }],
                1,
                1,
                20,
            ))
        });
        container.domain_audit_logs = Arc::new(repo);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/audit-logs")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", test_token()),
            ))
            .insert_header(("x-test-authorities", "ROLE_ADMIN,audit_logs:read"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().map(|items| items.len()), Some(1));
    }
}
