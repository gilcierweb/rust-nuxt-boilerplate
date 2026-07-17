use std::collections::HashSet;

use crate::models::role::ROLE_ADMIN;

use super::{admin_ability, customer_ability, engine::Ability};

pub fn build_ability(role_claim: i32, roles: &[String]) -> Ability {
    let mut ability = Ability::new();

    if is_admin(role_claim, roles) {
        admin_ability::apply_admin_ability(&mut ability);
    }

    if roles
        .iter()
        .any(|role| role.eq_ignore_ascii_case("customer"))
    {
        customer_ability::apply_customer_ability(&mut ability);

        // If customer profile is present together with admin role, keep admin grants as final rule.
        if is_admin(role_claim, roles) {
            admin_ability::apply_admin_ability(&mut ability);
        }
    }

    ability
}

pub fn build_authorities(role_claim: i32, roles: &[String]) -> HashSet<String> {
    let mut authorities = HashSet::new();

    for role in roles {
        authorities.insert(format!("ROLE_{}", role.to_uppercase()));
    }

    authorities.extend(build_ability(role_claim, roles).authorities());
    authorities
}

fn is_admin(role_claim: i32, roles: &[String]) -> bool {
    role_claim == ROLE_ADMIN.as_i32() || roles.iter().any(|role| role.eq_ignore_ascii_case("admin"))
}

#[cfg(test)]
mod tests {
    use crate::authz::ability::{AbilityAction, AbilityResource};

    use super::super::engine::authority_for;
    use super::{build_ability, build_authorities};

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
    fn customer_gets_read_authorities_in_fallback() {
        let ability = build_ability(3, &["customer".to_string()]);
        let authorities = ability.authorities();
        assert!(authorities.contains(&authority_for(AbilityResource::Users, AbilityAction::Read)));
    }
}
