//! Builds the ability of a subject from the grants stored in the database.
//!
//! Every rule comes from a grant row: no code path contributes hardcoded role
//! grants. Unknown permission codes are ignored (fail closed) and effect
//! values outside the known set read as cannot.

use uuid::Uuid;

use super::engine::Ability;
use super::types::{PermissionEffect, RoleScope};
use crate::authz::grants::RoleGrant;

/// Builds the ability of `subject` from its grants.
///
/// Unscoped grants become class level rules, scoped grants become instance
/// rules restricted to their scope. A cannot link always wins over a can
/// link, whatever the order of the grants.
pub fn build_ability_from_grants(subject: Uuid, grants: &[RoleGrant]) -> Ability {
    let mut ability = Ability::new(subject);

    for grant in grants {
        let Some(code) = grant.permission_code.as_deref() else {
            // A role without links grants nothing.
            continue;
        };

        let Some((resource, action)) = super::engine::parse_code(code) else {
            tracing::debug!(
                event = "authz.unknown_code",
                code = %code,
                "permission code ignored: unknown resource or action"
            );
            continue;
        };

        let scope = scope_of(grant);
        match (effect_of(grant), scope) {
            (PermissionEffect::Can, None) => ability.can(action, resource),
            (PermissionEffect::Can, Some(scope)) => ability.can_scoped(action, resource, scope),
            (PermissionEffect::Cannot, None) => ability.cannot(action, resource),
            (PermissionEffect::Cannot, Some(scope)) => {
                ability.cannot_scoped(action, resource, scope)
            },
        }
    }

    ability
}

/// The scope of a grant: unscoped roles have none, scoped roles carry their
/// resource type and target (`resource_id = None` means self scope).
fn scope_of(grant: &RoleGrant) -> Option<RoleScope> {
    match (&grant.resource_type, grant.resource_id) {
        (None, None) => None,
        (Some(resource_type), resource_id) => Some(RoleScope {
            resource_type: resource_type.clone(),
            resource_id,
        }),
        // Malformed scope (a target without a resource type): the rule stays
        // bound to that exact target instead of widening to class level.
        (None, Some(resource_id)) => Some(RoleScope {
            resource_type: String::new(),
            resource_id: Some(resource_id),
        }),
    }
}

/// The effect of a grant. A missing effect reads as cannot (fail closed).
fn effect_of(grant: &RoleGrant) -> PermissionEffect {
    grant
        .effect
        .map(PermissionEffect::from_i16)
        .unwrap_or(PermissionEffect::Cannot)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{build_ability_from_grants, scope_of};
    use crate::authz::ability::engine::authority_for;
    use crate::authz::ability::{AbilityAction, AbilityResource};
    use crate::authz::grants::RoleGrant;

    fn subject() -> Uuid {
        Uuid::nil()
    }

    fn other() -> Uuid {
        Uuid::from_u128(42)
    }

    fn grant(role: &str, code: &str, effect: i16) -> RoleGrant {
        RoleGrant {
            role_name: role.to_string(),
            resource_type: None,
            resource_id: None,
            permission_code: Some(code.to_string()),
            effect: Some(effect),
        }
    }

    fn scoped_grant(
        role: &str,
        code: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
    ) -> RoleGrant {
        RoleGrant {
            role_name: role.to_string(),
            resource_type: Some(resource_type.to_string()),
            resource_id,
            permission_code: Some(code.to_string()),
            effect: Some(0),
        }
    }

    fn catalog_codes() -> [&'static str; 24] {
        [
            "users:read",
            "users:create",
            "users:update",
            "users:delete",
            "roles:read",
            "roles:create",
            "roles:update",
            "roles:delete",
            "audit_logs:read",
            "audit_logs:create",
            "audit_logs:update",
            "audit_logs:delete",
            "profiles:read",
            "profiles:create",
            "profiles:update",
            "profiles:delete",
            "upload:read",
            "upload:create",
            "upload:update",
            "upload:delete",
            "permissions:read",
            "permissions:create",
            "permissions:update",
            "permissions:delete",
        ]
    }

    #[test]
    fn admin_from_data_gets_full_catalog() {
        let grants: Vec<RoleGrant> = catalog_codes()
            .iter()
            .map(|code| grant("admin", code, 0))
            .collect();

        let ability = build_ability_from_grants(subject(), &grants);

        let authorities = ability.permissions().can;
        assert_eq!(authorities.len(), 24, "the full catalog must be granted");
        assert!(authorities.contains(&authority_for(
            AbilityResource::Users,
            AbilityAction::Delete
        )));
        assert!(authorities.contains(&authority_for(
            AbilityResource::Permissions,
            AbilityAction::Update
        )));
    }

    #[test]
    fn customer_self_scoped_profiles() {
        let grants = vec![
            scoped_grant("customer", "profiles:read", "user", None),
            scoped_grant("customer", "profiles:update", "user", None),
        ];

        let ability = build_ability_from_grants(subject(), &grants);

        assert!(ability.can_check_subject(
            AbilityAction::Read,
            AbilityResource::Profiles,
            subject()
        ));
        assert!(ability.can_check_subject(
            AbilityAction::Update,
            AbilityResource::Profiles,
            subject()
        ));
        assert!(!ability.can_check_subject(
            AbilityAction::Read,
            AbilityResource::Profiles,
            other()
        ));
        assert!(!ability.can_check(AbilityAction::Read, AbilityResource::Profiles));
    }

    #[test]
    fn deny_link_beats_allow_link_from_other_role() {
        let grants = vec![
            grant("admin", "users:read", 0),
            grant("admin", "users:create", 0),
            grant("restricted", "users:create", 1),
        ];

        let ability = build_ability_from_grants(subject(), &grants);

        assert!(!ability.can_check(AbilityAction::Create, AbilityResource::Users));
        assert!(ability.can_check(AbilityAction::Read, AbilityResource::Users));
    }

    #[test]
    fn unknown_code_is_ignored() {
        let grants = vec![grant("admin", "documents:read", 0)];

        let ability = build_ability_from_grants(subject(), &grants);

        assert!(ability.permissions().can.is_empty());
    }

    #[test]
    fn out_of_range_effect_is_deny() {
        let grants = vec![grant("admin", "users:read", 7)];

        let ability = build_ability_from_grants(subject(), &grants);

        assert!(!ability.can_check(AbilityAction::Read, AbilityResource::Users));
        assert!(ability.permissions().can.is_empty());
    }

    #[test]
    fn scope_of_reads_the_role_scope() {
        let unscoped = grant("admin", "users:read", 0);
        assert_eq!(scope_of(&unscoped), None);

        let scoped = scoped_grant("customer", "profiles:read", "user", None);
        let scope = scope_of(&scoped).expect("self scoped role");
        assert_eq!(scope.resource_type, "user");
        assert_eq!(scope.resource_id, None);
    }
}
