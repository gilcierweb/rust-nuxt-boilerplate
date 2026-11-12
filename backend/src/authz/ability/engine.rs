use std::collections::HashSet;

use uuid::Uuid;

use super::types::{
    AbilityAction, AbilityResource, CRUD_ACTIONS, DOMAIN_RESOURCES, Permissions, RoleScope,
};

/// A single authorization rule. `scope = None` grants or denies at class level
/// (every target of the resource); `scope = Some` restricts the rule to
/// instance checks only and never opens class or admin access.
#[derive(Debug, Clone)]
pub struct AbilityRule {
    allowed: bool,
    action: AbilityAction,
    resource: AbilityResource,
    scope: Option<RoleScope>,
}

/// The authorization decision object for one authenticated subject.
/// Rules come exclusively from database grants (see the data driven builder);
/// no code path contributes hardcoded role grants.
#[derive(Debug, Clone)]
pub struct Ability {
    subject: Uuid,
    rules: Vec<AbilityRule>,
}

impl Ability {
    pub fn new(subject: Uuid) -> Self {
        Self {
            subject,
            rules: Vec::new(),
        }
    }

    pub fn subject(&self) -> Uuid {
        self.subject
    }

    pub fn can(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.rules.push(AbilityRule {
            allowed: true,
            action,
            resource,
            scope: None,
        });
    }

    pub fn cannot(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.rules.push(AbilityRule {
            allowed: false,
            action,
            resource,
            scope: None,
        });
    }

    pub fn can_scoped(
        &mut self,
        action: AbilityAction,
        resource: AbilityResource,
        scope: RoleScope,
    ) {
        self.rules.push(AbilityRule {
            allowed: true,
            action,
            resource,
            scope: Some(scope),
        });
    }

    /// A rule scoped to the authenticated subject itself
    /// (`resource_id = NULL` means self in the roles table).
    pub fn can_self(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.can_scoped(
            action,
            resource,
            RoleScope {
                resource_type: "user".to_string(),
                resource_id: None,
            },
        );
    }

    pub fn cannot_scoped(
        &mut self,
        action: AbilityAction,
        resource: AbilityResource,
        scope: RoleScope,
    ) {
        self.rules.push(AbilityRule {
            allowed: false,
            action,
            resource,
            scope: Some(scope),
        });
    }

    pub fn cannot_self(&mut self, action: AbilityAction, resource: AbilityResource) {
        self.cannot_scoped(
            action,
            resource,
            RoleScope {
                resource_type: "user".to_string(),
                resource_id: None,
            },
        );
    }

    /// Class level check: scoped rules are ignored, deny beats allow.
    pub fn can_check(&self, action: AbilityAction, resource: AbilityResource) -> bool {
        self.evaluate(action, resource, None)
    }

    /// The opposite of [`Ability::can_check`].
    pub fn cannot_check(&self, action: AbilityAction, resource: AbilityResource) -> bool {
        !self.can_check(action, resource)
    }

    /// Instance level check: scoped rules apply to the target and deny still
    /// beats allow.
    pub fn can_check_subject(
        &self,
        action: AbilityAction,
        resource: AbilityResource,
        subject_id: Uuid,
    ) -> bool {
        self.evaluate(action, resource, Some(subject_id))
    }

    /// The opposite of [`Ability::can_check_subject`].
    pub fn cannot_check_subject(
        &self,
        action: AbilityAction,
        resource: AbilityResource,
        subject_id: Uuid,
    ) -> bool {
        !self.can_check_subject(action, resource, subject_id)
    }

    /// The effective permissions of the subject: the class level authorities
    /// granted and the ones denied (the CanCanCan `permissions`).
    pub fn permissions(&self) -> Permissions {
        let (can, cannot) = self.class_authorities();
        Permissions { can, cannot }
    }

    /// The granted and the denied class level authority sets.
    fn class_authorities(&self) -> (HashSet<String>, HashSet<String>) {
        let denied = self.denied_class_authorities();
        let mut out = HashSet::new();

        for rule in &self.rules {
            if !rule.allowed || rule.scope.is_some() {
                continue;
            }

            for resource in expandable_resources(rule.resource) {
                for action in expandable_actions(rule.action) {
                    let authority = authority_for(resource, action);
                    if !denied.contains(&authority) {
                        out.insert(authority);
                    }
                }
            }
        }

        (out, denied)
    }

    /// The `resource:action` set that class level deny rules match.
    fn denied_class_authorities(&self) -> HashSet<String> {
        let mut denied = HashSet::new();

        for rule in &self.rules {
            if rule.allowed || rule.scope.is_some() {
                continue;
            }

            for resource in expandable_resources(rule.resource) {
                for action in expandable_actions(rule.action) {
                    denied.insert(authority_for(resource, action));
                }
            }
        }

        denied
    }

    /// True when at least one effective class level permission covers a
    /// resource of the given domain. A scoped rule never satisfies this: it
    /// grants nothing at the class level, so it never opens the domain.
    pub fn has_class_permission_in(&self, domain: &[AbilityResource]) -> bool {
        for resource in domain {
            for action in CRUD_ACTIONS {
                if self.can_check(action, *resource)
                    || self.can_check(AbilityAction::Manage, *resource)
                {
                    return true;
                }
            }
        }
        false
    }

    fn evaluate(
        &self,
        action: AbilityAction,
        resource: AbilityResource,
        target_id: Option<Uuid>,
    ) -> bool {
        let mut result = false;

        for rule in &self.rules {
            if !self.scope_matches(rule, target_id) {
                continue;
            }

            if matches_rule(rule, action, resource) {
                if !rule.allowed {
                    // Deny beats allow: the first matching deny decides.
                    return false;
                }
                result = true;
            }
        }

        result
    }

    /// A rule participates in an evaluation when its scope fits the check:
    /// class checks see unscoped rules, instance checks see both, with the
    /// target inside the scope.
    fn scope_matches(&self, rule: &AbilityRule, target_id: Option<Uuid>) -> bool {
        match (&rule.scope, target_id) {
            (None, _) => true,
            // Class check: scoped rules never participate.
            (Some(_), None) => false,
            // Self scope: only the authenticated subject itself.
            (Some(scope), Some(target)) => match scope.resource_id {
                None => target == self.subject,
                Some(explicit) => target == explicit,
            },
        }
    }
}

fn matches_rule(rule: &AbilityRule, action: AbilityAction, resource: AbilityResource) -> bool {
    let action_matches = rule.action == action || rule.action == AbilityAction::Manage;
    let resource_matches = rule.resource == resource || rule.resource == AbilityResource::All;
    action_matches && resource_matches
}

fn expandable_actions(action: AbilityAction) -> Vec<AbilityAction> {
    if action == AbilityAction::Manage {
        CRUD_ACTIONS.to_vec()
    } else {
        vec![action]
    }
}

fn expandable_resources(resource: AbilityResource) -> Vec<AbilityResource> {
    if resource == AbilityResource::All {
        DOMAIN_RESOURCES.to_vec()
    } else {
        vec![resource]
    }
}

pub fn authority_for(resource: AbilityResource, action: AbilityAction) -> String {
    format!("{}:{}", resource.as_code(), action.as_code())
}

/// Parses a `resource:action` permission code. Splits once on `:` and both
/// halves must be known enum codes; anything else is None (fail closed).
///
/// Read by the grants data access (the builder) and by the guards module: a
/// grant is a set of these codes.
pub fn parse_code(code: &str) -> Option<(AbilityResource, AbilityAction)> {
    let (resource_part, action_part) = code.split_once(':')?;
    let resource = AbilityResource::from_code(resource_part)?;
    let action = AbilityAction::from_code(action_part)?;
    Some((resource, action))
}

/// Parses permission codes into the class level authority set they grant.
/// Unknown codes are skipped with a debug log (fail closed, never fail open)
/// and wildcards expand through the enums.
///
/// Read by the builder and by the repositories that resolve the allow set of a
/// role.
pub fn expand_allow_codes(codes: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();

    for code in codes {
        let Some((resource, action)) = parse_code(code) else {
            tracing::debug!(
                event = "authz.unknown_code",
                code = %code,
                "permission code ignored: unknown resource or action"
            );
            continue;
        };

        for expanded_resource in expandable_resources(resource) {
            for expanded_action in expandable_actions(action) {
                out.insert(authority_for(expanded_resource, expanded_action));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{
        Ability, AbilityAction, AbilityResource, RoleScope, authority_for, expand_allow_codes,
        parse_code,
    };
    use crate::authz::ability::types::{CRUD_ACTIONS, DOMAIN_RESOURCES};

    fn subject() -> Uuid {
        Uuid::nil()
    }

    fn other_uuid() -> Uuid {
        Uuid::from_u128(42)
    }

    use AbilityAction::{Create, Delete, Manage, Read, Update};
    use AbilityResource::{All, AuditLogs, Profiles, Roles, Users};

    #[test]
    fn deny_beats_allow_regardless_of_order() {
        let mut late = Ability::new(subject());
        late.can(Manage, All);
        late.cannot(Delete, Users);
        assert!(!late.can_check(Delete, Users) && late.can_check(Read, Users));

        let mut early = Ability::new(subject());
        early.cannot(Delete, Users);
        early.can(Manage, All);
        assert!(!early.can_check(Delete, Users));
    }

    #[test]
    fn permissions_expose_the_granted_and_the_denied_authorities() {
        let mut ability = Ability::new(subject());
        ability.can(Manage, All);
        ability.cannot(Delete, Users);

        let permissions = ability.permissions();
        assert!(!permissions.can.contains(&authority_for(Users, Delete)));
        assert!(permissions.can.contains(&authority_for(Users, Read)));
        assert!(permissions.cannot.contains(&authority_for(Users, Delete)));
    }

    #[test]
    fn deny_users_delete_under_manage_all() {
        let mut ability = Ability::new(subject());
        ability.can(Manage, All);
        ability.cannot(Delete, Users);
        let authorities = ability.permissions().can;
        assert!(!authorities.contains(&authority_for(Users, Delete)));
        assert!(authorities.contains(&authority_for(Users, Read)));
    }

    #[test]
    fn scoped_rule_never_grants_class_access() {
        let mut ability = Ability::new(subject());
        ability.can_self(Read, Profiles);
        assert!(!ability.can_check(Read, Profiles));
        assert!(ability.can_check_subject(Read, Profiles, subject()));
        assert!(!ability.can_check_subject(Read, Profiles, other_uuid()));
    }

    #[test]
    fn explicit_scope_matches_only_that_target() {
        let target = other_uuid();
        let mut ability = Ability::new(subject());
        ability.can_scoped(
            Read,
            Users,
            RoleScope {
                resource_type: "user".into(),
                resource_id: Some(target),
            },
        );
        assert!(ability.can_check_subject(Read, Users, target));
        assert!(!ability.can_check_subject(Read, Users, subject()));
        assert!(!ability.can_check(Read, Users));
    }

    #[test]
    fn scoped_deny_beats_class_allow_on_instance() {
        let mut ability = Ability::new(subject());
        ability.can(Read, Users);
        ability.cannot_scoped(
            Read,
            Users,
            RoleScope {
                resource_type: "user".into(),
                resource_id: Some(subject()),
            },
        );
        assert!(ability.can_check(Read, Users));
        assert!(!ability.can_check_subject(Read, Users, subject()));
    }

    #[test]
    fn manage_all_expansion_is_enum_driven() {
        let mut ability = Ability::new(subject());
        ability.can(Manage, All);
        let authorities = ability.permissions().can;
        for resource in DOMAIN_RESOURCES {
            for action in CRUD_ACTIONS {
                assert!(
                    authorities.contains(&authority_for(resource, action)),
                    "missing {}",
                    authority_for(resource, action)
                );
            }
        }
        assert_eq!(authorities.len(), 24);
    }

    #[test]
    fn manage_wildcard_matches_any_action() {
        let mut ability = Ability::new(subject());
        ability.can(Manage, Users);

        assert!(ability.can_check(Read, Users));
        assert!(ability.can_check(Create, Users));
        assert!(ability.can_check(Update, Users));
        assert!(ability.can_check(Delete, Users));
    }

    #[test]
    fn all_wildcard_matches_any_resource() {
        let mut ability = Ability::new(subject());
        ability.can(Read, All);

        assert!(ability.can_check(Read, Users));
        assert!(ability.can_check(Read, Roles));
        assert!(ability.can_check(Read, AuditLogs));
        assert!(ability.can_check(Read, Profiles));
    }

    #[test]
    fn has_class_permission_in_distinguishes_class_and_scoped() {
        let mut scoped_only = Ability::new(subject());
        scoped_only.can_self(Read, Profiles);
        assert!(!scoped_only.has_class_permission_in(&DOMAIN_RESOURCES));

        let mut class_level = Ability::new(subject());
        class_level.can(Read, Users);
        assert!(class_level.has_class_permission_in(&DOMAIN_RESOURCES));
    }

    #[test]
    fn empty_ability_produces_no_permissions() {
        let ability = Ability::new(subject());
        assert!(ability.permissions().can.is_empty());
    }

    #[test]
    fn expand_allow_codes_skips_unknown_codes() {
        let codes = [
            "users:read".to_string(),
            "documnets:read".to_string(),
            "all:manage".to_string(),
        ];
        let expanded = expand_allow_codes(&codes);
        assert!(expanded.contains("users:read"));
        assert!(!expanded.contains("documnets:read"));
    }

    #[test]
    fn expand_allow_codes_expands_wildcards() {
        let codes = vec!["all:manage".to_string()];
        let expanded = expand_allow_codes(&codes);
        assert_eq!(expanded.len(), 24);
        assert!(expanded.contains("users:delete"));
        assert!(expanded.contains("profiles:read"));
    }

    #[test]
    fn parse_code_splits_once() {
        assert_eq!(parse_code("users:read"), Some((Users, Read)));
        assert!(parse_code("nocolon").is_none());
        assert!(parse_code("users:read:extra").is_none());
    }

    #[test]
    fn cannot_check_is_the_opposite_of_can_check() {
        let mut ability = Ability::new(subject());
        ability.can(Read, Users);

        assert!(ability.can_check(Read, Users));
        assert!(!ability.cannot_check(Read, Users));
        assert!(ability.cannot_check(Delete, Users));
    }

    #[test]
    fn cannot_check_subject_respects_the_scope() {
        let mut ability = Ability::new(subject());
        ability.can_self(Read, Profiles);

        assert!(ability.can_check_subject(Read, Profiles, subject()));
        assert!(!ability.cannot_check_subject(Read, Profiles, subject()));
        assert!(ability.cannot_check_subject(Read, Profiles, other_uuid()));
    }

    #[test]
    fn permissions_separate_granted_and_denied() {
        let mut ability = Ability::new(subject());
        ability.can(Read, Users);
        ability.cannot(Delete, Users);

        let permissions = ability.permissions();

        assert!(permissions.can.contains(&authority_for(Users, Read)));
        assert!(!permissions.can.contains(&authority_for(Users, Delete)));
        assert!(permissions.cannot.contains(&authority_for(Users, Delete)));
    }
}
