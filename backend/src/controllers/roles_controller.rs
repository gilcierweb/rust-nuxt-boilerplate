use actix_web::{HttpResponse, delete, get, patch, post, web};
use actix_web_grants::authorities::AuthDetails;
use diesel::result::Error as DieselError;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    authz::ability::{AbilityAction, AbilityResource, authorize},
    errors::{AppError, AppResult},
    models::role::NewRole,
    repositories::container::AppContainer,
    utils::{
        pagination::{PaginatedResponse, PaginationParams},
        sanitize::{sanitize_input, strip_html},
        validation::first_validation_error_message,
    },
};

fn map_repo_error(error: DieselError, entity: &str) -> AppError {
    match error {
        DieselError::NotFound => AppError::NotFound(entity.to_string()),
        other => AppError::Database(other),
    }
}

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_role_scope", skip_on_field_errors = false))]
struct RoleWriteRequest {
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

#[get("/roles")]
pub async fn list_roles(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Read)?;
    let pagination = pagination.into_inner().validated();
    let items = container.roles.all().await.map_err(AppError::Database)?;

    let total = items.len() as i64;
    let offset = pagination.offset() as usize;
    let limit = pagination.limit() as usize;

    let paginated_data: Vec<_> = items.into_iter().skip(offset).take(limit).collect();
    let response =
        PaginatedResponse::new(paginated_data, total, pagination.page, pagination.per_page);

    Ok(HttpResponse::Ok().json(response))
}

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
        .map_err(|error| map_repo_error(error, "Role"))?;
    Ok(HttpResponse::Ok().json(item))
}

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

    let new_role = NewRole {
        name: payload.name,
        resource_type: payload.resource_type,
        resource_id: payload.resource_id,
    };

    let updated = container
        .roles
        .update(&id.into_inner(), &new_role)
        .await
        .map_err(|error| map_repo_error(error, "Role"))?;
    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/roles/{id}")]
pub async fn delete_role(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Roles, AbilityAction::Delete)?;
    let affected = container
        .roles
        .destroy(&id.into_inner())
        .await
        .map_err(AppError::Database)?;
    if affected == 0 {
        return Err(AppError::NotFound("Role".to_string()));
    }

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

    use actix_web::{App, body::to_bytes, http::StatusCode, test, web};
    use chrono::Utc;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::models::role::Role;
    use crate::repositories::roles_repository::MockIRoleRepository;
    use crate::repositories::test_utils::mocks::mock_container;

    use super::test_config;

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
        let mut container = mock_container();
        let mut roles_repo = MockIRoleRepository::new();
        roles_repo.expect_all().times(1).returning(|| {
            Ok(vec![Role {
                id: Uuid::new_v4(),
                name: "admin".to_string(),
                resource_type: None,
                resource_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
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
