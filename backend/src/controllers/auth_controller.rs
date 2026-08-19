use actix_web::cookie::time::Duration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::Utc;
use diesel::result::Error as DieselError;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::config::AppConfig;
use crate::db::schema::users as users_table;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{AuthUser, create_token_with_kid};
use crate::models::magic_link_token::NewMagicLinkToken;
use crate::models::profile::NewProfile;
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::models::role::{ROLE_ADMIN, ROLE_OPERATOR, ROLE_VIEWER};
use crate::models::user::{NewUser, User};
use crate::repositories::container::AppContainer;
use crate::repositories::traits::users_trait::IUserRepositoryTransaction;
use crate::security::SecurityService;
use crate::services::auth_service::{
    hash_password, needs_rehash, rehash_password, validate_password_strength, verify_password,
};
use crate::services::token_service::hash_token;
use crate::utils::ip::extract_client_ip;
use crate::utils::validation::first_validation_error_message;

// -- Request/Response DTOs

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    // RFC 5321 §4.5.3 limits the full forward-path to 256 chars; 254 is the
    // practical max for the email address itself.
    #[validate(
        email(message = "auth.validation.invalid_email"),
        length(max = 254, message = "auth.validation.email_too_long")
    )]
    pub email: String,
    // min=12: fast-fail for obviously short inputs before the full strength
    // check. max=128: prevents multi-MB strings from reaching Argon2id.
    #[validate(length(
        min = 12,
        max = 128,
        message = "auth.validation.password_invalid_length"
    ))]
    pub password: String,
    // password_confirmation is compared in the handler; cap at same max to
    // avoid asymmetric allocation.
    #[validate(length(max = 128, message = "auth.validation.password_invalid_length"))]
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(
        email(message = "auth.validation.invalid_email"),
        length(max = 254, message = "auth.validation.email_too_long")
    )]
    pub email: String,
    // No min check here — wrong passwords should still hit the DB lookup so
    // timing is consistent. max=128 prevents large payloads reaching Argon2id.
    #[validate(length(max = 128, message = "auth.validation.password_invalid_length"))]
    pub password: String,
    /// Optional TOTP code if 2FA is enabled (exactly 6 decimal digits)
    #[validate(length(min = 6, max = 6, message = "auth.validation.otp_invalid"))]
    pub otp_code: Option<String>,
    /// Optional backup code if 2FA is enabled (alphanumeric, 8 chars)
    #[validate(length(min = 8, max = 8, message = "auth.validation.backup_code_invalid"))]
    pub backup_code: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RecoverRequest {
    #[validate(
        email(message = "auth.validation.invalid_email"),
        length(max = 254, message = "auth.validation.email_too_long")
    )]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    // Reset tokens are random hex strings; cap to avoid unbounded allocations.
    #[validate(length(min = 1, max = 512, message = "auth.validation.token_invalid"))]
    pub token: String,
    // min=12: fast-fail. max=128: prevents large payloads reaching Argon2id.
    #[validate(length(
        min = 12,
        max = 128,
        message = "auth.validation.password_invalid_length"
    ))]
    pub password: String,
    #[validate(length(max = 128, message = "auth.validation.password_invalid_length"))]
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Enable2FARequest {
    /// TOTP codes are exactly 6 decimal digits.
    #[validate(length(min = 6, max = 6, message = "auth.validation.otp_invalid"))]
    pub otp_code: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Setup2FARequest {
    /// Optional TOTP code for step-up verification when 2FA is already enabled.
    /// Required if 2FA is currently enabled to prevent unauthorized secret rotation.
    #[validate(length(min = 6, max = 6, message = "auth.validation.otp_invalid"))]
    pub otp_code: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    // Current password: cap at 128 to prevent large payloads reaching Argon2id.
    // No min check — wrong passwords should still reach the verify step so
    // timing stays consistent.
    #[validate(length(max = 128, message = "auth.validation.password_invalid_length"))]
    pub current_password: String,
    // Align min with the password strength policy (12 chars) and cap at 128.
    #[validate(length(
        min = 12,
        max = 128,
        message = "auth.validation.password_invalid_length"
    ))]
    pub new_password: String,
    #[validate(length(max = 128, message = "auth.validation.password_invalid_length"))]
    pub password_confirmation: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub profile_id: Uuid,
    pub is_otp_enabled: bool,
    pub roles: Vec<String>,
}

// POST /api/v1/auth/register
/// Register a new user account.
///
/// Creates the user, the linked profile, and dispatches a confirmation email.
/// The endpoint is intentionally uniform: it always returns `201` for valid
/// input even if the email already exists (the duplicate path returns `409`).
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists"),
        (status = 429, description = "Too many requests")
    )
)]
#[post("/register")]
pub async fn register(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    // Get locale from request for localized emails/errors
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);

    if body.password != body.password_confirmation {
        return Err(AppError::Validation(
            t!("auth.password.mismatch", locale = locale).into_owned(),
        ));
    }

    validate_password_strength(&body.password, &locale)?;
    let encrypted_password = hash_password(&body.password, container.config.as_ref())?;
    let now = Utc::now();
    let confirmation_token = Uuid::new_v4().to_string();
    let confirmation_token_digest = hash_token(
        &confirmation_token,
        &container.config.refresh_token_hash_salt,
    );
    let security = SecurityService::from_config(container.config.as_ref())?;
    let protected_email = security.protect_email(&body.email)?;
    let email_fingerprint = fingerprint_value(&protected_email.blind_index);

    let new_user = NewUser {
        id: Uuid::new_v4(),
        email_blind_index: protected_email.blind_index,
        email_encrypted: protected_email.encrypted,
        encrypted_password,
        confirmation_token_digest: Some(confirmation_token_digest),
        unconfirmed_email_blind_index: None,
        unconfirmed_email_encrypted: None,
        encryption_key_version: protected_email.key_version,
        created_at: now,
        updated_at: now,
    };

    let user: User = match container.users.create(&new_user).await {
        Ok(u) => u,
        Err(DieselError::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            tracing::warn!(
                event = "auth.register.duplicate_email",
                email_fingerprint = %email_fingerprint,
                "register blocked by duplicate email"
            );
            return Err(AppError::Conflict(
                t!("auth.register.email_exists").into_owned(),
            ));
        },
        Err(e) => return Err(AppError::Database(e)),
    };

    // Create profile for the user
    let new_profile = NewProfile::for_user(user.id, protected_email.key_version);
    container
        .profiles
        .create(&new_profile)
        .await
        .map_err(AppError::Database)?;

    let email_service = container.email_service.clone();
    // Get locale from request for localized emails
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);
    if let Err(error) = email_service
        .send_confirmation_email(&body.email, &confirmation_token, &locale)
        .await
    {
        tracing::warn!("confirmation email delivery skipped or failed: {}", error);
    }

    tracing::info!(
        event = "auth.register.success",
        user_id = %user.id,
        email_fingerprint = %email_fingerprint,
        "user registered"
    );

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": t!("auth.register.success"),
        "user_id": user.id,
    })))
}

// GET /api/v1/auth/confirm?token=xxx
/// Confirm a user email using the token sent during registration.
///
/// The token is opaque to the client; it is stored hashed server-side.
#[utoipa::path(
    get,
    path = "/api/v1/auth/confirm",
    tag = "auth",
    params(
        ("token" = String, Query, description = "Confirmation token delivered by email")
    ),
    responses(
        (status = 200, description = "Email confirmed"),
        (status = 400, description = "Invalid or missing token")
    )
)]
#[get("/confirm")]
pub async fn confirm(
    container: web::Data<AppContainer>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> AppResult<HttpResponse> {
    let token = query
        .get("token")
        .ok_or_else(|| AppError::BadRequest(t!("auth.reset.token_invalid").into_owned()))?;

    // Find user by confirmation token and confirm email
    let affected_rows = container
        .users
        .confirm_email(&hash_token(
            token,
            &container.config.refresh_token_hash_salt,
        ))
        .await
        .map_err(AppError::Database)?;

    if affected_rows == 0 {
        tracing::warn!(
            event = "auth.confirm.invalid_token",
            "email confirmation failed"
        );
        return Err(AppError::BadRequest(
            t!("auth.reset.token_invalid").into_owned(),
        ));
    }

    tracing::info!(event = "auth.confirm.success", "email confirmed");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.register.email_confirmed")
    })))
}

/// Get user roles with Redis caching to avoid N+1 queries
/// Cache TTL: 10 minutes (roles are low-churn; invalidation is explicit via
/// `invalidate_user_roles_cache` and `invalidate_role_cache` on role changes)
async fn get_cached_user_roles(container: &AppContainer, user_id: &Uuid) -> AppResult<Vec<String>> {
    const ROLE_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(600);

    let cache_key = format!("user_roles:{}", user_id);

    // Try to get from cache first
    if let Some(cached_roles) = container.cache.get::<Vec<String>>(&cache_key).await {
        return Ok(cached_roles);
    }

    // Cache miss - fetch from database
    let roles = container
        .users
        .get_user_roles(user_id)
        .await
        .map_err(AppError::Database)?;

    // Cache for 10 minutes (explicit invalidation on role changes)
    let _ = container
        .cache
        .set_with_ttl(&cache_key, &roles, ROLE_CACHE_TTL)
        .await;

    Ok(roles)
}

/// Invalidate cached user roles (call when roles change for a specific user)
pub async fn invalidate_user_roles_cache(container: &AppContainer, user_id: &Uuid) {
    let cache_key = format!("user_roles:{}", user_id);
    let _ = container.cache.delete(&cache_key).await;
}

/// Invalidate role cache for all users assigned to a given role.
///
/// Called from role CRUD endpoints (create, update, delete) to ensure
/// stale role data does not persist beyond the 10-minute TTL window.
pub async fn invalidate_role_cache(container: &AppContainer, role_id: &Uuid) {
    if let Ok(user_roles) = container.user_roles.find_by_role(role_id).await {
        for ur in user_roles {
            invalidate_user_roles_cache(container, &ur.user_id).await;
        }
    }
}

// POST /api/v1/auth/login
/// Authenticate an existing user and issue an access token.
///
/// When 2FA is enabled, the caller must also supply `otp_code`; otherwise the
/// request is rejected with `403 Forbidden`. Repeated failures may trigger a
/// temporary account lockout (`423 Locked`).
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized - Invalid credentials"),
        (status = 403, description = "Forbidden - OTP required"),
        (status = 423, description = "Locked - Temporary lockout due to failed attempts"),
        (status = 429, description = "Too many requests")
    )
)]
#[post("/login")]
pub async fn login(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let security = SecurityService::from_config(container.config.as_ref())?;
    let normalized_email = security.normalize_email(&body.email);
    let email_lookup = security.protect_email(&normalized_email)?;
    let request_ip = extract_client_ip(&req, &container.config.trusted_proxies);
    let user_agent = request_user_agent(&req);

    // Find user by email
    let user: User = match container
        .users
        .find_by_email(&email_lookup.blind_index)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            let users_total = container.users.count().await.unwrap_or(-1).to_string();
            let key_version = container.config.current_encryption_key_version;
            let key_fingerprint = fingerprint_value(container.config.blind_index_key.as_bytes());
            tracing::warn!(
                event = "auth.login.user_not_found",
                email_fingerprint = %fingerprint_value(&email_lookup.blind_index),
                key_fingerprint = %key_fingerprint,
                key_version = key_version,
                users_total = %users_total,
                ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
                user_agent = user_agent.as_deref().unwrap_or("unknown"),
                "login failed: user not found"
            );
            return Err(AppError::Unauthorized(t!("auth.login.failed").into_owned()));
        },
        Err(e) => return Err(AppError::Database(e)),
    };

    // Check if account is locked
    if user.is_locked(container.config.account_lockout_duration_secs) {
        tracing::warn!(
            event = "auth.login.blocked_locked",
            user_id = %user.id,
            ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
            user_agent = user_agent.as_deref().unwrap_or("unknown"),
            "login blocked: account locked"
        );
        return Err(AppError::BadRequest(t!("auth.login.locked").into_owned()));
    }

    // Check email confirmation
    if !user.is_confirmed() {
        tracing::warn!(
            event = "auth.login.blocked_unconfirmed",
            user_id = %user.id,
            ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
            "login blocked: email not confirmed"
        );
        return Err(AppError::BadRequest(
            t!("auth.login.email_not_confirmed").into_owned(),
        ));
    }

    // Verify password
    let password_valid = verify_password(&body.password, &user.encrypted_password)?;
    if !password_valid {
        container
            .users
            .record_failed_login(&user.id, 10, &container.config.refresh_token_hash_salt)
            .await
            .map_err(AppError::Database)?;
        tracing::warn!(
            event = "auth.login.invalid_password",
            user_id = %user.id,
            ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
            user_agent = user_agent.as_deref().unwrap_or("unknown"),
            "login failed: invalid password"
        );
        return Err(AppError::Unauthorized(t!("auth.login.failed").into_owned()));
    }
    if user.is_otp_enabled() {
        // Check for backup code first (one-time use)
        if let Some(backup_code) = &body.backup_code {
            let backup_codes = user.otp_backup_codes.as_ref().ok_or_else(|| {
                AppError::Internal(t!("auth.2fa.invalid_backup_codes").into_owned())
            })?;

            // Find and verify backup code (constant-time comparison)
            let mut backup_code_valid = false;
            let mut used_index: Option<usize> = None;

            for (index, stored_code_opt) in backup_codes.iter().enumerate() {
                if let Some(stored_code) = stored_code_opt
                    && verify_password(backup_code, stored_code)?
                {
                    backup_code_valid = true;
                    used_index = Some(index);
                    break;
                }
            }

            if !backup_code_valid {
                tracing::warn!(
                    event = "auth.login.invalid_backup_code",
                    user_id = %user.id,
                    ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
                    user_agent = user_agent.as_deref().unwrap_or("unknown"),
                    "login failed: invalid backup code"
                );
                return Err(AppError::Unauthorized(t!("auth.login.failed").into_owned()));
            }

            // Remove used backup code (one-time use)
            if let Some(index) = used_index {
                let mut updated_codes = backup_codes.clone();
                updated_codes[index] = None;
                container
                    .users
                    .update_backup_codes(&user.id, &updated_codes)
                    .await
                    .map_err(AppError::Database)?;
            }
        } else {
            // Standard TOTP verification
            match &body.otp_code {
                None => {
                    tracing::info!(
                        event = "auth.login.otp_required",
                        user_id = %user.id,
                        ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
                        "login requires otp"
                    );
                    return Ok(HttpResponse::Ok().json(serde_json::json!({
                        "requires_otp": true,
                        "message": t!("auth.2fa.setup_required")
                    })));
                },
                Some(code) => {
                    let secret = user.otp_secret.as_ref().ok_or(AppError::Internal(
                        t!("auth.2fa.invalid_secret").into_owned(),
                    ))?;
                    if let Err(error) = verify_totp(secret, code) {
                        tracing::warn!(
                            event = "auth.login.invalid_otp",
                            user_id = %user.id,
                            ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
                            user_agent = user_agent.as_deref().unwrap_or("unknown"),
                            "login failed: invalid otp"
                        );
                        return Err(error);
                    }
                },
            }
        }
    }

    // Get profile ID
    let profile = container
        .profiles
        .find_by_user_id(&user.id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Internal(
            t!("users.profile_not_found").into_owned(),
        ))?;
    // Fetch user roles BEFORE generating the token.
    // This is NOT redundant — the `role_claim` (primary role as i32) is embedded
    // into the JWT and used by `Claims::has_role()` + `build_ability` for fine-grained
    // authorization on every authenticated request. Without this call, the issued
    // token would have a default/zero `role` and downstream authorization would fail.
    // Cached for 10 min (see P3); explicit invalidation on role changes.
    let roles = get_cached_user_roles(&container, &user.id).await?;
    let role_claim = primary_role_claim(&roles);

    // Generate tokens
    #[allow(clippy::get_first)]
    let active_kid = container.config.jwt_secrets.get(0).map(|k| k.kid.as_str());
    let access_token = create_token_with_kid(
        user.id,
        profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
        "access",
        active_kid,
    )?;

    let refresh_token_plain = generate_random_token(48);
    let refresh_token_hash = hash_token(
        &refresh_token_plain,
        &container.config.refresh_token_hash_salt,
    );

    // Store refresh token
    let ip_string = req.peer_addr().map(|addr| addr.ip().to_string());

    // Parse IP for IpNet type (used in DB). Since this comes from trusted peer_addr,
    // parse should always succeed, but log a warning if it doesn't.
    let ip: Option<ipnet::IpNet> = ip_string.as_ref().and_then(|s| {
        s.parse::<ipnet::IpNet>()
            .inspect_err(|e| {
                tracing::warn!(
                    event = "auth.ip_parse_error",
                    ip = %s,
                    error = %e,
                    "Failed to parse trusted peer IP as IpNet"
                );
            })
            .ok()
    });

    // Upgrade password hash if using outdated Argon2 parameters - use transaction for atomicity
    if needs_rehash(&user.encrypted_password, container.config.as_ref()) {
        let new_hash = match rehash_password(&body.password, container.config.as_ref()) {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(
                    event = "auth.login.rehash_error",
                    user_id = %user.id,
                    error = %e,
                    "password rehash error"
                );
                // Non-fatal: continue with old hash
                user.encrypted_password.clone()
            },
        };
        let user_id = user.id;
        let ip_clone = ip;
        container
            .users_tx
            .run_transaction(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::users::dsl::*;
                    // Update password
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            encrypted_password.eq(&new_hash),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await?;
                    // Record successful login
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            failed_attempts.eq(0),
                            locked_at.eq::<Option<chrono::NaiveDateTime>>(None),
                            current_sign_in_at.eq(Some(chrono::Utc::now().naive_utc())),
                            last_sign_in_at.eq(diesel::dsl::sql::<
                                diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>,
                            >("current_sign_in_at")),
                            current_sign_in_ip.eq(ip_clone),
                            sign_in_count.eq(diesel::dsl::sql::<diesel::sql_types::Integer>(
                                "sign_in_count + 1",
                            )),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await?;
                    Ok::<_, diesel::result::Error>(())
                })
            })
            .await
            .map_err(AppError::Database)?;
    } else {
        // No rehash needed, just record successful login
        container
            .users
            .record_successful_login(&user.id, ip)
            .await
            .map_err(AppError::Database)?;
    }

    let new_refresh = NewRefreshToken {
        id: Uuid::new_v4(),
        user_id: user.id,
        token_hash: refresh_token_hash,
        device_info: req
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        ip_address: ip_string.clone(),
        expires_at: Utc::now()
            + chrono::Duration::seconds(container.config.jwt_refresh_expiry_secs),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    container
        .refresh_tokens
        .create(&new_refresh)
        .await
        .map_err(AppError::Database)?;

    tracing::info!(
        event = "auth.login.success",
        user_id = %user.id,
        ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
        user_agent = user_agent.as_deref().unwrap_or("unknown"),
        "login success"
    );

    let mut response = HttpResponse::Ok();
    response.cookie(build_refresh_cookie(
        container.config.as_ref(),
        &refresh_token_plain,
    ));
    response.cookie(clear_legacy_refresh_cookie(container.config.as_ref()));

    Ok(response.json(AuthResponse {
        access_token,
        token_type: "Bearer",
        expires_in: container.config.jwt_access_expiry_secs,
        user: build_user_info(container.config.as_ref(), &user, profile.id, roles)?,
    }))
}

// POST /api/v1/auth/refresh
/// Rotate the refresh token cookie and return a fresh access token.
///
/// The endpoint reads the `refresh_token` cookie, validates the digest,
/// revokes the presented token, issues a new one (cookie rotation), and
/// returns a brand-new access token. Returns `401` if no valid refresh
/// token is present.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "auth",
    responses(
        (
            status = 200,
            description = "New access token issued; refresh cookie rotated",
            body = AuthResponse,
            headers(
                ("Set-Cookie" = String, description = "Rotated `refresh_token` cookie")
            )
        ),
        (status = 401, description = "Missing or invalid refresh token")
    )
)]
#[post("/refresh")]
pub async fn refresh(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let refresh_tokens = extract_refresh_cookies(&req);
    if refresh_tokens.is_empty() {
        tracing::warn!(
            event = "auth.refresh.invalid_token",
            ip = extract_client_ip(&req, &container.config.trusted_proxies)
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            user_agent = request_user_agent(&req).as_deref().unwrap_or("unknown"),
            "refresh failed: missing refresh token cookie"
        );
        return Err(AppError::Unauthorized(
            t!("auth.missing_token").into_owned(),
        ));
    }

    let mut rotated_result = None;

    // Try each refresh token cookie until we find one that can be rotated
    // rotate_token atomically validates the token (exists, not revoked, not expired),
    // revokes it, and creates a new token preserving device_info and ip_address
    for refresh_token in refresh_tokens {
        // Try to atomically rotate the token using plaintext verification
        // rotate_token verifies the plaintext token against stored Argon2id hashes,
        // immediately revokes the old one, generates a new token, hashes it,
        // stores it, and returns both the stored RefreshToken and new plain token
        match container
            .refresh_tokens
            .rotate_token(
                &refresh_token,
                container.config.jwt_refresh_expiry_secs,
                &container.config.refresh_token_hash_salt,
            )
            .await
        {
            Ok(Some((rotated, new_plain))) => {
                rotated_result = Some((rotated, new_plain));
                break;
            },
            Ok(None) => continue, // Token was already revoked/expired or not found
            Err(e) => return Err(AppError::Database(e)),
        }
    }

    let (rotated, new_refresh_plain) = match rotated_result {
        Some(rotated) => rotated,
        None => {
            tracing::warn!(
                event = "auth.refresh.invalid_token",
                ip = extract_client_ip(&req, &container.config.trusted_proxies)
                    .map(|ip| ip.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                user_agent = request_user_agent(&req).as_deref().unwrap_or("unknown"),
                "refresh failed: invalid or missing token"
            );
            return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
        },
    };

    // Get user
    let user = container
        .users
        .find(&rotated.user_id)
        .await
        .map_err(AppError::Database)?;

    // Get profile
    let _profile = container
        .profiles
        .find_by_user_id(&user.id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Internal(
            t!("users.profile_not_found").into_owned(),
        ))?;
    // Required for the new access token's role_claim (see login note above).
    let roles = get_cached_user_roles(&container, &user.id).await?;
    let role_claim = primary_role_claim(&roles);

    // Generate new access token
    #[allow(clippy::get_first)]
    let active_kid = container.config.jwt_secrets.get(0).map(|k| k.kid.as_str());
    let access_token = create_token_with_kid(
        user.id,
        _profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
        "access",
        active_kid,
    )?;

    tracing::info!(
        event = "auth.refresh.success",
        user_id = %user.id,
        "access token refreshed"
    );

    let mut response = HttpResponse::Ok();
    response.cookie(build_refresh_cookie(
        container.config.as_ref(),
        &new_refresh_plain,
    ));
    response.cookie(clear_legacy_refresh_cookie(container.config.as_ref()));

    Ok(response.json(serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": container.config.jwt_access_expiry_secs,
    })))
}

async fn session_impl(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let (_, stored) = find_valid_refresh_token(container.as_ref(), &req).await?;

    let user = container
        .users
        .find(&stored.user_id)
        .await
        .map_err(AppError::Database)?;

    let profile = container
        .profiles
        .find_by_user_id(&user.id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Internal(
            t!("users.profile_not_found").into_owned(),
        ))?;

    // Required for the new access token's role_claim (see login note above).
    let roles = get_cached_user_roles(&container, &user.id).await?;
    let role_claim = primary_role_claim(&roles);
    #[allow(clippy::get_first)]
    let active_kid = container.config.jwt_secrets.get(0).map(|k| k.kid.as_str());
    let access_token = create_token_with_kid(
        user.id,
        profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
        "access",
        active_kid,
    )?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(SessionResponse {
            access_token,
            token_type: "Bearer",
            expires_in: container.config.jwt_access_expiry_secs,
            user: build_user_info(container.config.as_ref(), &user, profile.id, roles)?,
        }))
}

// GET /api/v1/auth/session
/// Return the current session: validates the refresh token cookie and returns
/// a fresh access token alongside the authenticated user profile.
#[utoipa::path(
    get,
    path = "/api/v1/auth/session",
    tag = "auth",
    responses(
        (status = 200, description = "Active session", body = SessionResponse),
        (status = 401, description = "Missing or invalid refresh token")
    )
)]
#[get("/session")]
pub async fn session(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    session_impl(container, req).await
}

// GET /api/v1/auth/session/
/// Trailing-slash alias for `/api/v1/auth/session`.
#[utoipa::path(
    get,
    path = "/api/v1/auth/session/",
    tag = "auth",
    responses(
        (status = 200, description = "Active session", body = SessionResponse),
        (status = 401, description = "Missing or invalid refresh token")
    )
)]
#[get("/session/")]
pub async fn session_trailing(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    session_impl(container, req).await
}

// POST /api/v1/auth/logout
/// Log the user out by blacklisting the bearer access token (if present)
/// and revoking every refresh token cookie the request carried.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logged out; refresh cookie cleared"),
        (status = 401, description = "Authentication required")
    ),
    security(("bearer_auth" = []))
)]
#[post("/logout")]
pub async fn logout(
    req: HttpRequest,
    container: web::Data<AppContainer>,
) -> AppResult<HttpResponse> {
    // Extract access token from Authorization header for blacklisting
    let access_token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_string());

    // Blacklist the access token if present
    if let Some(token) = &access_token {
        let token_hash =
            crate::repositories::access_token_blacklist::hash_token_for_blacklist(token);
        let ttl = container.config.jwt_access_expiry_secs as u64;
        if let Err(e) = container.access_token_blacklist.add(&token_hash, ttl).await {
            tracing::warn!("Failed to blacklist access token: {}", e);
        }
    }

    let mut revoked_count: usize = 0;
    for refresh_token in extract_refresh_cookies(&req) {
        if let Ok(Some(token)) = container
            .refresh_tokens
            .find_valid_by_token(&refresh_token, &container.config.refresh_token_hash_salt)
            .await
        {
            container
                .refresh_tokens
                .revoke(&token.id)
                .await
                .map_err(AppError::Database)?;
            revoked_count += 1;
        }
    }

    let mut response = HttpResponse::Ok();
    response.cookie(clear_refresh_cookie(container.config.as_ref()));
    response.cookie(clear_legacy_refresh_cookie(container.config.as_ref()));

    tracing::info!(
        event = "auth.logout.success",
        revoked_tokens = revoked_count,
        ip = extract_client_ip(&req, &container.config.trusted_proxies)
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        "logout success"
    );

    Ok(response.json(serde_json::json!({
        "message": t!("auth.logout.success")
    })))
}

// POST /api/v1/auth/recover
/// Initiate the password recovery flow.
///
/// Always returns `200 OK` with a generic success message to prevent email
/// enumeration; the reset email is sent only when the address matches a real
/// account.
#[utoipa::path(
    post,
    path = "/api/v1/auth/recover",
    tag = "auth",
    request_body = RecoverRequest,
    responses(
        (status = 200, description = "Recovery email accepted for processing"),
        (status = 400, description = "Validation error"),
        (status = 429, description = "Too many requests")
    )
)]
#[post("/recover")]
pub async fn recover_password(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<RecoverRequest>,
) -> AppResult<HttpResponse> {
    recover_password_handler(req, container, body).await
}

// POST /api/v1/auth/forgot-password (semantic alias for /auth/recover)
/// Semantic alias for `/api/v1/auth/recover`.
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "auth",
    request_body = RecoverRequest,
    responses(
        (status = 200, description = "Recovery email accepted for processing"),
        (status = 400, description = "Validation error"),
        (status = 429, description = "Too many requests")
    )
)]
#[post("/forgot-password")]
pub async fn forgot_password(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<RecoverRequest>,
) -> AppResult<HttpResponse> {
    recover_password_handler(req, container, body).await
}

async fn recover_password_handler(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<RecoverRequest>,
) -> AppResult<HttpResponse> {
    // Get locale from request for localized emails
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);

    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let security = SecurityService::from_config(container.config.as_ref())?;
    let normalized_email = security.normalize_email(&body.email);
    let email_lookup = security.protect_email(&normalized_email)?;

    // Always execute the same code path to prevent email enumeration attacks
    // Generate token and attempt DB update regardless of user existence
    let token = Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();
    let token_digest = hash_token(&token, &container.config.refresh_token_hash_salt);

    // Attempt to find user and create reset token
    // If user doesn't exist, this will silently fail (0 rows updated)
    if let Ok(Some(user)) = container
        .users
        .find_by_email(&email_lookup.blind_index)
        .await
    {
        let _ = container
            .users
            .create_password_reset_token(&user.id, &token_digest, now)
            .await;
    }

    // Always attempt to send email (will fail silently for non-existent users)
    // This ensures the same timing regardless of user existence
    let email_service = container.email_service.clone();
    if let Err(error) = email_service
        .send_password_reset_email(&body.email, &token, &locale)
        .await
    {
        tracing::debug!("password reset email delivery skipped or failed: {}", error);
    }

    tracing::info!(
        event = "auth.recover.requested",
        email_fingerprint = %fingerprint_value(&email_lookup.blind_index),
        "password recovery requested"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.recover.email_sent")
    })))
}

// POST /api/v1/auth/reset
/// Complete the password recovery flow: exchange the reset token (sent via
/// email) for a new password. All existing refresh tokens are revoked on
/// success and a password-changed notification is dispatched.
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password updated"),
        (status = 400, description = "Invalid token, expired token, or validation error")
    )
)]
#[post("/reset")]
pub async fn reset_password(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<ResetPasswordRequest>,
) -> AppResult<HttpResponse> {
    // Get locale from request for localized errors
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);

    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    if body.password != body.password_confirmation {
        return Err(AppError::Validation(
            t!("auth.password.mismatch", locale = locale).into_owned(),
        ));
    }

    let token_digest = hash_token(&body.token, &container.config.refresh_token_hash_salt);

    // Perform token lookup and validation in constant time to prevent
    // timing attacks that could distinguish between invalid/expired/valid tokens.
    // All paths execute the same database queries and checks.
    let user_opt = container
        .users
        .find_by_reset_token_digest(&token_digest)
        .await
        .map_err(AppError::Database)?;

    let now = Utc::now();
    let mut valid = false;
    let mut user_id: Option<uuid::Uuid> = None;

    if let Some(user) = user_opt
        && let Some(sent_at) = user.reset_password_sent_at
    {
        // Token exists and has a sent_at timestamp - check expiry
        let duration = now.signed_duration_since(sent_at);
        // Use constant-time comparison: always compute, don't branch on result
        let is_expired = duration > chrono::Duration::hours(2);
        // Store user_id only if not expired (but still compute duration)
        if !is_expired {
            valid = true;
            user_id = Some(user.id);
        }
    }

    // Always log at the same level to avoid timing leaks through log volume
    if !valid {
        tracing::warn!(
            event = "auth.reset.invalid_or_expired_token",
            "password reset failed: token invalid or expired"
        );
        return Err(AppError::BadRequest(
            t!("auth.reset.token_invalid").into_owned(),
        ));
    }

    // At this point, we have a valid user_id
    let user_id = user_id.expect("valid reset should have user_id");

    validate_password_strength(&body.password, &locale)?;
    let hashed_password = hash_password(&body.password, container.config.as_ref())?;
    let affected_rows = container
        .users
        .update_password(&user_id, &hashed_password)
        .await
        .map_err(AppError::Database)?;

    if affected_rows == 0 {
        tracing::warn!(
            event = "auth.reset.invalid_token_rows",
            user_id = %user_id,
            "password reset failed: no rows updated"
        );
        return Err(AppError::BadRequest(
            t!("auth.reset.token_invalid").into_owned(),
        ));
    }

    let revoked_tokens = container
        .refresh_tokens
        .revoke_all_for_user(&user_id)
        .await
        .map_err(AppError::Database)?;

    // Send password reset confirmation email
    let security = SecurityService::from_config(container.config.as_ref())?;
    let user = container
        .users
        .find(&user_id)
        .await
        .map_err(AppError::Database)?;
    let email = security.decrypt_user_email(&user)?;
    // Get locale from request for localized emails
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);
    let email_service = container.email_service.clone();
    if let Err(error) = email_service
        .send_password_changed_notification(&email, &locale)
        .await
    {
        tracing::warn!("password reset confirmation email failed: {}", error);
    }

    tracing::info!(
        event = "auth.reset.success",
        user_id = %user_id,
        revoked_tokens,
        "password reset success"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.reset.success")
    })))
}

// POST /api/v1/auth/2fa/setup
/// Initialise TOTP two-factor authentication for the current user.
///
/// Generates a fresh secret and a corresponding `otpauth://` URL suitable for
/// any standard authenticator app. The secret is stored but 2FA is **not**
/// enabled until `POST /2fa/enable` succeeds with a valid TOTP code.
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/setup",
    tag = "auth",
    request_body = Setup2FARequest,
    responses(
        (status = 200, description = "TOTP secret and provisioning URL issued"),
        (status = 400, description = "2FA already enabled; step-up verification required"),
        (status = 401, description = "Authentication required")
    ),
    security(("bearer_auth" = []))
)]
#[post("/2fa/setup")]
pub async fn setup_2fa(
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<Setup2FARequest>,
) -> AppResult<HttpResponse> {
    use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

    let user_data = container
        .users
        .find(&user.claims().sub)
        .await
        .map_err(AppError::Database)?;

    // If 2FA is already enabled, require step-up verification (current TOTP code)
    if user_data.is_otp_enabled() {
        let otp_code = body
            .otp_code
            .as_ref()
            .ok_or_else(|| AppError::BadRequest(t!("auth.2fa.step_up_required").into_owned()))?;

        let secret = user_data
            .otp_secret
            .as_ref()
            .ok_or_else(|| AppError::Internal(t!("auth.2fa.invalid_secret").into_owned()))?;

        verify_totp(secret, otp_code)?;
    }

    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();
    let security = SecurityService::from_config(container.config.as_ref())?;
    let current_email = security.decrypt_user_email(&user_data)?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA256,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some(container.config.totp_issuer.clone()),
        current_email,
    )
    .map_err(|e| AppError::Internal(format!("TOTP error: {}", e)))?;

    let qr_code_url = totp.get_url();

    // Store secret temporarily (not enabled until verified via /2fa/enable)
    container
        .users
        .set_otp_secret(&user.claims().sub, &secret_base32)
        .await
        .map_err(AppError::Database)?;

    tracing::info!(
        event = "auth.2fa.setup",
        user_id = %user.claims().sub,
        "2fa setup initialized"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "secret": secret_base32,
        "qr_code_url": qr_code_url,
        "message": t!("auth.2fa.setup_instructions")
    })))
}

// POST /api/v1/auth/2fa/enable
/// Confirm a TOTP code and enable 2FA for the current user.
///
/// Returns a set of one-time backup codes. All refresh tokens for the user
/// are revoked on success.
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/enable",
    tag = "auth",
    request_body = Enable2FARequest,
    responses(
        (status = 200, description = "2FA enabled; backup codes returned"),
        (status = 400, description = "Setup not initiated or invalid code"),
        (status = 401, description = "Authentication required or invalid TOTP code")
    ),
    security(("bearer_auth" = []))
)]
#[post("/2fa/enable")]
pub async fn enable_2fa(
    req: HttpRequest,
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<Enable2FARequest>,
) -> AppResult<HttpResponse> {
    // Get locale from request for localized emails
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);

    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let user_id = user.claims().sub;

    let user_data = container
        .users
        .find(&user_id)
        .await
        .map_err(AppError::Database)?;

    let secret = user_data
        .otp_secret
        .as_ref()
        .ok_or_else(|| AppError::BadRequest(t!("auth.2fa.setup_not_initiated").into_owned()))?;

    verify_totp(secret, &body.otp_code)?;

    // Generate backup codes with higher entropy (8 chars = ~48 bits)
    let backup_codes_plain: Vec<String> = (0..8)
        .map(|_| generate_random_token(8).to_uppercase())
        .collect();

    // Hash backup codes before storing (using same Argon2id as passwords)
    let backup_codes_hashed: Vec<String> = backup_codes_plain
        .iter()
        .map(|code| hash_password(code, container.config.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;

    // Store hashed backup codes (wrapped in Option for DB schema compatibility)
    let backup_codes_for_storage: Vec<Option<String>> =
        backup_codes_hashed.into_iter().map(Some).collect();

    container
        .users
        .enable_2fa(&user_id, &backup_codes_for_storage)
        .await
        .map_err(AppError::Database)?;

    let revoked_tokens = container
        .refresh_tokens
        .revoke_all_for_user(&user_id)
        .await
        .map_err(AppError::Database)?;

    // Send 2FA setup email with secret and backup codes
    let security = SecurityService::from_config(container.config.as_ref())?;
    let email = security.decrypt_user_email(&user_data)?;
    let qr_code_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        container.config.totp_issuer, email, secret, container.config.totp_issuer
    );

    let email_service = container.email_service.clone();
    if let Err(error) = email_service
        .send_2fa_setup_email(&email, secret, &qr_code_url, &backup_codes_plain, &locale)
        .await
    {
        tracing::warn!("2fa setup email delivery failed: {}", error);
    }

    tracing::info!(
        event = "auth.2fa.enabled",
        user_id = %user_id,
        revoked_tokens,
        "2fa enabled"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.2fa.setup_success"),
        "backup_codes": backup_codes_plain,
        "warning": t!("auth.2fa.backup_codes_warning")
    })))
}

// POST /api/v1/auth/2fa/disable
/// Disable 2FA for the current user after validating a fresh TOTP code.
#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/disable",
    tag = "auth",
    request_body = Enable2FARequest,
    responses(
        (status = 200, description = "2FA disabled"),
        (status = 400, description = "2FA was not enabled or invalid code"),
        (status = 401, description = "Authentication required or invalid TOTP code")
    ),
    security(("bearer_auth" = []))
)]
#[post("/2fa/disable")]
pub async fn disable_2fa(
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<Enable2FARequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let user_id = user.claims().sub;

    let user_data = container
        .users
        .find(&user_id)
        .await
        .map_err(AppError::Database)?;

    let secret = user_data
        .otp_secret
        .as_ref()
        .ok_or_else(|| AppError::BadRequest(t!("auth.2fa.not_enabled").into_owned()))?;

    verify_totp(secret, &body.otp_code)?;

    container
        .users
        .disable_2fa(&user_id)
        .await
        .map_err(AppError::Database)?;

    let revoked_tokens = container
        .refresh_tokens
        .revoke_all_for_user(&user_id)
        .await
        .map_err(AppError::Database)?;

    tracing::info!(
        event = "auth.2fa.disabled",
        user_id = %user_id,
        revoked_tokens,
        "2fa disabled"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.2fa.disabled_success")
    })))
}

// POST /api/v1/auth/change-password
/// Change the current user's password.
///
/// Requires the current password (re-verified under Argon2id) and the new
/// password (subject to strength policy). All refresh tokens are revoked
/// and a password-changed notification is dispatched.
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    tag = "auth",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required or invalid current password")
    ),
    security(("bearer_auth" = []))
)]
#[post("/change-password")]
pub async fn change_password(
    req: HttpRequest,
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<ChangePasswordRequest>,
) -> AppResult<HttpResponse> {
    // Get locale from request for localized errors/emails
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);

    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    if body.new_password != body.password_confirmation {
        return Err(AppError::Validation(
            t!("auth.password.mismatch", locale = locale).into_owned(),
        ));
    }

    let user_id = user.claims().sub;

    let user_data = container
        .users
        .find(&user_id)
        .await
        .map_err(AppError::Database)?;

    if !verify_password(&body.current_password, &user_data.encrypted_password)? {
        tracing::warn!(
            event = "auth.password.change_invalid_current",
            user_id = %user_id,
            "change password failed: invalid current password"
        );
        return Err(AppError::Unauthorized(
            t!("auth.password.invalid_current").into_owned(),
        ));
    }

    validate_password_strength(&body.new_password, &locale)?;
    let hashed = hash_password(&body.new_password, container.config.as_ref())?;

    container
        .users
        .update_password(&user_id, &hashed)
        .await
        .map_err(AppError::Database)?;

    let revoked_tokens = container
        .refresh_tokens
        .revoke_all_for_user(&user_id)
        .await
        .map_err(AppError::Database)?;

    // Send password changed notification email
    let security = SecurityService::from_config(container.config.as_ref())?;
    let email = security.decrypt_user_email(&user_data)?;
    let email_service = container.email_service.clone();
    if let Err(error) = email_service
        .send_password_changed_notification(&email, &locale)
        .await
    {
        tracing::warn!("password changed notification email failed: {}", error);
    }

    tracing::info!(
        event = "auth.password.changed",
        user_id = %user_id,
        revoked_tokens,
        "password changed"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.password.changed_success")
    })))
}

// GET /api/v1/auth/me
/// Return the id and decrypted email of the currently authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Authenticated user info"),
        (status = 401, description = "Authentication required")
    ),
    security(("bearer_auth" = []))
)]
#[get("/me")]
pub async fn me(user: AuthUser, container: web::Data<AppContainer>) -> AppResult<HttpResponse> {
    let user_data = container
        .users
        .find(&user.claims().sub)
        .await
        .map_err(AppError::Database)?;
    let security = SecurityService::from_config(container.config.as_ref())?;
    let current_email = security.decrypt_user_email(&user_data)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": user.claims().sub,
        "email": current_email,
    })))
}

// -- Helpers

fn build_user_info(
    config: &AppConfig,
    user: &User,
    profile_id: Uuid,
    roles: Vec<String>,
) -> AppResult<UserInfo> {
    let security = SecurityService::from_config(config)?;
    let email = security.decrypt_user_email(user)?;

    Ok(UserInfo {
        id: user.id,
        email,
        profile_id,
        is_otp_enabled: user.is_otp_enabled(),
        roles,
    })
}

fn primary_role_claim(roles: &[String]) -> i32 {
    if roles.iter().any(|role| role.eq_ignore_ascii_case("admin")) {
        return ROLE_ADMIN.as_i32();
    }

    if roles.iter().any(|role| {
        role.eq_ignore_ascii_case("operator")
            || role.eq_ignore_ascii_case("moderator")
            || role.eq_ignore_ascii_case("support")
            || role.eq_ignore_ascii_case("creator")
            || role.eq_ignore_ascii_case("agency")
    }) {
        return ROLE_OPERATOR.as_i32();
    }

    ROLE_VIEWER.as_i32()
}

fn verify_totp(secret_base32: &str, code: &str) -> AppResult<()> {
    use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

    let secret = Secret::Encoded(secret_base32.to_string());
    let totp = TOTP::new(
        TotpAlgorithm::SHA256,
        6,
        1,
        30,
        secret
            .to_bytes()
            .map_err(|_| AppError::Unauthorized("Invalid TOTP secret".to_string()))?,
        None,
        "".to_string(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid TOTP".to_string()))?;

    if totp
        .check_current(code)
        .map_err(|_| AppError::Unauthorized("Invalid TOTP code".to_string()))?
    {
        Ok(())
    } else {
        Err(AppError::Unauthorized("Invalid TOTP code".to_string()))
    }
}

fn generate_random_token(length: usize) -> String {
    use rand::Rng;
    use rand::rngs::OsRng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = OsRng;
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn request_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("User-Agent")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn fingerprint_value(value: &[u8]) -> String {
    value
        .iter()
        .take(6)
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join("")
}

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const REFRESH_COOKIE_PATH: &str = "/";
const LEGACY_REFRESH_COOKIE_PATH: &str = "/api/v1/auth";

fn build_refresh_cookie(config: &AppConfig, refresh_token: &str) -> Cookie<'static> {
    build_refresh_cookie_for_path(config, refresh_token, REFRESH_COOKIE_PATH)
}

fn build_refresh_cookie_for_path(
    config: &AppConfig,
    refresh_token: &str,
    path: &str,
) -> Cookie<'static> {
    let same_site = auth_cookie_same_site(config);
    let mut cookie = Cookie::build(REFRESH_COOKIE_NAME, refresh_token.to_owned())
        .http_only(true)
        .path(path.to_owned())
        .same_site(same_site)
        .max_age(Duration::seconds(config.jwt_refresh_expiry_secs));

    // Use `should_use_secure_cookies()` instead of `is_production_like()` so
    // that developers tunnelling through HTTPS proxies (ngrok, Cloudflare
    // Tunnel, etc.) can opt in via COOKIE_SECURE=true without changing the
    // environment name. The flag is always set in staging/production regardless.
    if config.should_use_secure_cookies() {
        cookie = cookie.secure(true);
    }

    cookie.finish()
}

fn clear_refresh_cookie(config: &AppConfig) -> Cookie<'static> {
    clear_refresh_cookie_for_path(config, REFRESH_COOKIE_PATH)
}

fn clear_legacy_refresh_cookie(config: &AppConfig) -> Cookie<'static> {
    clear_refresh_cookie_for_path(config, LEGACY_REFRESH_COOKIE_PATH)
}

fn clear_refresh_cookie_for_path(config: &AppConfig, path: &str) -> Cookie<'static> {
    let same_site = auth_cookie_same_site(config);
    let mut cookie = Cookie::build(REFRESH_COOKIE_NAME, "")
        .http_only(true)
        .path(path.to_owned())
        .same_site(same_site)
        .max_age(Duration::seconds(0));

    if config.should_use_secure_cookies() {
        cookie = cookie.secure(true);
    }

    cookie.finish()
}

fn extract_refresh_cookies(req: &HttpRequest) -> Vec<String> {
    req.headers()
        .get_all(actix_web::http::header::COOKIE)
        .filter_map(|value| value.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|part| {
            part.trim()
                .strip_prefix(&format!("{}=", REFRESH_COOKIE_NAME))
                .map(str::to_owned)
        })
        .filter(|value| !value.is_empty())
        .collect()
}

async fn find_valid_refresh_token(
    container: &AppContainer,
    req: &HttpRequest,
) -> AppResult<(String, RefreshToken)> {
    let refresh_tokens = extract_refresh_cookies(req);
    if refresh_tokens.is_empty() {
        return Err(AppError::Unauthorized(
            t!("auth.missing_token").into_owned(),
        ));
    }

    for refresh_token in refresh_tokens {
        let stored = match container
            .refresh_tokens
            .find_valid_by_token(&refresh_token, &container.config.refresh_token_hash_salt)
            .await
        {
            Ok(Some(token)) => token,
            Ok(None) => continue,
            Err(e) => return Err(AppError::Database(e)),
        };

        if stored.is_valid() {
            return Ok((refresh_token, stored));
        }
    }

    Err(AppError::Unauthorized("Invalid refresh token".to_string()))
}

fn auth_cookie_same_site(config: &AppConfig) -> SameSite {
    match std::env::var("AUTH_COOKIE_SAME_SITE")
        .unwrap_or_else(|_| "lax".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        // Browsers reject SameSite=None cookies without Secure.
        // In local HTTP development, force Lax to keep session cookies working.
        "none" if !config.is_production_like() => SameSite::Lax,
        "none" => SameSite::None,
        "strict" => SameSite::Strict,
        _ => SameSite::Lax,
    }
}

// Helper trait extensions for User
pub trait UserExt {
    fn is_locked(&self, lockout_duration_secs: i64) -> bool;
    fn is_confirmed(&self) -> bool;
    fn is_otp_enabled(&self) -> bool;
}

impl UserExt for User {
    fn is_locked(&self, lockout_duration_secs: i64) -> bool {
        if let Some(locked_at) = self.locked_at {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(locked_at);
            duration.num_seconds() < lockout_duration_secs
        } else {
            false
        }
    }

    fn is_confirmed(&self) -> bool {
        self.confirmed_at.is_some()
    }

    fn is_otp_enabled(&self) -> bool {
        self.otp_enabled_at.is_some() && self.otp_secret.is_some()
    }
}

// =========================================================================
// MAGIC LINK (passwordless authentication)
// =========================================================================

// Request DTO for sending a magic link.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MagicLinkRequest {
    #[validate(
        email(message = "auth.validation.invalid_email"),
        length(max = 254, message = "auth.validation.email_too_long")
    )]
    pub email: String,
}

// Request DTO for verifying a magic link token.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MagicLinkVerifyRequest {
    #[validate(length(min = 1, max = 512, message = "auth.validation.token_invalid"))]
    pub token: String,
}

const MAGIC_LINK_EXPIRY_MINUTES: i64 = 15;

/// POST /api/v1/auth/magic-link
///
/// Request a passwordless magic link to be sent to the given email.
/// Always returns 200 to prevent email enumeration.
#[utoipa::path(
    post,
    path = "/api/v1/auth/magic-link",
    tag = "auth",
    request_body = MagicLinkRequest,
    responses(
        (status = 200, description = "Magic link sent (if email exists)"),
        (status = 400, description = "Validation error"),
        (status = 429, description = "Too many requests")
    )
)]
#[post("/magic-link")]
pub async fn request_magic_link(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<MagicLinkRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let security = SecurityService::from_config(container.config.as_ref())?;
    let normalized_email = security.normalize_email(&body.email);
    let email_lookup = security.protect_email(&normalized_email)?;
    let request_ip = extract_client_ip(&req, &container.config.trusted_proxies);
    let user_agent = request_user_agent(&req);

    // Generate a random token and its digest for storage.
    let token = generate_random_token(48);
    let token_digest = hash_token(&token, &container.config.refresh_token_hash_salt);
    let now = Utc::now();
    let expires_at = now + chrono::Duration::minutes(MAGIC_LINK_EXPIRY_MINUTES);

    // Attempt to find user and create magic link token
    if let Ok(Some(user)) = container
        .users
        .find_by_email(&email_lookup.blind_index)
        .await
    {
        let ip_net: Option<ipnet::IpNet> = request_ip
            .map(|addr| addr.to_string())
            .and_then(|s| s.parse::<ipnet::IpNet>().ok());
        let new_token = NewMagicLinkToken::new(
            user.id,
            token_digest,
            ip_net,
            user_agent.clone(),
            expires_at,
        );
        let _ = container.magic_link_tokens.create(&new_token).await;
    }

    // Always attempt to send email (will fail silently for non-existent users)
    // This ensures the same timing regardless of user existence.
    let email_service = container.email_service.clone();
    // Get locale from request for localized emails
    let locale = crate::middleware::locale::RequestLocale::from_request(&req);
    if let Err(error) = email_service
        .send_magic_link_email(&body.email, &token, &locale)
        .await
    {
        tracing::debug!("magic link email delivery skipped or failed: {}", error);
    }

    tracing::info!(
        event = "auth.magic_link.requested",
        email_fingerprint = %fingerprint_value(&email_lookup.blind_index),
        "magic link requested"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.magic_link.sent")
    })))
}

/// POST /api/v1/auth/magic-link/verify
///
/// Verify a magic link token and issue a session (access + refresh tokens).
#[utoipa::path(
    post,
    path = "/api/v1/auth/magic-link/verify",
    tag = "auth",
    request_body = MagicLinkVerifyRequest,
    responses(
        (status = 200, description = "Session issued", body = AuthResponse),
        (status = 400, description = "Invalid or expired token"),
        (status = 401, description = "Token invalid or already used"),
        (status = 429, description = "Too many requests")
    )
)]
#[post("/magic-link/verify")]
pub async fn verify_magic_link(
    req: HttpRequest,
    container: web::Data<AppContainer>,
    body: web::Json<MagicLinkVerifyRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let token_digest = hash_token(&body.token, &container.config.refresh_token_hash_salt);
    let request_ip = extract_client_ip(&req, &container.config.trusted_proxies);
    let user_agent = request_user_agent(&req);

    // Find the magic link token by digest
    let magic_token = match container
        .magic_link_tokens
        .find_by_digest(&token_digest)
        .await
    {
        Ok(Some(t)) => t,
        Ok(None) => {
            tracing::warn!(
                event = "auth.magic_link.verify.invalid",
                "magic link verification failed: token not found"
            );
            return Err(AppError::Unauthorized(
                t!("auth.magic_link.invalid").into_owned(),
            ));
        },
        Err(e) => return Err(AppError::Database(e)),
    };

    // Check expiry
    if magic_token.expires_at <= chrono::Utc::now() {
        tracing::warn!(
            event = "auth.magic_link.verify.expired",
            user_id = %magic_token.user_id,
            "magic link verification failed: token expired"
        );
        return Err(AppError::Unauthorized(
            t!("auth.magic_link.expired").into_owned(),
        ));
    }

    // Check if already consumed
    if magic_token.consumed_at.is_some() {
        tracing::warn!(
            event = "auth.magic_link.verify.consumed",
            user_id = %magic_token.user_id,
            "magic link verification failed: token already used"
        );
        return Err(AppError::Unauthorized(
            t!("auth.magic_link.already_used").into_owned(),
        ));
    }

    // Mark token as consumed
    container
        .magic_link_tokens
        .mark_consumed(&magic_token.id)
        .await
        .map_err(AppError::Database)?;

    // Fetch user
    let user = container
        .users
        .find(&magic_token.user_id)
        .await
        .map_err(AppError::Database)?;

    // Check if account is locked
    if user.is_locked(container.config.account_lockout_duration_secs) {
        return Err(AppError::BadRequest(t!("auth.login.locked").into_owned()));
    }

    // Check email confirmation
    if !user.is_confirmed() {
        return Err(AppError::BadRequest(
            t!("auth.login.email_not_confirmed").into_owned(),
        ));
    }

    // Fetch profile
    let profile = container
        .profiles
        .find_by_user_id(&user.id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Internal(
            t!("users.profile_not_found").into_owned(),
        ))?;

    // Fetch roles
    let roles = get_cached_user_roles(&container, &user.id).await?;
    let role_claim = primary_role_claim(&roles);

    // Generate tokens (same as login)
    #[allow(clippy::get_first)]
    let active_kid = container.config.jwt_secrets.get(0).map(|k| k.kid.as_str());
    let access_token = create_token_with_kid(
        user.id,
        profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
        "access",
        active_kid,
    )?;

    let refresh_token_plain = generate_random_token(48);
    let refresh_token_hash = hash_token(
        &refresh_token_plain,
        &container.config.refresh_token_hash_salt,
    );

    // Store refresh token
    let ip_string = req.peer_addr().map(|addr| addr.ip().to_string());
    let ip: Option<ipnet::IpNet> = ip_string.as_ref().and_then(|s| {
        s.parse::<ipnet::IpNet>()
            .inspect_err(|e| {
                tracing::warn!(
                    event = "auth.ip_parse_error",
                    ip = %s,
                    error = %e,
                    "Failed to parse trusted peer IP as IpNet"
                );
            })
            .ok()
    });

    let new_refresh = NewRefreshToken {
        id: Uuid::new_v4(),
        user_id: user.id,
        token_hash: refresh_token_hash,
        device_info: req
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        ip_address: ip_string.clone(),
        expires_at: Utc::now()
            + chrono::Duration::seconds(container.config.jwt_refresh_expiry_secs),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    container
        .refresh_tokens
        .create(&new_refresh)
        .await
        .map_err(AppError::Database)?;

    // Record successful login
    container
        .users
        .record_successful_login(&user.id, ip)
        .await
        .map_err(AppError::Database)?;

    tracing::info!(
        event = "auth.magic_link.verify.success",
        user_id = %user.id,
        ip = request_ip.map(|ip| ip.to_string()).as_deref().unwrap_or("unknown"),
        user_agent = user_agent.as_deref().unwrap_or("unknown"),
        "magic link verification success"
    );

    let mut response = HttpResponse::Ok();
    response.cookie(build_refresh_cookie(
        container.config.as_ref(),
        &refresh_token_plain,
    ));
    response.cookie(clear_legacy_refresh_cookie(container.config.as_ref()));

    Ok(response.json(AuthResponse {
        access_token,
        token_type: "Bearer",
        expires_in: container.config.jwt_access_expiry_secs,
        user: build_user_info(container.config.as_ref(), &user, profile.id, roles)?,
    }))
}

/// Register all auth routes (relative paths — mounted under `/auth` scope by `router.rs`).
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(register)
        .service(recover_password)
        .service(forgot_password)
        .service(reset_password)
        .service(confirm)
        .service(request_magic_link)
        .service(verify_magic_link)
        .service(me)
        .service(session)
        .service(session_trailing)
        .service(refresh)
        .service(logout)
        .service(setup_2fa)
        .service(enable_2fa)
        .service(disable_2fa)
        .service(change_password);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, test, web};
    use serde_json::json;

    use super::*;
    use crate::models::profile::Profile;
    use crate::models::refresh_token::RefreshToken;
    use crate::models::user::User;
    use crate::repositories::mocks::{mock_app_config, mock_container};
    use crate::repositories::profiles_repository::MockIProfileRepository;
    use crate::repositories::refresh_tokens_repository::MockIRefreshTokenRepository;
    use crate::repositories::users_repository::MockIUserRepository;
    use crate::security::SecurityService;

    fn test_user(email: &str) -> User {
        let security = SecurityService::from_config(&mock_app_config()).unwrap();
        let protected_email = security.protect_email(email).unwrap();

        User {
            id: Uuid::new_v4(),
            email_blind_index: protected_email.blind_index,
            email_encrypted: protected_email.encrypted,
            encrypted_password: hash_password("CorrectPassword1", &mock_app_config()).unwrap(),
            reset_password_token_digest: None,
            reset_password_sent_at: None,
            remember_created_at: None,
            sign_in_count: 0,
            current_sign_in_at: None,
            last_sign_in_at: None,
            current_sign_in_ip: None,
            last_sign_in_ip: None,
            confirmation_token_digest: None,
            confirmed_at: Some(chrono::Utc::now()),
            confirmation_sent_at: None,
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: None,
            failed_attempts: 0,
            unlock_token_digest: None,
            locked_at: None,
            otp_secret: None,
            otp_enabled_at: None,
            otp_backup_codes: None,
            encryption_key_version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn test_profile(user_id: Uuid) -> Profile {
        Profile {
            id: Uuid::new_v4(),
            user_id,
            first_name: None,
            last_name: None,
            full_name: None,
            nickname: None,
            bio: None,
            avatar: None,
            birthday: None,
            cpf_encrypted: None,
            cpf_blind_index: None,
            phone_encrypted: None,
            phone_blind_index: None,
            whatsapp_encrypted: None,
            whatsapp_blind_index: None,
            status: true,
            social_network: json!({}),
            encryption_key_version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[actix_web::test]
    async fn test_login_user_not_found() {
        let mut mock_users = MockIUserRepository::new();

        mock_users
            .expect_find_by_email()
            .withf(|blind_index| blind_index.len() == 32)
            .times(1)
            .returning(|_| Ok(None));

        mock_users.expect_count().times(1).returning(|| Ok(0));

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({
                "email": "nonexistent@example.com",
                "password": "Password123"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Ensure that a non-existing user gets a 401 Unauthorized
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_login_invalid_password() {
        let mut mock_users = MockIUserRepository::new();

        mock_users
            .expect_find_by_email()
            .withf(|blind_index| blind_index.len() == 32)
            .times(1)
            .returning(|_| Ok(Some(test_user("user@example.com"))));

        mock_users
            .expect_record_failed_login()
            .times(1)
            .returning(|_, _, _| Ok(1));

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({
                "email": "user@example.com",
                "password": "WrongPassword2"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Ensure invalid password gets 401
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn login_sets_http_only_refresh_cookie_and_hides_token_from_json() {
        let user = test_user("user@example.com");
        let profile = test_profile(user.id);

        let mut mock_users = MockIUserRepository::new();
        mock_users
            .expect_find_by_email()
            .times(1)
            .returning(move |_| Ok(Some(user.clone())));
        mock_users
            .expect_record_successful_login()
            .times(1)
            .returning(|_, _| Ok(1));
        mock_users
            .expect_get_user_roles()
            .times(1)
            .returning(|_| Ok(vec!["fan".to_string()]));

        let mut mock_profiles = MockIProfileRepository::new();
        mock_profiles
            .expect_find_by_user_id()
            .times(1)
            .returning(move |_| Ok(Some(profile.clone())));

        let mut mock_refresh_tokens = MockIRefreshTokenRepository::new();
        mock_refresh_tokens.expect_create().times(1).returning(|_| {
            Ok(RefreshToken {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                token_hash: "hash".to_string(),
                device_info: None,
                ip_address: None,
                expires_at: chrono::Utc::now(),
                revoked_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        });

        let mut container = mock_container();
        container.users = Arc::new(mock_users);
        container.profiles = Arc::new(mock_profiles);
        container.refresh_tokens = Arc::new(mock_refresh_tokens);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .insert_header(("User-Agent", "test-agent"))
            .set_json(json!({
                "email": "user@example.com",
                "password": "CorrectPassword1"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let set_cookie = resp
            .headers()
            .get(actix_web::http::header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(set_cookie.contains("refresh_token="));
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("Path=/"));

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("refresh_token").is_none());
        assert!(body.get("access_token").is_some());
    }

    #[actix_web::test]
    async fn refresh_rejects_requests_without_refresh_cookie() {
        let container = mock_container();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(refresh),
        )
        .await;

        let req = test::TestRequest::post().uri("/refresh").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn session_rejects_requests_without_refresh_cookie() {
        let container = mock_container();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(session),
        )
        .await;

        let req = test::TestRequest::get().uri("/session").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn session_returns_user_from_valid_refresh_cookie_without_rotating_cookie() {
        let refresh_token_plain = "plain-refresh-token";
        let user = test_user("user@example.com");
        let profile = test_profile(user.id);
        let user_for_find = user.clone();
        let refresh_user_id = user.id;

        let mut mock_users = MockIUserRepository::new();
        mock_users
            .expect_find()
            .times(1)
            .returning(move |_| Ok(user_for_find.clone()));
        mock_users
            .expect_get_user_roles()
            .times(1)
            .returning(|_| Ok(vec!["fan".to_string()]));

        let mut mock_profiles = MockIProfileRepository::new();
        mock_profiles
            .expect_find_by_user_id()
            .times(1)
            .returning(move |_| Ok(Some(profile.clone())));

        let mut mock_refresh_tokens = MockIRefreshTokenRepository::new();
        let refresh_token_plain_owned = refresh_token_plain.to_string();
        mock_refresh_tokens
            .expect_find_valid_by_token()
            .withf(move |value, _| value == refresh_token_plain_owned)
            .times(1)
            .returning(move |_, _| {
                Ok(Some(RefreshToken {
                    id: Uuid::new_v4(),
                    user_id: refresh_user_id,
                    token_hash: "hash".to_string(),
                    device_info: Some("test-agent".to_string()),
                    ip_address: None,
                    expires_at: chrono::Utc::now() + chrono::Duration::days(1),
                    revoked_at: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let mut container = mock_container();
        container.users = Arc::new(mock_users);
        container.profiles = Arc::new(mock_profiles);
        container.refresh_tokens = Arc::new(mock_refresh_tokens);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(session),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/session")
            .cookie(Cookie::new(REFRESH_COOKIE_NAME, refresh_token_plain))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        assert!(
            resp.headers()
                .get(actix_web::http::header::SET_COOKIE)
                .is_none()
        );

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(
            body.get("user")
                .and_then(|u| u.get("email"))
                .and_then(|v| v.as_str()),
            Some("user@example.com")
        );
        assert!(body.get("access_token").is_some());
        assert_eq!(
            body.get("token_type").and_then(|v| v.as_str()),
            Some("Bearer")
        );
    }

    #[actix_web::test]
    async fn refresh_accepts_valid_cookie_when_legacy_cookie_is_also_present() {
        let valid_refresh_token = "valid-refresh-token";
        let invalid_refresh_token = "invalid-refresh-token";
        let user = test_user("user@example.com");
        let user_for_find = user.clone();
        let profile = test_profile(user.id);
        let refresh_user_id = user.id;

        let mut mock_users = MockIUserRepository::new();
        mock_users
            .expect_find()
            .times(1)
            .returning(move |_| Ok(user_for_find.clone()));
        mock_users
            .expect_get_user_roles()
            .times(1)
            .returning(|_| Ok(vec!["fan".to_string()]));

        let mut mock_profiles = MockIProfileRepository::new();
        mock_profiles
            .expect_find_by_user_id()
            .times(1)
            .returning(move |_| Ok(Some(profile.clone())));

        let mut mock_refresh_tokens = MockIRefreshTokenRepository::new();

        // Mock rotate_token for the valid token (first invalid token will be skipped by rotate_token returning None)
        let rotated_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id: refresh_user_id,
            token_hash: "new-hash".to_string(),
            device_info: Some("test-agent".to_string()),
            ip_address: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(1),
            revoked_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        mock_refresh_tokens
            .expect_rotate_token()
            .times(1)
            .returning(move |_, _, _| {
                Ok(Some((rotated_token.clone(), "new-plain-token".to_string())))
            });

        let mut container = mock_container();
        container.users = Arc::new(mock_users);
        container.profiles = Arc::new(mock_profiles);
        container.refresh_tokens = Arc::new(mock_refresh_tokens);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(refresh),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/refresh")
            .insert_header((
                actix_web::http::header::COOKIE,
                format!(
                    "refresh_token={}; refresh_token={}",
                    invalid_refresh_token, valid_refresh_token
                ),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let cookies: Vec<_> = resp
            .headers()
            .get_all(actix_web::http::header::SET_COOKIE)
            .filter_map(|value| value.to_str().ok())
            .collect();
        assert!(cookies.iter().any(|cookie| cookie.contains("Path=/")));
        assert!(
            cookies
                .iter()
                .any(|cookie| cookie.contains("Path=/api/v1/auth"))
        );
    }

    #[actix_web::test]
    async fn logout_clears_refresh_cookie_even_when_token_is_missing() {
        let container = mock_container();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(logout),
        )
        .await;

        let req = test::TestRequest::post().uri("/logout").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let set_cookie = resp
            .headers()
            .get(actix_web::http::header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(set_cookie.contains("refresh_token="));
        assert!(set_cookie.contains("Max-Age=0"));
    }

    #[actix_web::test]
    async fn confirm_rejects_invalid_confirmation_token() {
        let mut mock_users = MockIUserRepository::new();

        mock_users
            .expect_confirm_email()
            .withf(|token_digest| token_digest.contains(':') && token_digest.len() > 64)
            .times(1)
            .returning(|_| Ok(0));

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(confirm),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/confirm?token=plain-confirmation-token")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn confirm_accepts_valid_confirmation_token_digest() {
        let mut mock_users = MockIUserRepository::new();

        mock_users
            .expect_confirm_email()
            .withf(|token_digest| token_digest.contains(':') && token_digest.len() > 64)
            .times(1)
            .returning(|_| Ok(1));

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(confirm),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/confirm?token=plain-confirmation-token")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn recover_password_stores_only_reset_token_digest() {
        let mut mock_users = MockIUserRepository::new();

        mock_users
            .expect_find_by_email()
            .withf(|blind_index| blind_index.len() == 32)
            .times(1)
            .returning(|_| Ok(Some(test_user("user@example.com"))));

        mock_users
            .expect_create_password_reset_token()
            .withf(|_, token_digest, _| token_digest.contains(':') && token_digest.len() > 64)
            .times(1)
            .returning(|_, _, _| Ok(1));

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(recover_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/recover")
            .set_json(json!({
                "email": "user@example.com"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn recover_password_ignores_unknown_email() {
        let mut mock_users = MockIUserRepository::new();

        mock_users
            .expect_find_by_email()
            .withf(|blind_index| blind_index.len() == 32)
            .times(1)
            .returning(|_| Ok(None));

        mock_users.expect_create_password_reset_token().times(0);

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(recover_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/recover")
            .set_json(json!({
                "email": "missing@example.com"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn reset_password_rejects_expired_token_digest() {
        let mut mock_users = MockIUserRepository::new();
        let mut user = test_user("user@example.com");
        user.reset_password_sent_at = Some(chrono::Utc::now() - chrono::Duration::hours(3));

        mock_users
            .expect_find_by_reset_token_digest()
            .withf(|token_digest| token_digest.contains(':') && token_digest.len() > 64)
            .times(1)
            .returning(move |_| Ok(Some(user.clone())));

        mock_users.expect_update_password().times(0);

        let mut container = mock_container();
        container.users = Arc::new(mock_users);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset")
            .set_json(json!({
                "token": "plain-reset-token",
                "password": "S3cur3P@ssw0rd!",
                "password_confirmation": "S3cur3P@ssw0rd!"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn reset_password_accepts_valid_token_digest() {
        let mut mock_users = MockIUserRepository::new();
        let mut user = test_user("user@example.com");
        // Set reset_password_sent_at to a valid time (within 2 hours)
        user.reset_password_sent_at = Some(chrono::Utc::now() - chrono::Duration::minutes(30));
        let expected_user_id = user.id;

        // Clone user for both mock expectations to avoid move issues
        let user_for_token = user.clone();
        let user_for_find = user.clone();

        mock_users
            .expect_find_by_reset_token_digest()
            .withf(|token_digest| token_digest.contains(':') && token_digest.len() > 64)
            .times(1)
            .returning(move |_| Ok(Some(user_for_token.clone())));

        mock_users
            .expect_update_password()
            .withf(move |user_id, hashed_password| {
                *user_id == expected_user_id
                    && hashed_password != "S3cur3P@ssw0rd!"
                    && verify_password("S3cur3P@ssw0rd!", hashed_password).unwrap_or(false)
            })
            .times(1)
            .returning(|_, _| Ok(1));

        // Mock find for email decryption
        mock_users
            .expect_find()
            .withf(move |user_id| *user_id == expected_user_id)
            .times(1)
            .returning(move |_| Ok(user_for_find.clone()));

        let mut mock_refresh = MockIRefreshTokenRepository::new();
        mock_refresh
            .expect_revoke_all_for_user()
            .withf(move |user_id| *user_id == expected_user_id)
            .times(1)
            .returning(|_| Ok(0));

        let mut container = mock_container();
        container.users = Arc::new(mock_users);
        container.refresh_tokens = Arc::new(mock_refresh);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset")
            .set_json(json!({
                "token": "plain-reset-token",
                "password": "S3cur3P@ssw0rd!",
                "password_confirmation": "S3cur3P@ssw0rd!"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn refresh_cookie_is_http_only_and_scoped() {
        let cookie = build_refresh_cookie(&mock_app_config(), "plain-refresh-token");

        assert!(cookie.http_only().unwrap_or(false));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.value(), "plain-refresh-token");
    }

    #[actix_web::test]
    async fn clear_refresh_cookie_expires_cookie_value() {
        let cookie = clear_refresh_cookie(&mock_app_config());

        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.max_age().map(|age| age.whole_seconds()), Some(0));
    }
}
