use std::collections::HashSet;

use actix_web::dev::ServiceRequest;
use actix_web::{Error, web};

use crate::AppState;
use crate::authz::ability::{build_ability, build_authorities};
use crate::middleware::auth::{Claims, verify_token};
use crate::repositories::access_token_blacklist::hash_token_for_blacklist;
use crate::repositories::container::AppContainer;

/// Whitelist of permissions that can be granted via the dynamic permission system.
/// Admin/sensitive permissions are excluded and only available via role-based abilities.
/// This prevents privilege escalation where a user with `permissions:create` could
/// create roles with admin permissions.
fn permission_whitelist() -> &'static [&'static str] {
    &[
        // User permissions (non-admin)
        "users:read",
        "profiles:read",
        "profiles:update",
        // Add other non-admin permissions here as needed
    ]
}

/// Check if a permission code is in the whitelist
fn is_permission_whitelisted(permission: &str) -> bool {
    permission_whitelist()
        .iter()
        .any(|&p| p.eq_ignore_ascii_case(permission))
}

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

/// Merge role-based abilities with dynamic permission grants (pure, unit-tested).
///
/// Role abilities ALWAYS apply — including admin `Manage All` — and dynamic
/// permission codes are strictly ADDITIVE, filtered by the whitelist. The
/// previous exclusive behavior (any permission row disabled role abilities
/// entirely) locked every seeded admin out of `roles:read`/`audit-logs:read`,
/// because the seed links permission rows to the admin role while the
/// whitelist only contains non-admin codes.
///
/// Anti-escalation is preserved: a dynamic code outside the whitelist never
/// grants anything beyond what the user's roles already entitle.
pub fn merge_authorities(
    role_claim: i32,
    roles: &[String],
    permission_codes: &[String],
) -> HashSet<String> {
    let mut authorities = HashSet::new();
    for role in roles {
        authorities.insert(format!("ROLE_{}", role.to_uppercase()));
    }
    authorities.extend(build_ability(role_claim, roles).authorities());
    // Filter permissions against whitelist to prevent privilege escalation
    for perm in permission_codes {
        if is_permission_whitelisted(perm) {
            authorities.insert(perm.clone());
        } else {
            tracing::warn!(
                "grants extractor: permission '{}' not in whitelist, ignoring",
                perm
            );
        }
    }
    authorities
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
        Ok(permission_codes) => merge_authorities(claims.role, &roles, &permission_codes),
        Err(error) => {
            tracing::warn!(
                "grants extractor: failed to load user permissions, using ability fallback: {}",
                error
            );

            merge_authorities(claims.role, &roles, &[])
        },
    }
}

#[cfg(test)]
mod tests {
    use super::merge_authorities;
    use crate::authz::ability::{AbilityAction, AbilityResource, authority_for};

    fn roles(names: &[&str]) -> Vec<String> {
        names.iter().map(ToString::to_string).collect()
    }

    fn perms(codes: &[&str]) -> Vec<String> {
        codes.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn admin_with_permission_rows_keeps_manage_all() {
        // Regression: the seed links permission rows to the admin role, so
        // admins ALWAYS have non-empty permission lists. Role abilities must
        // still apply, otherwise every admin gets 403 on roles/audit-logs.
        let authorities = merge_authorities(1, &roles(&["admin"]), &perms(&["users:read"]));

        assert!(authorities.contains(&authority_for(AbilityResource::Roles, AbilityAction::Read)));
        assert!(authorities.contains(&authority_for(
            AbilityResource::AuditLogs,
            AbilityAction::Read
        )));
        assert!(authorities.contains(&authority_for(AbilityResource::Users, AbilityAction::Read)));
        assert!(authorities.contains("ROLE_ADMIN"));
    }

    #[test]
    fn admin_without_permission_rows_keeps_manage_all() {
        let authorities = merge_authorities(1, &roles(&["admin"]), &[]);

        assert!(authorities.contains(&authority_for(AbilityResource::Roles, AbilityAction::Read)));
        assert!(authorities.contains(&authority_for(
            AbilityResource::AuditLogs,
            AbilityAction::Read
        )));
    }

    #[test]
    fn non_admin_gets_only_whitelisted_dynamic_perm() {
        let authorities =
            merge_authorities(3, &roles(&["viewer"]), &perms(&["users:read"]));

        assert!(authorities.contains("users:read"));
        assert!(authorities.contains("ROLE_VIEWER"));
        assert!(!authorities.contains(&authority_for(AbilityResource::Roles, AbilityAction::Read)));
        assert!(!authorities.contains(&authority_for(
            AbilityResource::AuditLogs,
            AbilityAction::Read
        )));
        assert!(!authorities.contains(&authority_for(
            AbilityResource::Users,
            AbilityAction::Delete
        )));
    }

    #[test]
    fn non_whitelisted_dynamic_perm_grants_nothing() {
        // Anti-escalation: a directly-granted sensitive code must not confer
        // authority beyond what the user's roles entitle.
        let authorities =
            merge_authorities(3, &roles(&["viewer"]), &perms(&["roles:read"]));

        assert!(!authorities.contains("roles:read"));
        assert!(authorities.contains("ROLE_VIEWER"));
    }
}
