//! Domain model of a role grant.
//!
//! A grant is one role permission link of a subject, carrying the scope of
//! the role that holds it. The database rows map to this type through
//! [`RoleGrantRow`]; every authorization rule comes from these rows only.

use uuid::Uuid;

use crate::repositories::traits::users_trait::RoleGrantRow;

/// One role permission link of a subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleGrant {
    /// Name of the role holding the link.
    pub role_name: String,
    /// Scope resource type of the role; `None` means an unscoped role.
    pub resource_type: Option<String>,
    /// Scope target of the role; `None` with a resource type means self scope.
    pub resource_id: Option<Uuid>,
    /// Permission code of the link; `None` when the role holds no link.
    pub permission_code: Option<String>,
    /// Effect stored on the link; `None` when the role holds no link.
    pub effect: Option<i16>,
}

impl From<RoleGrantRow> for RoleGrant {
    fn from(row: RoleGrantRow) -> Self {
        Self {
            role_name: row.role_name,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            permission_code: row.permission_code,
            effect: row.effect,
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::RoleGrant;
    use crate::repositories::traits::users_trait::RoleGrantRow;

    #[test]
    fn row_conversion_keeps_every_field() {
        let resource_id = Uuid::new_v4();
        let row = RoleGrantRow {
            role_name: "manager".to_string(),
            resource_type: Some("user".to_string()),
            resource_id: Some(resource_id),
            permission_code: Some("profiles:read".to_string()),
            effect: Some(0),
        };

        let grant = RoleGrant::from(row);

        assert_eq!(grant.role_name, "manager");
        assert_eq!(grant.resource_type.as_deref(), Some("user"));
        assert_eq!(grant.resource_id, Some(resource_id));
        assert_eq!(grant.permission_code.as_deref(), Some("profiles:read"));
        assert_eq!(grant.effect, Some(0));
    }
}
