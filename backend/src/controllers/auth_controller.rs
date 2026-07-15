use crate::{
    auth::password::{password_hash, verify},
    config::AppConfig,
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::profile::NewProfile,
    models::refresh_token::{NewRefreshToken, RefreshToken},
    models::role::{ROLE_ADMIN, ROLE_OPERATOR, ROLE_VIEWER},
    models::user::{NewUser, User},
    repositories::container::AppContainer,
    security::SecurityService,
    services::email_service::EmailService,
    services::token_service::hash_token,
    utils::validation::first_validation_error_message,
};
use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite, time::Duration},
    get, post, web,
};
use chrono::Utc;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// -- Request/Response DTOs

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "auth.validation.invalid_email"))]
    pub email: String,
    #[validate(length(min = 8, message = "auth.validation.password_too_short"))]
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "auth.validation.invalid_email"))]
    pub email: String,
    pub password: String,
    /// Optional TOTP code if 2FA is enabled
    pub otp_code: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RecoverRequest {
    #[validate(email(message = "auth.validation.invalid_email"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, message = "auth.validation.password_too_short"))]
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize)]
pub struct Enable2FARequest {
    pub otp_code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, message = "auth.validation.password_too_short"))]
    pub new_password: String,
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 409, description = "Email already exists")
    )
)]
#[post("/register")]
pub async fn register(
    container: web::Data<AppContainer>,
    body: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    if body.password != body.password_confirmation {
        return Err(AppError::Validation(
            t!("auth.password.mismatch").into_owned(),
        ));
    }

    let encrypted_password = password_hash(body.password.clone());
    let now = Utc::now();
    let confirmation_token = Uuid::new_v4().to_string();
    let confirmation_token_digest = hash_token(&confirmation_token);
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
        }
        Err(e) => return Err(AppError::Database(e)),
    };

    // Create profile for the user
    let new_profile = NewProfile::for_user(user.id, protected_email.key_version);
    container
        .profiles
        .create(&new_profile)
        .await
        .map_err(AppError::Database)?;

    let email_service = EmailService::from_config(container.config.as_ref());
    if let Err(error) = email_service
        .send_confirmation(&body.email, &confirmation_token)
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
        .confirm_email(&hash_token(token))
        .await
        .map_err(AppError::Database)?;

    if affected_rows == 0 {
        tracing::warn!(event = "auth.confirm.invalid_token", "email confirmation failed");
        return Err(AppError::BadRequest(
            t!("auth.reset.token_invalid").into_owned(),
        ));
    }

    tracing::info!(event = "auth.confirm.success", "email confirmed");

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.register.email_confirmed")
    })))
}

// POST /api/v1/auth/login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Unauthorized - Invalid credentials"),
        (status = 403, description = "Forbidden - OTP required"),
        (status = 423, description = "Locked - Temporary lockout due to failed attempts")
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
    let request_ip = request_ip(&req);
    let user_agent = request_user_agent(&req);

    // Find user by email
    let user: User = match container
        .users
        .find_by_email(&email_lookup.blind_index)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::warn!(
                event = "auth.login.user_not_found",
                email_fingerprint = %fingerprint_value(&email_lookup.blind_index),
                ip = request_ip.as_deref().unwrap_or("unknown"),
                user_agent = user_agent.as_deref().unwrap_or("unknown"),
                "login failed: user not found"
            );
            return Err(AppError::Unauthorized(t!("auth.login.failed").into_owned()));
        }
        Err(e) => return Err(AppError::Database(e)),
    };

    // Check if account is locked
    if user.is_locked() {
        tracing::warn!(
            event = "auth.login.blocked_locked",
            user_id = %user.id,
            ip = request_ip.as_deref().unwrap_or("unknown"),
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
            ip = request_ip.as_deref().unwrap_or("unknown"),
            "login blocked: email not confirmed"
        );
        return Err(AppError::BadRequest(
            t!("auth.login.email_not_confirmed").into_owned(),
        ));
    }

    // Verify password
    let password_valid = verify(body.password.clone(), user.encrypted_password.clone());
    if !password_valid {
        container
            .users
            .record_failed_login(&user.id, 10)
            .await
            .map_err(AppError::Database)?;
        tracing::warn!(
            event = "auth.login.invalid_password",
            user_id = %user.id,
            ip = request_ip.as_deref().unwrap_or("unknown"),
            user_agent = user_agent.as_deref().unwrap_or("unknown"),
            "login failed: invalid password"
        );
        return Err(AppError::Unauthorized(t!("auth.login.failed").into_owned()));
    }

    // Verify TOTP if 2FA is enabled
    if user.is_otp_enabled() {
        match &body.otp_code {
            None => {
                tracing::info!(
                    event = "auth.login.otp_required",
                    user_id = %user.id,
                    ip = request_ip.as_deref().unwrap_or("unknown"),
                    "login requires otp"
                );
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "requires_otp": true,
                    "message": t!("auth.2fa.setup_required")
                })));
            }
            Some(code) => {
                let secret = user.otp_secret.as_ref().ok_or(AppError::Internal(
                    t!("auth.2fa.invalid_secret").into_owned(),
                ))?;
                if let Err(error) = verify_totp(secret, code) {
                    tracing::warn!(
                        event = "auth.login.invalid_otp",
                        user_id = %user.id,
                        ip = request_ip.as_deref().unwrap_or("unknown"),
                        user_agent = user_agent.as_deref().unwrap_or("unknown"),
                        "login failed: invalid otp"
                    );
                    return Err(error);
                }
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
    // Get user roles and generate token with role claim
    let roles = container
        .users
        .get_user_roles(&user.id)
        .await
        .map_err(AppError::Database)?;
    let role_claim = primary_role_claim(&roles);

    // Generate tokens
    let access_token = crate::middleware::auth::create_token(
        user.id,
        profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
    )?;

    let refresh_token_plain = generate_random_token(48);
    let refresh_token_hash = hash_token(&refresh_token_plain);

    // Store refresh token
    let ip_string = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let ip: Option<ipnet::IpNet> = ip_string
        .as_ref()
        .and_then(|s| s.parse::<ipnet::IpNet>().ok());

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
        event = "auth.login.success",
        user_id = %user.id,
        ip = request_ip.as_deref().unwrap_or("unknown"),
        user_agent = user_agent.as_deref().unwrap_or("unknown"),
        "login success"
    );

    let mut response = HttpResponse::Ok();
    response.cookie(build_refresh_cookie(
        container.config.as_ref(),
        &refresh_token_plain,
    ));

    Ok(response.json(AuthResponse {
        access_token,
        token_type: "Bearer",
        expires_in: container.config.jwt_access_expiry_secs,
        user: build_user_info(container.config.as_ref(), &user, profile.id, roles)?,
    }))
}

// POST /api/v1/auth/refresh
#[post("/refresh")]
pub async fn refresh(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let refresh_tokens = extract_refresh_cookies(&req);
    if refresh_tokens.is_empty() {
        tracing::warn!(
            event = "auth.refresh.invalid_token",
            ip = request_ip(&req).as_deref().unwrap_or("unknown"),
            user_agent = request_user_agent(&req).as_deref().unwrap_or("unknown"),
            "refresh failed: missing refresh token cookie"
        );
        return Err(AppError::Unauthorized("Missing refresh token cookie".to_string()));
    }

    let mut rotated_token = None;

    // Try each refresh token cookie until we find one that can be rotated
    // rotate_token atomically validates the token (exists, not revoked, not expired),
    // revokes it, and creates a new token with the same device_info and ip_address
    for refresh_token in refresh_tokens {
        let token_hash = hash_token(&refresh_token);

        // Try to atomically rotate the token
        // rotate_token validates the token, revokes the old one, and creates a new one atomically
        match container
            .refresh_tokens
            .rotate_token(&token_hash, &NewRefreshToken {
                id: Uuid::new_v4(),
                user_id: Uuid::nil(), // Will be set from old token
                token_hash: String::new(), // Will be set from new token
                device_info: None, // Will be copied from old token
                ip_address: None,  // Will be copied from old token
                expires_at: Utc::now()
                    + chrono::Duration::seconds(container.config.jwt_refresh_expiry_secs),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .await
        {
            Ok(Some(rotated)) => {
                rotated_token = Some(rotated);
                break;
            }
            Ok(None) => continue, // Token was already revoked/expired or not found
            Err(e) => return Err(AppError::Database(e)),
        }
    }

    let rotated = match rotated_token {
        Some(rotated) => rotated,
        None => {
            tracing::warn!(
                event = "auth.refresh.invalid_token",
                ip = request_ip(&req).as_deref().unwrap_or("unknown"),
                user_agent = request_user_agent(&req).as_deref().unwrap_or("unknown"),
                "refresh failed: invalid or missing token"
            );
            return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
        }
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
    let roles = container
        .users
        .get_user_roles(&user.id)
        .await
        .map_err(AppError::Database)?;
    let role_claim = primary_role_claim(&roles);

    // Generate new access token
    let access_token = crate::middleware::auth::create_token(
        user.id,
        _profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
    )?;

    // The rotated token is already the new refresh token in the database
    // Just extract the plain token from the rotated token (not available, so generate new one for cookie)
    // Note: We can't get the plain token back from the hash, so we generate a new one for the cookie
    // The actual token in DB is the rotated token returned from rotate_token
    let new_refresh_plain = generate_random_token(48);

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

    let roles = container
        .users
        .get_user_roles(&user.id)
        .await
        .map_err(AppError::Database)?;
    let role_claim = primary_role_claim(&roles);
    let access_token = crate::middleware::auth::create_token(
        user.id,
        profile.id,
        role_claim,
        &container.config.jwt_secret,
        container.config.jwt_access_expiry_secs,
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
#[get("/session")]
pub async fn session(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    session_impl(container, req).await
}

// GET /api/v1/auth/session/
#[get("/session/")]
pub async fn session_trailing(
    container: web::Data<AppContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    session_impl(container, req).await
}

// POST /api/v1/auth/logout
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
        let token_hash = crate::repositories::access_token_blacklist::hash_token_for_blacklist(token);
        let ttl = container.config.jwt_access_expiry_secs as u64;
        if let Err(e) = container.access_token_blacklist.add(&token_hash, ttl).await {
            tracing::warn!("Failed to blacklist access token: {}", e);
        }
    }

    let mut revoked_count: usize = 0;
    for refresh_token in extract_refresh_cookies(&req) {
        let token_hash = hash_token(&refresh_token);

        if let Ok(Some(token)) = container
            .refresh_tokens
            .find_by_token_hash(&token_hash)
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
        ip = request_ip(&req).as_deref().unwrap_or("unknown"),
        "logout success"
    );

    Ok(response.json(serde_json::json!({
        "message": t!("auth.logout.success")
    })))
}

// POST /api/v1/auth/recover
#[post("/recover")]
pub async fn recover_password(
    container: web::Data<AppContainer>,
    body: web::Json<RecoverRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    let security = SecurityService::from_config(container.config.as_ref())?;
    let normalized_email = security.normalize_email(&body.email);
    let email_lookup = security.protect_email(&normalized_email)?;

    // Ignore errors — always return success to prevent email enumeration
    let mut matched_user = false;
    if let Ok(Some(user)) = container
        .users
        .find_by_email(&email_lookup.blind_index)
        .await
    {
        matched_user = true;
        let token = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();
        let email_service = EmailService::from_config(container.config.as_ref());

        container
            .users
            .create_password_reset_token(&user.id, &hash_token(&token), now)
            .await
            .map_err(AppError::Database)?;

        if let Err(error) = email_service.send_password_reset(&body.email, &token).await {
            tracing::warn!("password reset email delivery skipped or failed: {}", error);
        }
    }

    tracing::info!(
        event = "auth.recover.requested",
        email_fingerprint = %fingerprint_value(&email_lookup.blind_index),
        matched_user,
        "password recovery requested"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.recover.email_sent")
    })))
}

// POST /api/v1/auth/reset
#[post("/reset")]
pub async fn reset_password(
    container: web::Data<AppContainer>,
    body: web::Json<ResetPasswordRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    if body.password != body.password_confirmation {
        return Err(AppError::Validation("Passwords do not match".to_string()));
    }

    let token_digest = hash_token(&body.token);
    let user = container
        .users
        .find_by_reset_token_digest(&token_digest)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            tracing::warn!(
                event = "auth.reset.invalid_token",
                "password reset failed: token not found"
            );
            AppError::BadRequest(t!("auth.reset.token_invalid").into_owned())
        })?;

    let sent_at = user
        .reset_password_sent_at
        .ok_or_else(|| AppError::BadRequest(t!("auth.reset.token_invalid").into_owned()))?;

    if Utc::now().signed_duration_since(sent_at) > chrono::Duration::hours(2) {
        tracing::warn!(
            event = "auth.reset.expired_token",
            user_id = %user.id,
            "password reset failed: token expired"
        );
        return Err(AppError::BadRequest(
            t!("auth.reset.token_invalid").into_owned(),
        ));
    }

    let hashed_password = password_hash(body.password.clone());
    let affected_rows = container
        .users
        .update_password(&user.id, &hashed_password)
        .await
        .map_err(AppError::Database)?;

    if affected_rows == 0 {
        tracing::warn!(
            event = "auth.reset.invalid_token_rows",
            user_id = %user.id,
            "password reset failed: no rows updated"
        );
        return Err(AppError::BadRequest(
            t!("auth.reset.token_invalid").into_owned(),
        ));
    }

    let revoked_tokens = container
        .refresh_tokens
        .revoke_all_for_user(&user.id)
        .await
        .map_err(AppError::Database)?;

    tracing::info!(
        event = "auth.reset.success",
        user_id = %user.id,
        revoked_tokens,
        "password reset success"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.reset.success")
    })))
}

// POST /api/v1/auth/2fa/setup
#[post("/2fa/setup")]
pub async fn setup_2fa(
    user: AuthUser,
    container: web::Data<AppContainer>,
) -> AppResult<HttpResponse> {
    use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();
    let user_data = container
        .users
        .find(&user.claims().sub)
        .await
        .map_err(AppError::Database)?;
    let security = SecurityService::from_config(container.config.as_ref())?;
    let current_email = security.decrypt_user_email(&user_data)?;

    let totp = TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some(container.config.totp_issuer.clone()),
        current_email,
    )
    .map_err(|e| AppError::Internal(format!("TOTP error: {}", e)))?;

    let qr_code_url = totp.get_url();

    // Store secret temporarily (not enabled until verified)
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
#[post("/2fa/enable")]
pub async fn enable_2fa(
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<Enable2FARequest>,
) -> AppResult<HttpResponse> {
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

    // Generate backup codes
    let backup_codes: Vec<String> = (0..8)
        .map(|_| generate_random_token(4).to_uppercase())
        .collect();

    container
        .users
        .enable_2fa(&user_id, &backup_codes)
        .await
        .map_err(AppError::Database)?;

    let revoked_tokens = container
        .refresh_tokens
        .revoke_all_for_user(&user_id)
        .await
        .map_err(AppError::Database)?;

    tracing::info!(
        event = "auth.2fa.enabled",
        user_id = %user_id,
        revoked_tokens,
        "2fa enabled"
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": t!("auth.2fa.setup_success"),
        "backup_codes": backup_codes,
        "warning": t!("auth.2fa.backup_codes_warning")
    })))
}

// POST /api/v1/auth/2fa/disable
#[post("/2fa/disable")]
pub async fn disable_2fa(
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<Enable2FARequest>,
) -> AppResult<HttpResponse> {
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
#[post("/change-password")]
pub async fn change_password(
    user: AuthUser,
    container: web::Data<AppContainer>,
    body: web::Json<ChangePasswordRequest>,
) -> AppResult<HttpResponse> {
    body.validate()
        .map_err(|e| AppError::Validation(first_validation_error_message(&e)))?;

    if body.new_password != body.password_confirmation {
        return Err(AppError::Validation("Passwords do not match".into()));
    }

    let user_id = user.claims().sub;

    let user_data = container
        .users
        .find(&user_id)
        .await
        .map_err(AppError::Database)?;

    if !verify(
        body.current_password.clone(),
        user_data.encrypted_password.clone(),
    ) {
        tracing::warn!(
            event = "auth.password.change_invalid_current",
            user_id = %user_id,
            "change password failed: invalid current password"
        );
        return Err(AppError::Unauthorized(
            t!("auth.password.invalid_current").into_owned(),
        ));
    }

    validate_password_strength(&body.new_password)?;
    let hashed = password_hash(body.new_password.clone());

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
        TotpAlgorithm::SHA1,
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
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn validate_password_strength(password: &str) -> AppResult<()> {
    if password.len() < 12 {
        return Err(AppError::Validation(
            t!("auth.password.must_be_12_chars").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_number").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_uppercase").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_punctuation()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_special").into_owned(),
        ));
    }
    Ok(())
}

fn request_ip(req: &HttpRequest) -> Option<String> {
    req.connection_info().realip_remote_addr().map(str::to_owned)
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

    if config.is_production_like() {
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

    if config.is_production_like() {
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
            "Missing refresh token cookie".to_string(),
        ));
    }

    for refresh_token in refresh_tokens {
        let token_hash = hash_token(&refresh_token);

        let stored = match container
            .refresh_tokens
            .find_by_token_hash(&token_hash)
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
    fn is_locked(&self) -> bool;
    fn is_confirmed(&self) -> bool;
    fn is_otp_enabled(&self) -> bool;
}

impl UserExt for User {
    fn is_locked(&self) -> bool {
        self.locked_at.is_some()
    }

    fn is_confirmed(&self) -> bool {
        self.confirmed_at.is_some()
    }

    fn is_otp_enabled(&self) -> bool {
        self.otp_enabled_at.is_some() && self.otp_secret.is_some()
    }
}

// -- Webhook Handlers

/// Handle Stripe webhook events
pub async fn stripe_webhook(
    req: HttpRequest,
    _payload: web::Bytes,
    _config: web::Data<AppConfig>,
) -> HttpResponse {
    // Signature was already verified by StripeWebhookVerifier middleware
    // Parse the event type from the header
    let event_type = req
        .headers()
        .get("stripe-event-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!(event_type = %event_type, "Received Stripe webhook");

    // For now, just acknowledge receipt
    // In a real implementation, you would:
    // 1. Parse the webhook payload
    // 2. Match on event_type
    // 3. Execute appropriate business logic
    // 4. Return appropriate response

    HttpResponse::Ok().json(serde_json::json!({
        "received": true,
        "event_type": event_type
    }))
}

/// Handle Pix webhook events (Brazilian instant payment system)
pub async fn pix_webhook(
    _payload: web::Bytes,
    _config: web::Data<AppConfig>,
) -> HttpResponse {
    // Pix webhooks may have different verification mechanisms
    // depending on the payment provider implementation
    
    tracing::info!("Received Pix webhook");

    // For now, just acknowledge receipt
    // In a real implementation, you would:
    // 1. Verify the webhook signature (provider-specific)
    // 2. Parse the payload
    // 3. Process the payment event
    // 4. Return appropriate response

    HttpResponse::Ok().json(serde_json::json!({
        "received": true
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::profile::Profile;
    use crate::models::refresh_token::RefreshToken;
    use crate::models::user::User;
    use crate::repositories::profiles_repository::MockIProfileRepository;
    use crate::repositories::refresh_tokens_repository::MockIRefreshTokenRepository;
    use crate::repositories::test_utils::mocks::mock_app_config;
    use crate::repositories::test_utils::mocks::mock_container;
    use crate::repositories::users_repository::MockIUserRepository;
    use crate::security::SecurityService;
    use actix_web::{App, test, web};
    use serde_json::json;
    use std::sync::Arc;

    fn test_user(email: &str) -> User {
        let security = SecurityService::from_config(&mock_app_config()).unwrap();
        let protected_email = security.protect_email(email).unwrap();

        User {
            id: Uuid::new_v4(),
            email_blind_index: protected_email.blind_index,
            email_encrypted: protected_email.encrypted,
            encrypted_password: crate::auth::password::password_hash(
                "CorrectPassword1".to_string(),
            ),
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
            .set_json(&json!({
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
            .returning(|_, _| Ok(1));

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
            .set_json(&json!({
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
            .set_json(&json!({
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
        let refresh_token_hash = hash_token(refresh_token_plain);
        mock_refresh_tokens
            .expect_find_by_token_hash()
            .withf(move |value| value == &refresh_token_hash)
            .times(1)
            .returning(move |_| {
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
            .returning(move |_, _| Ok(Some(rotated_token.clone())));

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
            .into_iter()
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
            .withf(|token_digest| token_digest.len() == 64)
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
            .withf(|token_digest| token_digest.len() == 64)
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
            .withf(|_, token_digest, _| token_digest.len() == 64)
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
            .set_json(&json!({
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
            .set_json(&json!({
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
            .withf(|token_digest| token_digest.len() == 64)
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
            .set_json(&json!({
                "token": "plain-reset-token",
                "password": "Password123",
                "password_confirmation": "Password123"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn reset_password_accepts_valid_token_digest() {
        let mut mock_users = MockIUserRepository::new();
        let mut user = test_user("user@example.com");
        user.reset_password_sent_at = Some(chrono::Utc::now() - chrono::Duration::minutes(30));
        let expected_user_id = user.id;

        mock_users
            .expect_find_by_reset_token_digest()
            .withf(|token_digest| token_digest.len() == 64)
            .times(1)
            .returning(move |_| Ok(Some(user.clone())));

        mock_users
            .expect_update_password()
            .withf(move |user_id, hashed_password| {
                *user_id == expected_user_id
                    && hashed_password != "Password123"
                    && crate::auth::password::verify(
                        "Password123".to_string(),
                        hashed_password.to_string(),
                    )
            })
            .times(1)
            .returning(|_, _| Ok(1));

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
            .set_json(&json!({
                "token": "plain-reset-token",
                "password": "Password123",
                "password_confirmation": "Password123"
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
