use std::sync::Arc;

use actix_web::{HttpResponse, delete, get, patch, post, put, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, CurrentAbility, PermissionEffect};
use crate::authz::guards::ADMINISTRATOR_CODES;
use crate::controllers::auth_controller::invalidate_role_cache;
use crate::errors::{AppError, AppResult};
use crate::models::permission::RolePermissionItem;
use crate::models::role::{NewRole, Role};
use crate::repositories::container::AppContainer;
use crate::repositories::traits::roles_trait::{ROLE_SORT_FIELDS, RoleMemberItem};
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

    // A scope names a resource of the catalog: an arbitrary string would let a
    // role scoped to one resource grant its codes on a record of another,
    // because the instance check compares the target id and never the type.
    if let Some(resource_type) = payload.resource_type.as_deref()
        && AbilityResource::from_code(resource_type).is_none()
    {
        return Err(validator::ValidationError::new("role_resource_type")
            .with_message("admin.roles.validation.resource_type_unknown".into()));
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
pub async fn all_roles(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
    search: web::Query<ListSearch>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Roles)?;
    let params = pagination.into_inner().validated();
    params.validate_sort_field(&ROLE_SORT_FIELDS)?;
    let search = search
        .into_inner()
        .q
        .map(|value| sanitize_input(&strip_html(&value)))
        .filter(|value| !value.trim().is_empty());

    let response = container
        .roles
        .list_paginated(&params, search)
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
pub async fn find_role(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Roles)?;
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
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    body: web::Json<RoleWriteRequest>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Create, AbilityResource::Roles)?;
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
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
    body: web::Json<RoleWriteRequest>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Update, AbilityResource::Roles)?;
    // The guard below reads the administrator sources and the write runs after,
    // so two concurrent requests could each see the other's survivor.
    let _write = crate::authz::guards::hold_administrator_write().await;
    let mut payload = body.into_inner();
    normalize_role_payload(&mut payload);
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    let role_id = id.into_inner();
    let current = container
        .roles
        .find(&role_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Role"))?;

    // The two write time guards of the role catalog: the system depends on the
    // protected names, and a role name answers exactly one role.
    crate::authz::guards::ensure_role_name_is_not_protected(&current.name, &payload.name)?;
    let name_holder = container
        .roles
        .find_by_name(&payload.name)
        .await
        .map_err(AppError::Database)?
        .map(|role| role.id);
    crate::authz::guards::ensure_role_name_is_free(name_holder, role_id)?;

    // A scoped rule grants nothing at the class level, so scoping a role that
    // is an administrator source today would empty the class level sources at
    // once: every holder would lose the roles read and the roles update.
    if current.resource_id.is_none() && payload.resource_id.is_some() {
        let holders = container
            .roles
            .role_ids_holding_all_codes(&administrator_codes())
            .await
            .map_err(AppError::Database)?;
        let holders_after: Vec<Uuid> = holders
            .into_iter()
            .filter(|holder| *holder != role_id)
            .collect();
        crate::authz::guards::ensure_role_administration_survives(&holders_after)?;
    }

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

/// The search box of a list, resolved server side.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListSearch {
    q: Option<String>,
}

/// The replacement of every link of a role.
///
/// The effect is the stored SMALLINT and the handler only accepts the two
/// values the model defines: the database stays CHECK free (AGENTS.md database
/// conventions) and the refusal happens before the write.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RolePermissionsRequest {
    #[validate(length(min = 1, message = "admin.roles.validation.permissions_empty"))]
    permissions: Vec<RolePermissionItem>,
}

/// Validates the payload of a link replacement: every code must belong to the
/// catalog and every effect must be one the model defines.
fn validate_permission_links(items: &[RolePermissionItem]) -> AppResult<()> {
    for item in items {
        let code = item.code.trim();
        let Some((resource_code, action_code)) = code.split_once(':') else {
            return Err(AppError::Validation(
                t!("admin.roles.validation.code_unknown").into_owned(),
            ));
        };

        if AbilityResource::from_code(resource_code).is_none()
            || AbilityAction::from_code(action_code).is_none()
        {
            return Err(AppError::Validation(
                t!("admin.roles.validation.code_unknown").into_owned(),
            ));
        }

        if item.effect().is_none() {
            return Err(AppError::Validation(
                t!("admin.roles.validation.effect_invalid").into_owned(),
            ));
        }
    }

    Ok(())
}

/// The administrator codes as owned strings, the shape the repository reads.
fn administrator_codes() -> Vec<String> {
    ADMINISTRATOR_CODES
        .iter()
        .map(|code| (*code).to_string())
        .collect()
}

/// The roles that would still hold the administrator codes after a link
/// replacement of `role_id`: every current holder except the edited one, plus
/// the edited one when the new set still carries both codes.
fn administrator_sources_after(
    holders_before: &[Uuid],
    role_id: Uuid,
    items: &[RolePermissionItem],
) -> Vec<Uuid> {
    let mut after: Vec<Uuid> = holders_before
        .iter()
        .copied()
        .filter(|holder| *holder != role_id)
        .collect();

    let keeps_administrator_codes = ADMINISTRATOR_CODES.iter().all(|code| {
        items
            .iter()
            .any(|item| &item.code == code && item.effect() == Some(PermissionEffect::Can))
    });
    if keeps_administrator_codes {
        after.push(role_id);
    }

    after
}

/// The permission links of a role, as the admin interface reads them back.
///
/// Requires the `roles:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles/{id}/permissions",
    tag = "roles",
    params(
        ("id" = Uuid, Path, description = "Role identifier")
    ),
    responses(
        (status = 200, description = "Permission links of the role", body = RolePermissionLinksResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:read` authority"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/roles/{id}/permissions")]
pub async fn find_role_permissions(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Roles)?;
    let role_id = id.into_inner();
    container
        .roles
        .find(&role_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Role"))?;

    let links = container
        .roles
        .list_permission_links(&role_id)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(RolePermissionLinksResponse { permissions: links }))
}

/// Replaces every permission link of a role in one transaction.
///
/// Requires the `roles:update` authority. Refuses when the change would leave
/// no role holding both administrator codes, because nobody could repair the
/// system afterwards.
#[utoipa::path(
    put,
    path = "/api/v1/admin/roles/{id}/permissions",
    tag = "roles",
    params(
        ("id" = Uuid, Path, description = "Role identifier")
    ),
    request_body = RolePermissionsRequest,
    responses(
        (status = 200, description = "Links replaced", body = UpdatedCountResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:update` authority, or a policy refusal (last administrator source, escalation, protected role)"),
        (status = 404, description = "Role not found"),
        (status = 422, description = "Unknown code or effect")
    ),
    security(("bearer_auth" = []))
)]
#[put("/roles/{id}/permissions")]
pub async fn update_role_permissions(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
    body: web::Json<RolePermissionsRequest>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Update, AbilityResource::Roles)?;
    // The same serialization as the role update: the holder set the guard reads
    // has to stay the one the write answers to.
    let _write = crate::authz::guards::hold_administrator_write().await;
    let payload = body.into_inner();
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;
    validate_permission_links(&payload.permissions)?;

    // A code that parses is not a code that exists: the catalog is editable at
    // runtime, so a payload may name a model pair whose row the admin removed.
    // The replacement deletes every link before it writes, so an unresolvable
    // code would empty the role while the response reports a smaller number.
    let catalog: std::collections::HashSet<String> = container
        .permissions
        .all()
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|permission| permission.code)
        .collect();
    let unknown: Vec<&str> = payload
        .permissions
        .iter()
        .map(|item| item.code.trim())
        .filter(|code| !catalog.contains(*code))
        .collect();
    if !unknown.is_empty() {
        return Err(AppError::Validation(
            t!("admin.roles.validation.code_unknown").into_owned(),
        ));
    }

    let role_id = id.into_inner();
    container
        .roles
        .find(&role_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Role"))?;

    let holders_before = container
        .roles
        .role_ids_holding_all_codes(&administrator_codes())
        .await
        .map_err(AppError::Database)?;
    let holders_after = administrator_sources_after(&holders_before, role_id, &payload.permissions);
    crate::authz::guards::ensure_role_administration_survives(&holders_after)?;

    // The actor may only grant what they hold: without this check one holder
    // of the two administrator codes could hand every customer the user
    // delete, an authority nobody gave them.
    let actor_allow = ability.permissions().can;
    let role_allow: std::collections::HashSet<String> = payload
        .permissions
        .iter()
        .filter(|item| item.effect() == Some(PermissionEffect::Can))
        .map(|item| item.code.clone())
        .collect();
    crate::authz::guards::ensure_no_escalation(&actor_allow, &role_allow)?;

    let updated = container
        .roles
        .replace_permission_links(&role_id, &payload.permissions)
        .await
        .map_err(AppError::Database)?;

    // The ability of the members of the role changed: their cached grants and
    // tokens have to go.
    invalidate_role_cache(&container, &role_id).await;

    Ok(HttpResponse::Ok().json(UpdatedCountResponse { updated }))
}

/// The members of a role, with the decrypted email.
///
/// Requires the `roles:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles/{id}/members",
    tag = "roles",
    params(
        ("id" = Uuid, Path, description = "Role identifier")
    ),
    responses(
        (status = 200, description = "Members of the role", body = Vec<RoleMemberItem>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `roles:read` authority"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/roles/{id}/members")]
pub async fn find_role_members(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Roles)?;
    let role_id = id.into_inner();
    container
        .roles
        .find(&role_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Role"))?;

    let security = crate::security::SecurityService::from_config(&container.config)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let members = container
        .roles
        .list_members(&role_id, Arc::new(security))
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(members))
}

/// Delete a role by its UUID.
///
/// Requires the `roles:delete` authority. Refuses the administrator role, the
/// role the registration resolves by default and any deletion that would leave
/// no role holding the administrator codes.
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
pub async fn destroy_role(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Delete, AbilityResource::Roles)?;
    // The guard below reads the administrator sources and the deletion runs
    // after, so two concurrent deletions could each see the other's survivor.
    let _write = crate::authz::guards::hold_administrator_write().await;
    let role_id = id.into_inner();

    let role = container
        .roles
        .find(&role_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "Role"))?;

    // The administrator role is the one every deployment keeps: deleting it
    // would take the whole catalog of links with it.
    if role.name == "admin" {
        return Err(AppError::Forbidden(
            t!("admin.roles.guard.administrator_role").into_owned(),
        ));
    }

    // The registration resolves its default role by name, so the row cannot
    // disappear while REGISTER_DEFAULT_ROLE points at it.
    if role.name == container.config.register_default_role {
        return Err(AppError::Forbidden(
            t!("admin.roles.guard.default_role").into_owned(),
        ));
    }

    // And no deletion may empty the administrator sources.
    let holders = container
        .roles
        .role_ids_holding_all_codes(&administrator_codes())
        .await
        .map_err(AppError::Database)?;
    let remaining: Vec<Uuid> = holders
        .into_iter()
        .filter(|holder| *holder != role_id)
        .collect();
    crate::authz::guards::ensure_role_administration_survives(&remaining)?;

    // Get all users who have this role BEFORE deleting (for token revocation)
    let affected_users = container
        .user_roles
        .find_by_role(&role_id)
        .await
        .map_err(AppError::Database)?;

    let affected = container
        .roles
        .destroy(&role_id)
        .await
        .map_err(AppError::Database)?;
    if affected == 0 {
        return Err(AppError::NotFound("Role".to_string()));
    }

    // Revoke all refresh tokens for affected users (force re-authentication)
    for user_role in &affected_users {
        let _ = container
            .refresh_tokens
            .revoke_all_for_user(&user_role.user_id)
            .await;
    }

    // Invalidate cached roles for all users who had this role
    invalidate_role_cache(&container, &role_id).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "deleted": true })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(all_roles)
        .service(find_role)
        .service(create_role)
        .service(update_role)
        .service(destroy_role)
        .service(find_role_permissions)
        .service(update_role_permissions)
        .service(find_role_members);
}

/// The links of a role as the admin interface reads them back.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RolePermissionLinksResponse {
    permissions: Vec<RolePermissionItem>,
}

/// The number of links a replacement wrote.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatedCountResponse {
    updated: usize,
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
    use chrono::Utc;
    use serde_json::Value;
    use uuid::Uuid;

    use actix_web::HttpMessage;

    use super::test_config;
    use crate::authz::test_helpers::ability_from_codes;
    use crate::models::role::Role;
    use crate::repositories::mocks::mock_container;
    use crate::repositories::roles_repository::MockIRoleRepository;

    #[actix_web::test]
    async fn all_roles_returns_forbidden_without_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/admin/roles").to_request();
        req.extensions_mut()
            .insert(ability_from_codes(uuid::Uuid::new_v4(), &[]));
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
            .set_json(serde_json::json!({
                "name": "customer",
                "resource_type": null,
                "resource_id": null
            }))
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(uuid::Uuid::new_v4(), &[]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "FORBIDDEN");
    }

    #[actix_web::test]
    async fn all_roles_returns_ok_when_authority_is_present() {
        use crate::utils::pagination::PaginatedResponse;

        let mut container = mock_container();
        let mut roles_repo = MockIRoleRepository::new();
        roles_repo
            .expect_list_paginated()
            .times(1)
            .returning(|_, _| {
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

        let req = test::TestRequest::get().uri("/admin/roles").to_request();
        req.extensions_mut()
            .insert(ability_from_codes(uuid::Uuid::new_v4(), &["roles:read"]));
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
            .set_json(serde_json::json!({
                "name": "admin",
                "resource_type": null,
                "resource_id": null
            }))
            .to_request();
        req.extensions_mut()
            .insert(ability_from_codes(uuid::Uuid::new_v4(), &["roles:create"]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["name"], "admin");
    }
}
