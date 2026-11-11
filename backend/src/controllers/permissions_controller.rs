//! Admin catalog of permissions: the human readable names and labels the
//! frontend shows, over the codes the authorization engine reads.
//!
//! The code is the identity of a permission (`{resource}:{action}`): it may
//! already be linked to roles, so a permission is only renamed or described
//! and a delete is refused while a role still carries it.

use actix_web::{HttpResponse, delete, get, patch, post, web};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, CurrentAbility};
use crate::errors::{AppError, AppResult};
use crate::models::permission::{
    NewPermission, PermissionCatalog, PermissionPatch, PermissionWithRoles,
};
use crate::repositories::container::AppContainer;
use crate::repositories::permissions_repository::PERMISSION_SORT_FIELDS;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::sanitize::{sanitize_input, strip_html};
use crate::utils::validation::first_validation_error_message;

/// The body of a permission creation. The resource and the action come from the
/// enums: an unknown pair is refused here, never stored.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PermissionRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "admin.permissions.validation.name_invalid"
    ))]
    name: String,
    resource: String,
    action: String,
    description: Option<String>,
}

/// The body of a permission update. The code is the identity, so the resource
/// and the action are absent on purpose: they cannot be edited.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PermissionUpdateRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "admin.permissions.validation.name_invalid"
    ))]
    name: Option<String>,
    description: Option<String>,
}

/// Splits a permission code into its resource and action parts, refusing an
/// unknown pair with a localized message.
fn parse_code_parts(resource: &str, action: &str) -> AppResult<(AbilityResource, AbilityAction)> {
    let resource_code = AbilityResource::from_code(resource.trim()).ok_or_else(|| {
        AppError::Validation(t!("admin.permissions.validation.resource_invalid").into_owned())
    })?;
    let action_code = AbilityAction::from_code(action.trim()).ok_or_else(|| {
        AppError::Validation(t!("admin.permissions.validation.action_invalid").into_owned())
    })?;

    Ok((resource_code, action_code))
}

/// Normalizes the free text of a permission payload.
fn normalize(payload: &mut PermissionRequest) {
    payload.name = sanitize_input(&strip_html(&payload.name));
    payload.description = payload
        .description
        .as_ref()
        .map(|value| sanitize_input(&strip_html(value)))
        .filter(|value| !value.trim().is_empty());
}

/// Normalizes the free text of a permission patch.
fn normalize_patch(payload: &mut PermissionUpdateRequest) {
    if let Some(name) = payload.name.as_ref() {
        payload.name = Some(sanitize_input(&strip_html(name)));
    }
    payload.description = payload
        .description
        .as_ref()
        .map(|value| sanitize_input(&strip_html(value)))
        .filter(|value| !value.trim().is_empty());
}

/// The permission a unique violation refers to, so the message names the field.
fn unique_violation(error: &diesel::result::Error) -> AppError {
    let constraint = match error {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            inner,
        ) => inner.constraint_name().map(str::to_string),
        _ => None,
    };

    match constraint.as_deref() {
        Some("uq_permissions_name") => {
            AppError::Validation(t!("admin.permissions.validation.name_taken").into_owned())
        },
        _ => AppError::Conflict(t!("admin.permissions.conflict.code_taken").into_owned()),
    }
}

/// Lists the permissions of the catalog with the roles carrying them.
///
/// Requires the `permissions:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/permissions",
    tag = "permissions",
    params(
        ("q" = Option<String>, Query, description = "Search over the code and the name"),
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (max 100)"),
        ("sort_by" = Option<String>, Query, description = "Sort field: `name`, `code` or `updated_at`"),
        ("sort_dir" = Option<String>, Query, description = "Sort direction: `asc` or `desc`")
    ),
    responses(
        (status = 200, description = "Paginated list of permissions", body = PaginatedResponse<PermissionWithRoles>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `permissions:read` authority"),
        (status = 422, description = "Sort field outside the allowlist")
    ),
    security(("bearer_auth" = []))
)]
#[get("/permissions")]
pub async fn all_permissions(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
    search: web::Query<PermissionSearch>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Permissions)?;

    let params = pagination.into_inner().validated();
    if let Some(sort_by) = params.sort_by.as_deref()
        && !PERMISSION_SORT_FIELDS.contains(&sort_by)
    {
        return Err(AppError::Validation(
            t!("admin.permissions.validation.sort_invalid").into_owned(),
        ));
    }

    let search = search
        .into_inner()
        .q
        .map(|value| sanitize_input(&strip_html(&value)))
        .filter(|value| !value.trim().is_empty());

    let response = container
        .permissions
        .list(&params, search)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(response))
}

/// The resources and actions a permission can name, with the i18n keys of
/// their labels.
///
/// Requires the `permissions:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/permissions/catalog",
    tag = "permissions",
    responses(
        (status = 200, description = "Selectable resources and actions", body = PermissionCatalog),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `permissions:read` authority")
    ),
    security(("bearer_auth" = []))
)]
#[get("/permissions/catalog")]
pub async fn permission_catalog(
    ability: CurrentAbility,
    _container: web::Data<AppContainer>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Permissions)?;

    Ok(HttpResponse::Ok().json(PermissionCatalog::from_enums()))
}

/// Creates a permission of the catalog.
///
/// Requires the `permissions:create` authority. The code is derived as
/// `{resource}:{action}`, so an existing pair answers 409.
#[utoipa::path(
    post,
    path = "/api/v1/admin/permissions",
    tag = "permissions",
    request_body = PermissionRequest,
    responses(
        (status = 201, description = "Permission created", body = PermissionWithRoles),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `permissions:create` authority"),
        (status = 409, description = "The code already exists"),
        (status = 422, description = "Unknown resource or action, or a name already taken")
    ),
    security(("bearer_auth" = []))
)]
#[post("/permissions")]
pub async fn create_permission(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    body: web::Json<PermissionRequest>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Create, AbilityResource::Permissions)?;

    let mut payload = body.into_inner();
    normalize(&mut payload);
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    parse_code_parts(&payload.resource, &payload.action)?;

    let new_permission = NewPermission {
        code: format!("{}:{}", payload.resource.trim(), payload.action.trim()),
        name: payload.name,
        description: payload.description,
    };

    let created = container
        .permissions
        .create(&new_permission)
        .await
        .map_err(|error| unique_violation(&error))?;

    let item = container
        .permissions
        .find_with_roles(&created.id)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Created().json(item))
}

/// Updates the name and the description of a permission.
///
/// Requires the `permissions:update` authority. The code is the identity of the
/// permission and never changes.
#[utoipa::path(
    patch,
    path = "/api/v1/admin/permissions/{id}",
    tag = "permissions",
    params(
        ("id" = Uuid, Path, description = "Permission identifier")
    ),
    request_body = PermissionUpdateRequest,
    responses(
        (status = 200, description = "Permission updated", body = PermissionWithRoles),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `permissions:update` authority"),
        (status = 404, description = "Permission not found"),
        (status = 422, description = "The name is already taken")
    ),
    security(("bearer_auth" = []))
)]
#[patch("/permissions/{id}")]
pub async fn update_permission(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
    body: web::Json<PermissionUpdateRequest>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Update, AbilityResource::Permissions)?;

    let mut payload = body.into_inner();
    normalize_patch(&mut payload);
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    // An empty patch would produce a statement that changes nothing: the caller
    // has to say which field it wants to write.
    if payload.name.is_none() && payload.description.is_none() {
        return Err(AppError::Validation(
            t!("admin.permissions.validation.nothing_to_update").into_owned(),
        ));
    }

    let permission_id = id.into_inner();
    // A missing permission is a 404 whatever the body asked for.
    container
        .permissions
        .find(&permission_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Permission"))?;

    let updated = container
        .permissions
        .update(
            &permission_id,
            &PermissionPatch {
                name: payload.name,
                description: payload.description,
            },
        )
        .await
        .map_err(|error| unique_violation(&error))?;

    let item = container
        .permissions
        .find_with_roles(&updated.id)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(item))
}

/// Deletes a permission of the catalog.
///
/// Requires the `permissions:delete` authority. A code still carried by a role
/// is refused: the cascade of the link table would revoke the permission
/// silently, so the admin has to remove the link first.
#[utoipa::path(
    delete,
    path = "/api/v1/admin/permissions/{id}",
    tag = "permissions",
    params(
        ("id" = Uuid, Path, description = "Permission identifier")
    ),
    responses(
        (status = 200, description = "Permission deleted"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `permissions:delete` authority"),
        (status = 404, description = "Permission not found"),
        (status = 409, description = "The code is still linked to a role")
    ),
    security(("bearer_auth" = []))
)]
#[delete("/permissions/{id}")]
pub async fn destroy_permission(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Delete, AbilityResource::Permissions)?;

    let permission_id = id.into_inner();
    container
        .permissions
        .find(&permission_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Permission"))?;

    let linked = container
        .permissions
        .linked_role_count(&permission_id)
        .await
        .map_err(AppError::Database)?;
    if linked > 0 {
        return Err(AppError::Conflict(
            t!("admin.permissions.conflict.still_linked").into_owned(),
        ));
    }

    let deleted = container
        .permissions
        .destroy(&permission_id)
        .await
        .map_err(AppError::Database)?;
    if deleted == 0 {
        return Err(AppError::NotFound("Permission".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "deleted": true })))
}

/// The search box of the list, resolved server side over the code and the name.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PermissionSearch {
    q: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    // The catalog route is registered before the identifier route: the path
    // segment of the identifier would otherwise swallow the word `catalog`.
    cfg.service(permission_catalog)
        .service(all_permissions)
        .service(create_permission)
        .service(update_permission)
        .service(destroy_permission);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin").configure(config));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use serde_json::Value;
    use uuid::Uuid;

    use actix_web::HttpMessage;

    use super::test_config;
    use crate::authz::test_helpers::ability_from_codes;
    use crate::models::permission::{Permission, PermissionWithRoles};
    use crate::repositories::mocks::mock_container;
    use crate::repositories::permissions_repository::MockIPermissionRepository;

    #[actix_web::test]
    async fn all_permissions_returns_forbidden_without_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/permissions")
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(Uuid::new_v4(), &[]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn all_permissions_refuses_a_sort_field_outside_the_allowlist() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/permissions?sort_by=description")
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(Uuid::new_v4(), &["permissions:read"]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn create_permission_returns_created_when_authority_is_present() {
        let mut container = mock_container();
        let mut permissions_repo = MockIPermissionRepository::new();
        permissions_repo.expect_create().times(1).returning(|item| {
            Ok(Permission {
                id: Uuid::new_v4(),
                code: item.code.clone(),
                name: item.name.clone(),
                description: item.description.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        });
        permissions_repo
            .expect_find_with_roles()
            .times(1)
            .returning(|id| {
                Ok(PermissionWithRoles {
                    id: *id,
                    code: "users:read".to_string(),
                    name: "Users read again".to_string(),
                    description: None,
                    resource: "users".to_string(),
                    action: "read".to_string(),
                    roles: Vec::new(),
                })
            });
        container.permissions = Arc::new(permissions_repo);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/admin/permissions")
            .set_json(serde_json::json!({
                "name": "Users read again",
                "resource": "users",
                "action": "read"
            }))
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(Uuid::new_v4(), &["permissions:create"]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "users:read");
    }
}
