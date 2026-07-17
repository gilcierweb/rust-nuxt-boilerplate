use std::collections::HashSet;

use super::types::{AbilityAction, AbilityResource};

#[derive(Debug, Clone)]
pub struct AbilityRule {
    allowed: bool,
    action: AbilityAction,
    resource: AbilityResource,
    instance_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Ability {
    rules: Vec<AbilityRule>,
}

impl Ability {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn can(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.rules.push(AbilityRule {
            allowed: true,
            action,
            resource,
            instance_only: false,
        });
    }

    #[allow(dead_code)]
    pub fn cannot(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.rules.push(AbilityRule {
            allowed: false,
            action,
            resource,
            instance_only: false,
        });
    }

    #[allow(dead_code)]
    pub fn allows(&self, action: AbilityAction, resource: AbilityResource) -> bool {
        let mut result = false;

        for rule in &self.rules {
            // Class/subject checks must ignore instance-only rules.
            if rule.instance_only {
                continue;
            }

            if matches_rule(rule, action, resource) {
                result = rule.allowed;
            }
        }

        result
    }

    #[allow(dead_code)]
    pub fn can_instance(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.rules.push(AbilityRule {
            allowed: true,
            action,
            resource,
            instance_only: true,
        });
    }

    #[allow(dead_code)]
    pub fn cannot_instance(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.rules.push(AbilityRule {
            allowed: false,
            action,
            resource,
            instance_only: true,
        });
    }

    #[allow(dead_code)]
    pub fn allows_instance(&self, action: AbilityAction, resource: AbilityResource) -> bool {
        let mut result = false;

        for rule in &self.rules {
            if matches_rule(rule, action, resource) {
                result = rule.allowed;
            }
        }

        result
    }

    pub fn authorities(&self) -> HashSet<String> {
        let mut out = HashSet::new();

        for rule in &self.rules {
            if !rule.allowed {
                continue;
            }

            match (rule.action, rule.resource) {
                (AbilityAction::Manage, AbilityResource::All) => {
                    for resource in managed_resources() {
                        for action in crud_actions() {
                            out.insert(authority_for(resource, action));
                        }
                    }
                },
                (AbilityAction::Manage, resource) => {
                    for action in crud_actions() {
                        out.insert(authority_for(resource, action));
                    }
                },
                (action, resource) => {
                    out.insert(authority_for(resource, action));
                },
            }
        }

        out
    }
}

pub fn authority_for(resource: AbilityResource, action: AbilityAction) -> String {
    format!("{}:{}", resource.as_code(), action.as_code())
}

#[allow(dead_code)]
fn matches_rule(rule: &AbilityRule, action: AbilityAction, resource: AbilityResource) -> bool {
    let action_matches = rule.action == action || rule.action == AbilityAction::Manage;
    let resource_matches = rule.resource == resource || rule.resource == AbilityResource::All;
    action_matches && resource_matches
}

fn crud_actions() -> [AbilityAction; 4] {
    [
        AbilityAction::Read,
        AbilityAction::Create,
        AbilityAction::Update,
        AbilityAction::Delete,
    ]
}

fn managed_resources() -> [AbilityResource; 4] {
    [
        AbilityResource::AuditLogs,
        AbilityResource::Roles,
        AbilityResource::Users,
        AbilityResource::All,
    ]
}

#[cfg(test)]
mod tests {
    use super::{Ability, AbilityAction, AbilityResource, authority_for};

    #[test]
    fn cannot_overrides_can_by_order() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Manage, AbilityResource::Roles);
        ability.cannot(AbilityAction::Delete, AbilityResource::Roles);

        assert!(!ability.allows(AbilityAction::Delete, AbilityResource::Roles));
        assert!(ability.allows(AbilityAction::Read, AbilityResource::Roles));
    }

    #[test]
    fn instance_rule_does_not_grant_class_check() {
        let mut ability = Ability::new();
        ability.can_instance(AbilityAction::Read, AbilityResource::Roles);

        assert!(!ability.allows(AbilityAction::Read, AbilityResource::Roles));
        assert!(ability.allows_instance(AbilityAction::Read, AbilityResource::Roles));
    }

    #[test]
    fn instance_cannot_overrides_instance_can_by_order() {
        let mut ability = Ability::new();
        ability.can_instance(AbilityAction::Update, AbilityResource::AuditLogs);
        ability.cannot_instance(AbilityAction::Update, AbilityResource::AuditLogs);

        assert!(!ability.allows_instance(AbilityAction::Update, AbilityResource::AuditLogs));
    }

    #[test]
    fn manage_all_expands_to_all_resources_and_crud_actions() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Manage, AbilityResource::All);

        let authorities = ability.authorities();
        for resource in [
            AbilityResource::AuditLogs,
            AbilityResource::Roles,
            AbilityResource::Users,
            AbilityResource::All,
        ] {
            for action in [
                AbilityAction::Read,
                AbilityAction::Create,
                AbilityAction::Update,
                AbilityAction::Delete,
            ] {
                let auth = authority_for(resource, action);
                assert!(authorities.contains(&auth), "Missing authority: {}", auth);
            }
        }
    }

    #[test]
    fn manage_specific_resource_expands_to_crud_actions() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Manage, AbilityResource::Users);

        let authorities = ability.authorities();
        assert!(authorities.contains(&authority_for(AbilityResource::Users, AbilityAction::Read)));
        assert!(authorities.contains(&authority_for(
            AbilityResource::Users,
            AbilityAction::Create
        )));
        assert!(authorities.contains(&authority_for(
            AbilityResource::Users,
            AbilityAction::Update
        )));
        assert!(authorities.contains(&authority_for(
            AbilityResource::Users,
            AbilityAction::Delete
        )));
        assert!(!authorities.contains(&authority_for(AbilityResource::Roles, AbilityAction::Read)));
    }

    #[test]
    fn single_action_produces_single_authority() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Read, AbilityResource::Users);

        let authorities = ability.authorities();
        assert_eq!(authorities.len(), 1);
        assert!(authorities.contains(&authority_for(AbilityResource::Users, AbilityAction::Read)));
    }

    #[test]
    fn empty_ability_produces_no_authorities() {
        let ability = Ability::new();
        assert!(ability.authorities().is_empty());
    }

    #[test]
    fn manage_wildcard_matches_any_action() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Manage, AbilityResource::Users);

        assert!(ability.allows(AbilityAction::Read, AbilityResource::Users));
        assert!(ability.allows(AbilityAction::Create, AbilityResource::Users));
        assert!(ability.allows(AbilityAction::Update, AbilityResource::Users));
        assert!(ability.allows(AbilityAction::Delete, AbilityResource::Users));
    }

    #[test]
    fn all_wildcard_matches_any_resource() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Read, AbilityResource::All);

        assert!(ability.allows(AbilityAction::Read, AbilityResource::Users));
        assert!(ability.allows(AbilityAction::Read, AbilityResource::Roles));
        assert!(ability.allows(AbilityAction::Read, AbilityResource::AuditLogs));
    }

    #[test]
    fn cannot_deny_takes_precedence_over_can_manage() {
        let mut ability = Ability::new();
        ability.can(AbilityAction::Manage, AbilityResource::All);
        ability.cannot(AbilityAction::Delete, AbilityResource::Users);

        assert!(ability.allows(AbilityAction::Read, AbilityResource::Users));
        assert!(!ability.allows(AbilityAction::Delete, AbilityResource::Users));
    }
}
