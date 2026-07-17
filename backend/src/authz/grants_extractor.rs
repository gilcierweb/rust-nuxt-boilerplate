use std::collections::HashSet;

use actix_web::{Error, dev::ServiceRequest, web};

use crate::{
    AppState,
    authz::ability::{build_ability, build_authorities},
    middleware::auth::{Claims, verify_token},
    repositories::access_token_blacklist::hash_token_for_blacklist,
    repositories::container::AppContainer,
};

pub async fn extract_authorities(req: &ServiceRequest) -> Result<HashSet<String>, Error> {
    let Some(raw_token) = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
    else {
        // Keep public/auth routes working: no bearer means no authorities.
        return Ok(HashSet::new());
    };

    let secret = req
        .app_data::<web::Data<AppState>>()
        .map(|state| state.config.jwt_secret.clone())
        .or_else(|| {
            req.app_data::<web::Data<AppContainer>>()
                .map(|container| container.config.jwt_secret.clone())
        })
        .unwrap_or_default();

    // Check token blacklist first
    if let Some(container) = req.app_data::<web::Data<AppContainer>>() {
        let token_hash = hash_token_for_blacklist(raw_token);
        if container
            .access_token_blacklist
            .is_blacklisted(&token_hash)
            .await
            .unwrap_or(false)
        {
            tracing::warn!("grants extractor: token is blacklisted");
            return Ok(HashSet::new());
        }
    }

    let claims = match verify_token(raw_token, &secret) {
        Ok(claims) => claims,
        Err(error) => {
            tracing::warn!("grants extractor: invalid bearer token: {}", error);
            return Ok(HashSet::new());
        },
    };

    let container = req.app_data::<web::Data<AppContainer>>();
    Ok(build_authorities_for_claims(&claims, container).await)
}

pub async fn build_authorities_for_claims(
    claims: &Claims,
    container: Option<&web::Data<AppContainer>>,
) -> HashSet<String> {
    let Some(container) = container else {
        return build_authorities(claims.role, &[]);
    };

    let roles = match container.users.get_user_roles(&claims.sub).await {
        Ok(roles) => roles,
        Err(error) => {
            tracing::warn!("grants extractor: failed to load user roles: {}", error);
            Vec::new()
        },
    };

    match container.users.get_user_permissions(&claims.sub).await {
        Ok(permission_codes) if !permission_codes.is_empty() => {
            let mut authorities = HashSet::new();
            for role in &roles {
                authorities.insert(format!("ROLE_{}", role.to_uppercase()));
            }
            authorities.extend(permission_codes);
            authorities
        },
        Ok(_) => {
            let mut authorities = HashSet::new();
            for role in &roles {
                authorities.insert(format!("ROLE_{}", role.to_uppercase()));
            }
            authorities.extend(build_ability(claims.role, &roles).authorities());
            authorities
        },
        Err(error) => {
            tracing::warn!(
                "grants extractor: failed to load user permissions, using ability fallback: {}",
                error
            );

            let mut authorities = HashSet::new();
            for role in &roles {
                authorities.insert(format!("ROLE_{}", role.to_uppercase()));
            }
            authorities.extend(build_ability(claims.role, &roles).authorities());
            authorities
        },
    }
}
