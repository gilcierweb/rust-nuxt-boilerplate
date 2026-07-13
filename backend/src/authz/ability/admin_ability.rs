use super::{Ability, AbilityAction, AbilityResource};

pub fn apply_admin_ability(ability: &mut Ability) {
    ability.can(AbilityAction::Manage, AbilityResource::All);
}
