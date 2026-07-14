use super::{Ability, AbilityAction, AbilityResource};

/// Customer abilities - can read own user profile
pub fn apply_customer_ability(ability: &mut Ability) {
    ability.can(AbilityAction::Read, AbilityResource::Users);
}
