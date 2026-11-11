use super::engine::Ability;
use super::types::{AbilityAction, AbilityResource};
use crate::models::role::ROLE_ADMIN;

/// Temporary shim: builds the ability from the role claim and role names so
/// the crate keeps compiling while the grants based transport lands in the
/// transport switch task. The hardcoded branches die there; every rule then
/// comes from database grants only.
pub fn build_ability(role_claim: i32, roles: &[String]) -> Ability {
    let mut ability = Ability::new(uuid::Uuid::nil());

    let has_admin_name = roles.iter().any(|role| role.eq_ignore_ascii_case("admin"));
    if role_claim == ROLE_ADMIN.as_i32() || has_admin_name {
        ability.can(AbilityAction::Manage, AbilityResource::All);
    }

    if roles
        .iter()
        .any(|role| role.eq_ignore_ascii_case("customer"))
    {
        // Previous behavior (customer_ability.rs): class level users:read.
        // Preserved verbatim so the extractor tests keep passing; the grants
        // based builder in the transport switch replaces this with scoped
        // rules from the database.
        ability.can(AbilityAction::Read, AbilityResource::Users);
    }

    ability
}

pub fn build_authorities(role_claim: i32, roles: &[String]) -> std::collections::HashSet<String> {
    let mut authorities = std::collections::HashSet::new();

    for role in roles {
        authorities.insert(format!("ROLE_{}", role.to_uppercase()));
    }

    authorities.extend(build_ability(role_claim, roles).authorities());
    authorities
}

#[cfg(test)]
mod tests {
    use super::super::engine::authority_for;
    use super::{build_ability, build_authorities};
    use crate::authz::ability::{AbilityAction, AbilityResource};

    #[test]
    fn admin_gets_users_crud_authorities() {
        let ability = build_ability(1, &["admin".to_string()]);
        let authorities = ability.authorities();

        assert!(authorities.contains(&authority_for(AbilityResource::Users, AbilityAction::Read)));
        assert!(authorities.contains(&authority_for(
            AbilityResource::Users,
            AbilityAction::Delete
        )));
    }

    #[test]
    fn build_authorities_adds_role_labels() {
        let authorities = build_authorities(3, &["customer".to_string()]);
        assert!(authorities.contains("ROLE_CUSTOMER"));
    }

    #[test]
    fn customer_gets_class_level_read_authority_in_shim() {
        let ability = build_ability(3, &["customer".to_string()]);
        assert!(
            ability
                .authorities()
                .contains(&authority_for(AbilityResource::Users, AbilityAction::Read))
        );
    }
}
