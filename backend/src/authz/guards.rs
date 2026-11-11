//! Write time authorization guards.
//!
//! These pure functions protect the rbac mutation endpoints against the two
//! disaster scenarios of a permissions admin: locking every administrator out
//! of role administration and granting an actor more authority than they hold.
//!
//! They take the holder sets as input, so a caller reads the database once and
//! stays testable without one.

use std::collections::HashSet;

use uuid::Uuid;

use crate::errors::AppError;

/// The permission codes that together grant role administration. At least one
/// role holding both must survive every role permissions or role deletion
/// change, otherwise no one can ever repair the system.
pub const ADMINISTRATOR_CODES: [&str; 2] = ["roles:read", "roles:update"];

/// Refuses when no role would hold the administrator codes anymore.
pub fn ensure_role_administration_survives(roles_holding_after: &[Uuid]) -> Result<(), AppError> {
    if roles_holding_after.is_empty() {
        return Err(AppError::Validation(
            t!("admin.roles.guard.administrator_source").into_owned(),
        ));
    }

    Ok(())
}

/// Refuses when the change removes the last administrator source. Removing
/// one of several holders is fine; removing the only one is not.
pub fn ensure_assignment_keeps_administrator(
    holders_before: &[Uuid],
    holders_after: &[Uuid],
) -> Result<(), AppError> {
    if holders_before.len() == 1 && holders_after.is_empty() {
        return Err(AppError::Validation(
            t!("admin.roles.guard.last_administrator_source").into_owned(),
        ));
    }

    Ok(())
}

/// Refuses when the assigned role grants authorities the actor does not hold
/// themselves (privilege escalation). Every allow code of the role must be
/// present in the actor effective allow set.
pub fn ensure_no_escalation(
    actor_allow: &HashSet<String>,
    role_allow: &HashSet<String>,
) -> Result<(), AppError> {
    if !role_allow.is_subset(actor_allow) {
        return Err(AppError::Validation(
            t!("admin.roles.guard.escalation").into_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::Uuid;

    use super::{
        ensure_assignment_keeps_administrator, ensure_no_escalation,
        ensure_role_administration_survives,
    };

    fn codes(values: &[&str]) -> HashSet<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn empty_administrator_sources_are_refused() {
        assert!(ensure_role_administration_survives(&[]).is_err());
        assert!(ensure_role_administration_survives(&[Uuid::new_v4()]).is_ok());
    }

    #[test]
    fn assignment_removing_last_holder_is_refused() {
        let holder = Uuid::new_v4();
        assert!(ensure_assignment_keeps_administrator(&[holder], &[]).is_err());
        assert!(ensure_assignment_keeps_administrator(&[holder, Uuid::new_v4()], &[]).is_ok());
    }

    #[test]
    fn escalation_is_refused() {
        let actor = codes(&["users:read"]);
        let role = codes(&["roles:read"]);
        assert!(ensure_no_escalation(&actor, &role).is_err());
        assert!(ensure_no_escalation(&role, &role).is_ok());
    }

    #[test]
    fn empty_role_allow_never_escalates() {
        let actor = codes(&[]);
        let role = codes(&[]);
        assert!(ensure_no_escalation(&actor, &role).is_ok());
    }
}
