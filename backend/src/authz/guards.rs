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

/// The role names the system itself depends on: the administrator role the
/// ability builder expands into the full catalog, and the default role the
/// registration assigns. A rename of either breaks the code that reads it.
pub const PROTECTED_ROLE_NAMES: [&str; 2] = ["admin", "customer"];

/// Refuses a rename of a role whose name the system depends on.
pub fn ensure_role_name_is_not_protected(
    current_name: &str,
    new_name: &str,
) -> Result<(), AppError> {
    if current_name != new_name && PROTECTED_ROLE_NAMES.contains(&current_name) {
        return Err(AppError::Validation(
            t!("admin.roles.guard.protected_name").into_owned(),
        ));
    }

    Ok(())
}

/// Refuses a rename onto a name another role already holds. The ability
/// builder resolves a role by its name, so a shared name would answer two
/// different roles for one lookup.
pub fn ensure_role_name_is_free(name_holder: Option<Uuid>, role_id: Uuid) -> Result<(), AppError> {
    if let Some(holder) = name_holder
        && holder != role_id
    {
        return Err(AppError::Conflict(
            t!("admin.roles.guard.name_taken").into_owned(),
        ));
    }

    Ok(())
}

/// Serializes the administrator writes of this process.
///
/// Every guard of this module reads the holder set and the write runs
/// afterwards, so two concurrent requests could each see the other's survivor:
/// both would pass the guard and the second would empty the set, which is the
/// disaster the guards exist to prevent. One process serves the self hosted
/// deployment of this project, so a process lock closes the race. A deployment
/// of several instances would replace this with a database advisory lock on the
/// same constant key.
pub static ADMINISTRATOR_WRITE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Holds the administrator write lock across a guard read and its write.
///
/// The guard lives until the end of the handler, and a handler takes it once:
/// no caller takes it twice, so the callers cannot deadlock on it.
pub async fn hold_administrator_write() -> tokio::sync::MutexGuard<'static, ()> {
    ADMINISTRATOR_WRITE.lock().await
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use uuid::Uuid;

    use super::{
        ensure_assignment_keeps_administrator, ensure_no_escalation,
        ensure_role_administration_survives, ensure_role_name_is_free,
        ensure_role_name_is_not_protected,
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

    #[test]
    fn a_protected_role_name_is_not_renamable() {
        assert!(ensure_role_name_is_not_protected("admin", "admin").is_ok());
        assert!(ensure_role_name_is_not_protected("admin", "boss").is_err());
        assert!(ensure_role_name_is_not_protected("customer", "client").is_err());
        assert!(ensure_role_name_is_not_protected("editor", "writer").is_ok());
    }

    #[test]
    fn a_name_another_role_holds_is_refused() {
        let role = Uuid::new_v4();
        assert!(ensure_role_name_is_free(None, role).is_ok());
        assert!(ensure_role_name_is_free(Some(role), role).is_ok());
        assert!(ensure_role_name_is_free(Some(Uuid::new_v4()), role).is_err());
    }
}
