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
        crate::controllers::auth_controller::stripe_webhook,
        crate::controllers::auth_controller::pix_webhook,
        // -- health / metrics
        crate::controllers::health_controller::health_check,
        crate::controllers::metrics_controller::metrics,
        // -- admin / roles
        crate::controllers::roles_controller::list_roles,
        crate::controllers::roles_controller::get_role,
        crate::controllers::roles_controller::create_role,
        crate::controllers::roles_controller::update_role,
        crate::controllers::roles_controller::delete_role,
        // -- admin / users
        crate::controllers::users_controller::list_users,
        crate::controllers::users_controller::get_user,
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
            // health
            crate::controllers::health_controller::HealthResponse,
            crate::controllers::health_controller::HealthDependencyStatus,
            // roles
            crate::controllers::roles_controller::RoleWriteRequest,
            crate::models::role::Role,
            crate::models::role::NewRole,
            // users
            crate::repositories::traits::users_trait::AdminUserLookupItem,
            crate::repositories::traits::users_trait::AdminUserItem,
            // audit logs
            crate::models::audit_log::AuditLog,
            crate::models::audit_log::NewAuditLog,
            // upload
            crate::controllers::upload_controller::UploadResponse,
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
                SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}
