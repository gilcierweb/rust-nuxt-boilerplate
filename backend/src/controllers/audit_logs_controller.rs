use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, CurrentAbility};
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::container::AppContainer;
use crate::repositories::traits::audit_logs_trait::AUDIT_LOG_SORT_FIELDS;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::sanitize::{sanitize_input, strip_html};
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
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
    search: web::Query<ListSearch>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::AuditLogs)?;
    let params = pagination.into_inner().validated();
    params.validate_sort_field(&AUDIT_LOG_SORT_FIELDS)?;
    let search = search
        .into_inner()
        .q
        .map(|value| sanitize_input(&strip_html(&value)))
        .filter(|value| !value.trim().is_empty());

    let response = container
        .domain_audit_logs
        .list_paginated(&params, search)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(response))
}

/// The search box of a list, resolved server side.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListSearch {
    q: Option<String>,
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
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::AuditLogs)?;
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
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    user: AuthUser,
    body: web::Json<NewAuditLog>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Create, AbilityResource::AuditLogs)?;
    let mut payload = body.into_inner();

    // SECURITY: actor_user_id must come from authenticated user, not request body
    // This prevents forging audit log entries as other users
    payload.actor_user_id = Some(user.claims().sub);
    payload.actor_role_snapshot = payload
        .actor_role_snapshot
        .as_ref()
        .map(|value| sanitize_input(&strip_html(value)))
        .filter(|value| !value.trim().is_empty());
    payload.action = sanitize_input(&strip_html(&payload.action));
    payload.resource_type = sanitize_input(&strip_html(&payload.resource_type));
    payload.user_agent = payload
        .user_agent
        .as_ref()
        .map(|value| sanitize_input(&strip_html(value)))
        .filter(|value| !value.trim().is_empty());
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
    cfg.service(web::scope("/admin").configure(config));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::HttpMessage;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use chrono::Utc;
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::test_config;
    use crate::authz::test_helpers::ability_from_codes;
    use crate::middleware::auth::{Claims, create_token};
    use crate::models::audit_log::AuditLog;
    use crate::repositories::audit_logs_repository::MockIAuditLogRepository;
    use crate::repositories::mocks::mock_container;

    fn test_token() -> String {
        create_token(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            "",
            3600,
            crate::middleware::auth::DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap()
    }

    fn test_claims() -> Claims {
        Claims {
            sub: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            role: crate::models::role::ROLE_ADMIN.as_i32(),
            token_use: "access".to_string(),
            exp: 0,
            iat: 0,
            nbf: 0,
            aud: crate::middleware::auth::DEFAULT_TOKEN_AUDIENCE.to_string(),
            jti: None,
        }
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
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(Uuid::new_v4(), &[]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn list_audit_logs_returns_ok_with_read_authority() {
        use crate::utils::pagination::PaginatedResponse;

        let mut container = mock_container();
        let mut repo = MockIAuditLogRepository::new();
        repo.expect_list_paginated().times(1).returning(|_, _| {
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
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(Uuid::new_v4(), &["audit_logs:read"]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().map(|items| items.len()), Some(1));
    }

    #[actix_web::test]
    async fn create_audit_log_strips_html_from_text_fields() {
        use crate::models::audit_log::NewAuditLog;

        let mut container = mock_container();
        let mut repo = MockIAuditLogRepository::new();
        repo.expect_create()
            .times(1)
            .returning(|payload: &NewAuditLog| {
                assert!(!payload.action.contains("<script>"));
                assert!(payload.action.contains("user.create"));
                assert!(
                    payload
                        .actor_role_snapshot
                        .as_ref()
                        .map(|s| !s.contains("<img"))
                        .unwrap_or(true)
                );
                assert!(
                    payload
                        .user_agent
                        .as_ref()
                        .map(|s| !s.contains("<script>"))
                        .unwrap_or(true)
                );
                Ok(AuditLog {
                    id: Uuid::new_v4(),
                    actor_user_id: payload.actor_user_id,
                    actor_role_snapshot: payload.actor_role_snapshot.clone(),
                    action: payload.action.clone(),
                    resource_type: payload.resource_type.clone(),
                    resource_id: payload.resource_id,
                    ip_address: None,
                    user_agent: payload.user_agent.clone(),
                    request_id: None,
                    changes: json!({}),
                    metadata: json!({}),
                    created_at: Utc::now(),
                    prev_hash: None,
                    hash: "a".repeat(64),
                })
            });
        container.domain_audit_logs = Arc::new(repo);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/admin/audit-logs")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", test_token()),
            ))
            .set_json(json!({
                "actor_user_id": Uuid::new_v4(),
                "actor_role_snapshot": "<img src=x onerror=alert(1)>admin",
                "action": "user.<script>alert(1)</script>create",
                "resource_type": "User",
                "user_agent": "<script>x</script>Mozilla/5.0",
                "changes": {},
                "metadata": {},
                "hash": "a".repeat(64),
            }))
            .to_request();
        req.extensions_mut().insert(test_claims());
        req.extensions_mut()
            .insert(ability_from_codes(Uuid::new_v4(), &["audit_logs:create"]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}
