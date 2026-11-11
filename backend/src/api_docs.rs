use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        // -- auth
        crate::controllers::auth_controller::register,
        crate::controllers::auth_controller::login,
        crate::controllers::auth_controller::refresh,
        crate::controllers::auth_controller::session,
        crate::controllers::auth_controller::session_trailing,
        crate::controllers::auth_controller::logout,
        crate::controllers::auth_controller::recover_password,
        crate::controllers::auth_controller::forgot_password,
        crate::controllers::auth_controller::reset_password,
        crate::controllers::auth_controller::setup_2fa,
        crate::controllers::auth_controller::enable_2fa,
        crate::controllers::auth_controller::disable_2fa,
        crate::controllers::auth_controller::change_password,
        crate::controllers::auth_controller::me,
        crate::controllers::auth_controller::confirm,
        // -- webhooks
        crate::controllers::webhooks_controller::stripe_webhook,
        crate::controllers::webhooks_controller::pix_webhook,
        // -- self service / profile
        crate::controllers::profile_controller::find_own_profile,
        crate::controllers::profile_controller::update_own_profile,
        // -- health / metrics
        crate::controllers::health_controller::liveness,
        crate::controllers::health_controller::health_check,
        crate::controllers::metrics_controller::metrics,
        // -- admin / permissions
        crate::controllers::permissions_controller::all_permissions,
        crate::controllers::permissions_controller::permission_catalog,
        crate::controllers::permissions_controller::create_permission,
        crate::controllers::permissions_controller::update_permission,
        crate::controllers::permissions_controller::destroy_permission,
        // -- admin / roles
        crate::controllers::roles_controller::all_roles,
        crate::controllers::roles_controller::find_role,
        crate::controllers::roles_controller::create_role,
        crate::controllers::roles_controller::update_role,
        crate::controllers::roles_controller::destroy_role,
        crate::controllers::roles_controller::find_role_permissions,
        crate::controllers::roles_controller::update_role_permissions,
        crate::controllers::roles_controller::find_role_members,
        // -- admin / users
        crate::controllers::users_controller::all_users,
        crate::controllers::users_controller::find_user,
        crate::controllers::users_controller::find_user_roles,
        crate::controllers::users_controller::update_user_roles,
        // -- admin / audit-logs
        crate::controllers::audit_logs_controller::list_audit_logs,
        crate::controllers::audit_logs_controller::get_audit_log,
        crate::controllers::audit_logs_controller::create_audit_log,
        // -- admin / upload
        crate::controllers::upload_controller::upload_file,
    ),
    components(
        schemas(
            // auth DTOs
            crate::controllers::auth_controller::RegisterRequest,
            crate::controllers::auth_controller::LoginRequest,
            crate::controllers::auth_controller::RecoverRequest,
            crate::controllers::auth_controller::ResetPasswordRequest,
            crate::controllers::auth_controller::Enable2FARequest,
            crate::controllers::auth_controller::ChangePasswordRequest,
            crate::controllers::auth_controller::AuthResponse,
            crate::controllers::auth_controller::SessionResponse,
            crate::controllers::auth_controller::UserInfo,
            crate::controllers::auth_controller::MeResponse,
            // health
            crate::controllers::health_controller::HealthResponse,
            crate::controllers::health_controller::HealthDependencyStatus,
            // permissions
            crate::controllers::permissions_controller::PermissionRequest,
            crate::controllers::permissions_controller::PermissionUpdateRequest,
            crate::controllers::permissions_controller::PermissionSearch,
            crate::models::permission::Permission,
            crate::models::permission::NewPermission,
            crate::models::permission::PermissionPatch,
            crate::models::permission::PermissionWithRoles,
            crate::models::permission::PermissionRole,
            crate::models::permission::PermissionCatalog,
            crate::models::permission::CatalogEntry,
            crate::utils::pagination::PaginatedResponse<crate::models::permission::PermissionWithRoles>,
            // roles
            crate::controllers::roles_controller::RoleWriteRequest,
            crate::controllers::roles_controller::RolePermissionsRequest,
            crate::controllers::roles_controller::RolePermissionLinksResponse,
            crate::controllers::roles_controller::UpdatedCountResponse,
            crate::models::role::Role,
            crate::models::permission::RolePermissionItem,
            crate::repositories::traits::roles_trait::RoleMemberItem,
            crate::models::role::NewRole,
            // users
            crate::controllers::users_controller::UserRolesRequest,
            crate::controllers::users_controller::UserRolesResponse,
            crate::controllers::users_controller::RoleSummary,
            crate::repositories::traits::users_trait::AdminUserLookupItem,
            crate::repositories::traits::users_trait::AdminUserItem,
            // audit logs
            crate::models::audit_log::AuditLog,
            crate::models::audit_log::NewAuditLog,
            // upload
            crate::controllers::upload_controller::UploadResponse,
            // self service profile
            crate::controllers::profile_controller::ProfileUpdateRequest,
            // pagination
            crate::utils::pagination::PaginationParams,
            crate::utils::pagination::PaginationMeta,
            crate::utils::pagination::PaginatedResponse<crate::models::role::Role>,
            crate::utils::pagination::PaginatedResponse<crate::repositories::traits::users_trait::AdminUserLookupItem>,
            crate::utils::pagination::PaginatedResponse<crate::models::audit_log::AuditLog>,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication & session endpoints"),
        (name = "webhooks", description = "External system webhooks (Stripe, Pix)"),
        (name = "health", description = "Service health probes"),
        (name = "metrics", description = "Prometheus-compatible operational metrics"),
        (name = "permissions", description = "Permission catalog management (admin)"),
        (name = "roles", description = "Role management (admin)"),
        (name = "users", description = "User management (admin)"),
        (name = "audit_logs", description = "Tamper-evident audit log (admin)"),
        (name = "upload", description = "Authenticated file uploads (admin)")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(utoipa::openapi::security::ApiKey::Header(
                    utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                )),
            );
        }
    }
}
