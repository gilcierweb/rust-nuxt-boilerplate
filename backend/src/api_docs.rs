use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::auth_controller::register,
        crate::controllers::auth_controller::login,
    ),
    components(
        schemas(
            crate::controllers::auth_controller::RegisterRequest,
            crate::controllers::auth_controller::LoginRequest,
            crate::controllers::auth_controller::AuthResponse,
            crate::controllers::auth_controller::UserInfo,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication Endpoints")
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
            )
        }
    }
}
