//! Resolution of the role assigned to a new registration.
//!
//! The role is resolved by name among the roles that are not bound to a
//! specific subject. The resolution fails closed: a missing name and an
//! ambiguous name both refuse the registration instead of guessing.

use uuid::Uuid;

use crate::models::role::Role;

/// Why the default registration role could not be resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultRoleError {
    /// No role carries the configured name.
    NotFound,
    /// More than one role carries the configured name.
    Ambiguous,
}

/// Resolves the role of a new registration from the candidates of that name.
///
/// A single candidate is the role of the registration. No candidate means the
/// name does not exist, several candidates mean the name is ambiguous: both
/// cases are refused, so a registration never lands on an unexpected role.
pub fn resolve_default_registration_role(candidates: &[Role]) -> Result<Uuid, DefaultRoleError> {
    match candidates {
        [] => Err(DefaultRoleError::NotFound),
        [only] => Ok(only.id),
        _ => Err(DefaultRoleError::Ambiguous),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::{DefaultRoleError, resolve_default_registration_role};
    use crate::models::role::Role;

    fn role(id: Uuid, name: &str, resource_type: Option<&str>) -> Role {
        Role {
            id,
            name: name.to_string(),
            resource_type: resource_type.map(str::to_string),
            resource_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn an_empty_candidate_list_is_not_found() {
        assert_eq!(
            resolve_default_registration_role(&[]),
            Err(DefaultRoleError::NotFound)
        );
    }

    #[test]
    fn a_single_candidate_is_the_default_role() {
        let id = Uuid::new_v4();

        assert_eq!(
            resolve_default_registration_role(&[role(id, "customer", Some("user"))]),
            Ok(id)
        );
    }

    #[test]
    fn several_candidates_are_ambiguous() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();

        assert_eq!(
            resolve_default_registration_role(&[
                role(first, "customer", None),
                role(second, "customer", Some("user")),
            ]),
            Err(DefaultRoleError::Ambiguous)
        );
    }
}
