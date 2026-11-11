//! Presentation tier of a subject, derived from its effective permissions.
//!
//! The numeric `role` claim of the access token is a presentation value only:
//! no authorization decision reads it. The tier mirrors what the subject can
//! actually do, so revoking a permission lowers the tier of the next token
//! issued instead of keeping a stale administrator label.

use crate::authz::ability::CurrentAbility;
use crate::authz::ability::{AbilityAction, AbilityResource, DOMAIN_RESOURCES};
use crate::models::role::{ROLE_ADMIN, ROLE_OPERATOR, ROLE_VIEWER};

/// The presentation tier of a subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationTier {
    /// Holds the role management permissions: tier 1.
    Administrator,
    /// Holds at least one class level write permission over the admin domain:
    /// tier 2.
    Operator,
    /// Read only, scoped only or no permission at all: tier 3.
    Viewer,
}

impl PresentationTier {
    /// The number carried by the legacy `role` claim.
    pub fn as_i32(self) -> i32 {
        match self {
            PresentationTier::Administrator => ROLE_ADMIN.as_i32(),
            PresentationTier::Operator => ROLE_OPERATOR.as_i32(),
            PresentationTier::Viewer => ROLE_VIEWER.as_i32(),
        }
    }
}

/// The write actions that make a subject an operator of the admin domain.
const MANAGEMENT_ACTIONS: [AbilityAction; 3] = [
    AbilityAction::Create,
    AbilityAction::Update,
    AbilityAction::Delete,
];

/// Derives the presentation tier of the subject from its effective
/// permissions.
///
/// Tier 1 requires the role management permissions, tier 2 requires at least
/// one class level write permission over the admin domain, and everything else
/// (read only, scoped only, empty) is tier 3.
pub fn presentation_tier(current: &CurrentAbility) -> PresentationTier {
    if current.can_check(AbilityAction::Update, AbilityResource::Roles)
        && current.can_check(AbilityAction::Read, AbilityResource::Roles)
    {
        return PresentationTier::Administrator;
    }

    for resource in DOMAIN_RESOURCES {
        for action in MANAGEMENT_ACTIONS {
            if current.can_check(action, resource) {
                return PresentationTier::Operator;
            }
        }
    }

    PresentationTier::Viewer
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{PresentationTier, presentation_tier};
    use crate::authz::ability::{
        Ability, AbilityAction, AbilityResource, CurrentAbility, build_ability_from_grants,
    };
    use crate::authz::grants::RoleGrant;

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

    fn tier_of_grants(grants: &[RoleGrant]) -> PresentationTier {
        let subject = Uuid::new_v4();
        let ability = build_ability_from_grants(subject, grants);
        presentation_tier(&CurrentAbility::new(ability))
    }

    #[test]
    fn the_role_management_permissions_are_the_administrator_tier() {
        let grants = vec![
            allow_grant("manager", "roles:read"),
            allow_grant("manager", "roles:update"),
        ];

        assert_eq!(tier_of_grants(&grants), PresentationTier::Administrator);
    }

    #[test]
    fn the_full_permission_set_is_the_administrator_tier() {
        let mut ability = Ability::new(Uuid::new_v4());
        ability.can(AbilityAction::Manage, AbilityResource::All);

        assert_eq!(
            presentation_tier(&CurrentAbility::new(ability)),
            PresentationTier::Administrator
        );
    }

    #[test]
    fn a_management_permission_is_the_operator_tier() {
        let grants = vec![allow_grant("manager", "users:update")];

        assert_eq!(tier_of_grants(&grants), PresentationTier::Operator);
    }

    #[test]
    fn read_only_permissions_are_the_viewer_tier() {
        let grants = vec![
            allow_grant("viewer", "users:read"),
            allow_grant("viewer", "profiles:read"),
        ];

        assert_eq!(tier_of_grants(&grants), PresentationTier::Viewer);
    }

    #[test]
    fn an_empty_ability_is_the_viewer_tier() {
        assert_eq!(tier_of_grants(&[]), PresentationTier::Viewer);
    }

    #[test]
    fn a_scoped_permission_never_raises_the_tier() {
        let grants = vec![self_scoped_grant("customer", "profiles:update")];

        assert_eq!(tier_of_grants(&grants), PresentationTier::Viewer);
    }
}
