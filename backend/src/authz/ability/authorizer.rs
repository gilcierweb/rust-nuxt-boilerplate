//! Authorization checks over the ability of the authenticated subject.

use super::engine::Ability;
use super::types::{AbilityAction, AbilityResource};
use crate::errors::AppError;

/// Checks a class level permission (the CanCanCan `authorize!`): the action
/// first, then the subject, exactly the argument order of the gem.
pub fn authorize(
    ability: &Ability,
    action: AbilityAction,
    resource: AbilityResource,
) -> Result<(), AppError> {
    if ability.can_check(action, resource) {
        return Ok(());
    }

    Err(forbidden(resource, action))
}

/// Checks a permission on one target: class level rules apply to every
/// target, scoped rules apply only inside their scope.
pub fn authorize_subject(
    ability: &Ability,
    action: AbilityAction,
    resource: AbilityResource,
    target_id: uuid::Uuid,
) -> Result<(), AppError> {
    if ability.can_check_subject(action, resource, target_id) {
        return Ok(());
    }

    Err(forbidden(resource, action))
}

/// The forbidden error of a refused check, with the localized labels.
fn forbidden(resource: AbilityResource, action: AbilityAction) -> AppError {
    let action_key = match action {
        AbilityAction::Read => "authorization.actions.read",
        AbilityAction::Create => "authorization.actions.create",
        AbilityAction::Update => "authorization.actions.update",
        AbilityAction::Delete => "authorization.actions.delete",
        AbilityAction::Manage => "authorization.actions.manage",
    };
    let resource_key = match resource {
        AbilityResource::All => "authorization.resources.all",
        AbilityResource::AuditLogs => "authorization.resources.audit_logs",
        AbilityResource::Permissions => "authorization.resources.permissions",
        AbilityResource::Profiles => "authorization.resources.profiles",
        AbilityResource::Roles => "authorization.resources.roles",
        AbilityResource::Upload => "authorization.resources.upload",
        AbilityResource::Users => "authorization.resources.users",
    };
    let action_label = t!(action_key);
    let resource_label = t!(resource_key);

    AppError::Forbidden(
        t!(
            "authorization.forbidden_action",
            action = action_label.as_ref(),
            resource = resource_label.as_ref()
        )
        .into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{authorize, authorize_subject};
    use crate::authz::ability::{Ability, AbilityAction, AbilityResource};

    fn subject() -> Uuid {
        Uuid::nil()
    }

    fn other() -> Uuid {
        Uuid::from_u128(42)
    }

    #[test]
    fn authorize_allows_a_granted_permission() {
        let mut ability = Ability::new(subject());
        ability.can(AbilityAction::Read, AbilityResource::Users);

        assert!(authorize(&ability, AbilityAction::Read, AbilityResource::Users).is_ok());
    }

    #[test]
    fn authorize_refuses_a_missing_permission() {
        let ability = Ability::new(subject());

        assert!(authorize(&ability, AbilityAction::Read, AbilityResource::Users).is_err());
    }

    #[test]
    fn authorize_subject_respects_the_self_scope() {
        let mut ability = Ability::new(subject());
        ability.can_self(AbilityAction::Read, AbilityResource::Profiles);

        assert!(
            authorize_subject(
                &ability,
                AbilityAction::Read,
                AbilityResource::Profiles,
                subject()
            )
            .is_ok()
        );
        assert!(
            authorize_subject(
                &ability,
                AbilityAction::Read,
                AbilityResource::Profiles,
                other()
            )
            .is_err()
        );
    }
}
