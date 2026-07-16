use actix_web::{HttpResponse, delete, get, patch, post, web};
use actix_web_grants::authorities::AuthDetails;
use diesel::result::Error as DieselError;
use uuid::Uuid;
use validator::Validate;

use crate::{
    authz::ability::{AbilityAction, AbilityResource, authorize},
    errors::{AppError, AppResult},
    models::audit_log::NewAuditLog,
    repositories::container::AppContainer,
    utils::{
        pagination::{PaginatedResponse, PaginationParams},
        validation::first_validation_error_message,
    },
};

fn map_repo_error(error: DieselError, entity: &str) -> AppError {
    match error {
        DieselError::NotFound => AppError::NotFound(entity.to_string()),
        other => AppError::Database(other),
    }
}

#[get("/audit-logs")]
pub async fn list_audit_logs(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::AuditLogs, AbilityAction::Read)?;
    let pagination = pagination.into_inner().validated();
    let items = container
        .domain_audit_logs
        .all()
        .await
        .map_err(AppError::Database)?;

    let total = items.len() as i64;
    let offset = pagination.offset() as usize;
    let limit = pagination.limit() as usize;

    let paginated_data: Vec<_> = items.into_iter().skip(offset).take(limit).collect();
    let response = PaginatedResponse::new(paginated_data, total, pagination.page, pagination.per_page);

    Ok(HttpResponse::Ok().json(response))
}

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
        .map_err(|error| map_repo_error(error, "AuditLog"))?;
    Ok(HttpResponse::Ok().json(item))
}

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

#[patch("/audit-logs/{id}")]
pub async fn update_audit_log(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
    body: web::Json<NewAuditLog>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::AuditLogs, AbilityAction::Update)?;
    let payload = body.into_inner();
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;
    let updated = container
        .domain_audit_logs
        .update(&id.into_inner(), &payload)
        .await
        .map_err(|error| map_repo_error(error, "AuditLog"))?;
    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/audit-logs/{id}")]
pub async fn delete_audit_log(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::AuditLogs, AbilityAction::Delete)?;
    let audit_log_id = id.into_inner();
    let affected = container
        .domain_audit_logs
        .destroy(&audit_log_id)
        .await
        .map_err(|error| map_repo_error(error, "AuditLog"))?;
    if affected == 0 {
        return Err(AppError::NotFound("AuditLog".to_string()));
    }
    Ok(HttpResponse::NoContent().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_audit_logs)
        .service(get_audit_log)
        .service(create_audit_log)
        .service(update_audit_log)
        .service(delete_audit_log);
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

    use actix_web::{App, body::to_bytes, dev::ServiceRequest, http::StatusCode, test, web};
    use chrono::Utc;
    use serde_json::{Value, json};
    use uuid::Uuid;

    use crate::middleware::auth::create_token;
    use crate::models::audit_log::AuditLog;
    use crate::repositories::audit_logs_repository::MockIAuditLogRepository;
    use crate::repositories::test_utils::mocks::mock_container;

    use super::test_config;

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
        let mut container = mock_container();
        let mut repo = MockIAuditLogRepository::new();
        repo.expect_all().times(1).returning(|| {
            Ok(vec![AuditLog {
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
            }])
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