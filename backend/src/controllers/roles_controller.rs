use actix_web::{HttpResponse, delete, get, patch, post, web};
use actix_web_grants::authorities::AuthDetails;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, authorize};
use crate::controllers::auth_controller::invalidate_role_cache;
use crate::errors::{AppError, AppResult};
use crate::models::role::{NewRole, Role};
use crate::repositories::container::AppContainer;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::sanitize::{sanitize_input, strip_html};
use crate::utils::validation::first_validation_error_message;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_role_scope", skip_on_field_errors = false))]
pub struct RoleWriteRequest {
    #[validate(length(min = 1, max = 50, message = "admin.roles.validation.name_invalid"))]
    name: String,
    #[validate(length(max = 255, message = "admin.roles.validation.resource_type_invalid"))]
    resource_type: Option<String>,
    resource_id: Option<Uuid>,
}

fn validate_role_scope(payload: &RoleWriteRequest) -> Result<(), validator::ValidationError> {
    if payload.resource_type.is_some() != payload.resource_id.is_some() {
        return Err(validator::ValidationError::new("role_scope")
            .with_message("admin.roles.validation.scope_invalid".into()));
    }

    Ok(())
}

fn normalize_role_payload(payload: &mut RoleWriteRequest) {
    payload.name = sanitize_input(&strip_html(&payload.name));
    payload.resource_type = payload
        .resource_type
        .as_ref()
        .map(|value| sanitize_input(&strip_html(value)))
        .filter(|value| !value.trim().is_empty());
}

/// List roles with pagination and optional sorting.
///
/// Requires the `roles:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles",
    tag = "roles",
    params(
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (max 100)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by"),
        ("sort_dir" = Option<String>, Query, description = "Sort direction: `asc` or `desc`")
    ),
    responses(
        (status = 200, description = "Paginated list of roles", body = PaginatedResponse<Role>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:read` authority")
    ),
    security(("bearer_auth" = []))
)]
#[get("/roles")]
pub async fn list_roles(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Read)?;
    let pagination = pagination.into_inner().validated();
    let response = container
        .roles
        .list_paginated(&pagination)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(response))
}

/// Fetch a single role by its UUID.
///
/// Requires the `roles:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles/{id}",
    tag = "roles",
    params(
        ("id" = Uuid, Path, description = "Role identifier")
    ),
    responses(
        (status = 200, description = "Role found", body = Role),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:read` authority"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/roles/{id}")]
pub async fn get_role(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Read)?;
    let item = container
        .roles
        .find(&id.into_inner())
        .await
        .map_err(|e| AppError::from_diesel(e, "Role"))?;
    Ok(HttpResponse::Ok().json(item))
}

/// Create a new role.
///
/// Requires the `roles:create` authority.
#[utoipa::path(
    post,
    path = "/api/v1/admin/roles",
    tag = "roles",
    request_body = RoleWriteRequest,
    responses(
        (status = 201, description = "Role created", body = Role),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:create` authority")
    ),
    security(("bearer_auth" = []))
)]
#[post("/roles")]
pub async fn create_role(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    body: web::Json<RoleWriteRequest>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Create)?;
    let mut payload = body.into_inner();
    normalize_role_payload(&mut payload);
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    let new_role = NewRole {
        name: payload.name,
        resource_type: payload.resource_type,
        resource_id: payload.resource_id,
    };

    let created = container
        .roles
        .create(&new_role)
        .await
        .map_err(AppError::Database)?;
    Ok(HttpResponse::Created().json(created))
}

/// Update an existing role.
///
/// Requires the `roles:update` authority. Invalidates cached roles for any
/// users assigned to the updated role.
#[utoipa::path(
    patch,
    path = "/api/v1/admin/roles/{id}",
    tag = "roles",
    params(
        ("id" = Uuid, Path, description = "Role identifier")
    ),
    request_body = RoleWriteRequest,
    responses(
        (status = 200, description = "Role updated", body = Role),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:update` authority"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
#[patch("/roles/{id}")]
pub async fn update_role(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
    body: web::Json<RoleWriteRequest>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Update)?;
    let mut payload = body.into_inner();
    normalize_role_payload(&mut payload);
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    let role_id = id.into_inner();
    let new_role = NewRole {
        name: payload.name,
        resource_type: payload.resource_type,
        resource_id: payload.resource_id,
    };

    let updated = container
        .roles
        .update(&role_id, &new_role)
        .await
        .map_err(|e| AppError::from_diesel(e, "Role"))?;

    // Invalidate cached roles for all users assigned to this role
    invalidate_role_cache(&container, &role_id).await;

    Ok(HttpResponse::Ok().json(updated))
}

/// Delete a role by its UUID.
///
/// Requires the `roles:delete` authority. Invalidates cached roles for any
/// users that were assigned to the deleted role.
#[utoipa::path(
    delete,
    path = "/api/v1/admin/roles/{id}",
    tag = "roles",
    params(
        ("id" = Uuid, Path, description = "Role identifier")
    ),
    responses(
        (status = 200, description = "Role deleted"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:delete` authority"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
#[delete("/roles/{id}")]
pub async fn delete_role(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Delete)?;
    let role_id = id.into_inner();
    let affected = container
        .roles
        .destroy(&role_id)
        .await
        .map_err(AppError::Database)?;
    if affected == 0 {
        return Err(AppError::NotFound("Role".to_string()));
    }

    // Invalidate cached roles for all users who had this role
    invalidate_role_cache(&container, &role_id).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "deleted": true })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_roles)
        .service(get_role)
        .service(create_role)
        .service(update_role)
        .service(delete_role);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    use crate::middleware::test_authorities::TestAuthorities;

    cfg.service(web::scope("/admin").wrap(TestAuthorities).configure(config));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use chrono::Utc;
    use serde_json::Value;
    use uuid::Uuid;

    use super::test_config;
    use crate::models::role::Role;
    use crate::repositories::roles_repository::MockIRoleRepository;
    use crate::repositories::test_utils::mocks::mock_container;

    #[actix_web::test]
    async fn list_roles_returns_forbidden_without_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/admin/roles").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "FORBIDDEN");
    }

    #[actix_web::test]
    async fn create_role_returns_forbidden_for_customer_without_create_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/admin/roles")
            .insert_header(("x-test-authorities", "ROLE_CUSTOMER"))
            .set_json(serde_json::json!({
                "name": "customer",
                "resource_type": null,
                "resource_id": null
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "FORBIDDEN");
    }

    #[actix_web::test]
    async fn list_roles_returns_ok_when_authority_is_present() {
        use crate::utils::pagination::PaginatedResponse;

        let mut container = mock_container();
        let mut roles_repo = MockIRoleRepository::new();
        roles_repo.expect_list_paginated().times(1).returning(|_| {
            Ok(PaginatedResponse::new(
                vec![Role {
                    id: Uuid::new_v4(),
                    name: "admin".to_string(),
                    resource_type: None,
                    resource_id: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }],
                1,
                1,
                20,
            ))
        });
        container.roles = Arc::new(roles_repo);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/roles")
            .insert_header(("x-test-authorities", "ROLE_ADMIN,roles:read"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().map(|items| items.len()), Some(1));
    }

    #[actix_web::test]
    async fn create_role_returns_created_when_authority_is_present() {
        let mut container = mock_container();
        let mut roles_repo = MockIRoleRepository::new();
        roles_repo.expect_create().times(1).returning(|item| {
            Ok(Role {
                id: Uuid::new_v4(),
                name: item.name.clone(),
                resource_type: item.resource_type.clone(),
                resource_id: item.resource_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        });
        container.roles = Arc::new(roles_repo);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/admin/roles")
            .insert_header(("x-test-authorities", "ROLE_ADMIN,roles:create"))
            .set_json(serde_json::json!({
                "name": "admin",
                "resource_type": null,
                "resource_id": null
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["name"], "admin");
    }
}
