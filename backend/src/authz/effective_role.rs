//! The role a subject presents, derived from its effective permissions.
//!
//! The numeric `role` claim of the access token is a presentation value only:
//! no authorization decision reads it. The role mirrors what the subject can
//! actually do, so revoking a permission lowers the role of the next token
//! issued instead of keeping a stale administrator label.
//!
//! The CanCanCan gem drives the ability from the role
//! (`can :manage, :all if user.role == "admin"`, `docs/role_based_authorization.md`).
//! This module runs the other way around, because the claim is legacy: the role
//! is read out of the ability the subject really holds. The vocabulary stays
//! the gem one, `User::ROLES`, an ordered list of roles.

use crate::authz::ability::CurrentAbility;
use crate::authz::ability::{AbilityAction, AbilityResource, DOMAIN_RESOURCES};
use crate::models::role::UserRole;

/// The write actions that make a subject an operator of the admin domain.
const MANAGEMENT_ACTIONS: [AbilityAction; 3] = [
    AbilityAction::Create,
    AbilityAction::Update,
    AbilityAction::Delete,
];

/// Derives the role a subject presents from its effective permissions.
///
/// The administrator role requires the role management permissions, the
/// operator role requires at least one class level write permission over the
/// admin domain, and everything else (read only, scoped only, empty) is the
/// viewer role.
pub fn effective_role(current: &CurrentAbility) -> UserRole {
    if current.can_check(AbilityAction::Update, AbilityResource::Roles)
        && current.can_check(AbilityAction::Read, AbilityResource::Roles)
    {
        return UserRole::Admin;
    }

    for resource in DOMAIN_RESOURCES {
        for action in MANAGEMENT_ACTIONS {
            if current.can_check(action, resource) {
                return UserRole::Operator;
            }
        }
    }

    UserRole::Viewer
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::effective_role;
    use crate::authz::ability::{
        Ability, AbilityAction, AbilityResource, CurrentAbility, build_ability_from_grants,
    };
    use crate::authz::grants::RoleGrant;
    use crate::models::role::UserRole;

    fn allow_grant(role: &str, code: &str) -> RoleGrant {
        RoleGrant {
            role_name: role.to_string(),
            resource_type: None,
            resource_id: None,
            permission_code: Some(code.to_string()),
            effect: Some(0),
        }
    }

    fn self_scoped_grant(role: &str, code: &str) -> RoleGrant {
        RoleGrant {
            role_name: role.to_string(),
            resource_type: Some("user".to_string()),
            resource_id: None,
            permission_code: Some(code.to_string()),
            effect: Some(0),
        }
    }

    fn role_of_grants(grants: &[RoleGrant]) -> UserRole {
        let subject = Uuid::new_v4();
        let ability = build_ability_from_grants(subject, grants);
        effective_role(&CurrentAbility::new(ability))
    }

    #[test]
    fn the_role_management_permissions_are_the_administrator_role() {
        let grants = vec![
            allow_grant("manager", "roles:read"),
            allow_grant("manager", "roles:update"),
        ];

        assert_eq!(role_of_grants(&grants), UserRole::Admin);
    }

    #[test]
    fn the_full_permission_set_is_the_administrator_role() {
        let mut ability = Ability::new(Uuid::new_v4());
        ability.can(AbilityAction::Manage, AbilityResource::All);

        assert_eq!(
            effective_role(&CurrentAbility::new(ability)),
            UserRole::Admin
        );
    }

    #[test]
    fn a_management_permission_is_the_operator_role() {
        let grants = vec![allow_grant("manager", "users:update")];

        assert_eq!(role_of_grants(&grants), UserRole::Operator);
    }

    #[test]
    fn read_only_permissions_are_the_viewer_role() {
        let grants = vec![
            allow_grant("viewer", "users:read"),
            allow_grant("viewer", "profiles:read"),
        ];

        assert_eq!(role_of_grants(&grants), UserRole::Viewer);
    }

    #[test]
    fn an_empty_ability_is_the_viewer_role() {
        assert_eq!(role_of_grants(&[]), UserRole::Viewer);
    }

    #[test]
    fn a_scoped_permission_never_raises_the_role() {
        let grants = vec![self_scoped_grant("customer", "profiles:update")];

        assert_eq!(role_of_grants(&grants), UserRole::Viewer);
    }
}
