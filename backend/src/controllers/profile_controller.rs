//! Self service endpoints of the profile.
//!
//! The subject only ever touches its own record, so the authorization is an
//! instance check: the self scoped customer role passes and any other scope is
//! refused. A class level `profiles:read` also reads, but the write always
//! needs the write authority (class level or scoped).

use actix_web::{HttpResponse, get, patch, web};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, CurrentAbility};
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::models::profile::{Profile, ProfilePatch};
use crate::repositories::container::AppContainer;
use crate::utils::validation::first_validation_error_message;

/// The profile fields a subject may edit about itself.
///
/// The identity fields (the user id, the status) and the encrypted attributes
/// (cpf, phone, whatsapp) are absent on purpose: the first belong to the user,
/// the second need the key material of their own flow.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ProfileUpdateRequest {
    #[validate(length(max = 100, message = "profile.validation.first_name_invalid"))]
    pub first_name: Option<String>,
    #[validate(length(max = 100, message = "profile.validation.last_name_invalid"))]
    pub last_name: Option<String>,
    #[validate(length(max = 255, message = "profile.validation.full_name_invalid"))]
    pub full_name: Option<String>,
    #[validate(length(max = 50, message = "profile.validation.nickname_invalid"))]
    pub nickname: Option<String>,
    #[validate(length(max = 2000, message = "profile.validation.bio_invalid"))]
    pub bio: Option<String>,
    #[validate(length(max = 500, message = "profile.validation.avatar_invalid"))]
    pub avatar: Option<String>,
    pub birthday: Option<chrono::NaiveDate>,
}

/// Reads the profile of the authenticated subject.
///
/// Requires the `profiles:read` authority over the subject's own record.
#[utoipa::path(
    get,
    path = "/api/v1/profile",
    tag = "profile",
    responses(
        (status = 200, description = "The profile of the caller", body = Profile),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `profiles:read` authority"),
        (status = 404, description = "The caller has no profile")
    ),
    security(("bearer_auth" = []))
)]
#[get("/profile")]
pub async fn find_own_profile(
    user: AuthUser,
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
) -> AppResult<HttpResponse> {
    let subject_id = user.claims().sub;
    // The instance check: the scope of a self scoped role is the subject
    // itself, so the target of the check is the user id.
    ability.authorize_subject(AbilityAction::Read, AbilityResource::Profiles, subject_id)?;

    let profile = container
        .profiles
        .find_by_user_id(&subject_id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Profile".to_string()))?;

    Ok(HttpResponse::Ok().json(profile))
}

/// Updates the profile of the authenticated subject.
///
/// Requires the `profiles:update` authority over the subject's own record.
#[utoipa::path(
    patch,
    path = "/api/v1/profile",
    tag = "profile",
    request_body = ProfileUpdateRequest,
    responses(
        (status = 200, description = "The updated profile", body = Profile),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `profiles:update` authority"),
        (status = 404, description = "The caller has no profile")
    ),
    security(("bearer_auth" = []))
)]
#[patch("/profile")]
pub async fn update_own_profile(
    user: AuthUser,
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    body: web::Json<ProfileUpdateRequest>,
) -> AppResult<HttpResponse> {
    let subject_id = user.claims().sub;
    ability.authorize_subject(AbilityAction::Update, AbilityResource::Profiles, subject_id)?;

    let payload = body.into_inner();
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    let profile = container
        .profiles
        .find_by_user_id(&subject_id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Profile".to_string()))?;

    let updated = container
        .profiles
        .patch(
            &profile.id,
            &ProfilePatch {
                first_name: payload.first_name,
                last_name: payload.last_name,
                full_name: payload.full_name,
                nickname: payload.nickname,
                bio: payload.bio,
                avatar: payload.avatar,
                birthday: payload.birthday,
            },
        )
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(updated))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(find_own_profile).service(update_own_profile);
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use uuid::Uuid;

    use actix_web::HttpMessage;

    use super::config;
    use crate::authz::test_helpers::ability_from_codes;
    use crate::middleware::auth::Claims;
    use crate::repositories::mocks::mock_container;

    #[actix_web::test]
    async fn find_own_profile_returns_forbidden_without_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(web::scope("/api/v1").configure(config)),
        )
        .await;

        // The handler reads the subject of the request from the claims the
        // `JwtAuth` middleware inserts, so the unit test inserts them too.
        let subject = Uuid::new_v4();
        let req = test::TestRequest::get().uri("/api/v1/profile").to_request();
        req.extensions_mut().insert(Claims {
            sub: subject,
            profile_id: Uuid::new_v4(),
            role: 3,
            token_use: "access".to_string(),
            exp: 0,
            iat: 0,
            nbf: 0,
            jti: None,
        });
        req.extensions_mut()
            .insert(ability_from_codes(subject, &[]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
