// The enum catalog is authored ahead of its consumers within the rbac plan
// (engine, builder, guards); the crate level allow matches models/role.rs.
#![allow(dead_code)]

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityAction {
    Read,
    Create,
    Update,
    Delete,
    Manage,
}

impl AbilityAction {
    pub fn as_code(self) -> &'static str {
        match self {
            AbilityAction::Read => "read",
            AbilityAction::Create => "create",
            AbilityAction::Update => "update",
            AbilityAction::Delete => "delete",
            AbilityAction::Manage => "manage",
        }
    }

    /// Parses an action code from a permission code segment.
    /// Unknown segments return None and callers must fail closed.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "read" => Some(AbilityAction::Read),
            "create" => Some(AbilityAction::Create),
            "update" => Some(AbilityAction::Update),
            "delete" => Some(AbilityAction::Delete),
            "manage" => Some(AbilityAction::Manage),
            _ => None,
        }
    }
}

/// Every action variant, derived once so expansions stay enum driven.
pub const ALL_ACTIONS: [AbilityAction; 5] = [
    AbilityAction::Read,
    AbilityAction::Create,
    AbilityAction::Update,
    AbilityAction::Delete,
    AbilityAction::Manage,
];

/// The four concrete actions a permission code can grant (`manage` is a rule
/// wildcard, never a stored permission code).
pub const CRUD_ACTIONS: [AbilityAction; 4] = [
    AbilityAction::Read,
    AbilityAction::Create,
    AbilityAction::Update,
    AbilityAction::Delete,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityResource {
    All,
    AuditLogs,
    Permissions,
    Profiles,
    Roles,
    Upload,
    Users,
}

impl AbilityResource {
    pub fn as_code(self) -> &'static str {
        match self {
            AbilityResource::All => "all",
            AbilityResource::AuditLogs => "audit_logs",
            AbilityResource::Permissions => "permissions",
            AbilityResource::Profiles => "profiles",
            AbilityResource::Roles => "roles",
            AbilityResource::Upload => "upload",
            AbilityResource::Users => "users",
        }
    }

    /// Parses a resource code from a permission code segment.
    /// Unknown segments return None and callers must fail closed.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "all" => Some(AbilityResource::All),
            "audit_logs" => Some(AbilityResource::AuditLogs),
            "permissions" => Some(AbilityResource::Permissions),
            "profiles" => Some(AbilityResource::Profiles),
            "roles" => Some(AbilityResource::Roles),
            "upload" => Some(AbilityResource::Upload),
            "users" => Some(AbilityResource::Users),
            _ => None,
        }
    }
}

/// The admin domain: every concrete resource the `/admin` scope can serve.
/// The `All` wildcard is excluded because a domain membership check must name
/// the resource it protects.
pub const DOMAIN_RESOURCES: [AbilityResource; 6] = [
    AbilityResource::AuditLogs,
    AbilityResource::Permissions,
    AbilityResource::Profiles,
    AbilityResource::Roles,
    AbilityResource::Upload,
    AbilityResource::Users,
];

/// Effect stored on the `roles_permissions` link as a SMALLINT.
/// The Rust model is the single source of truth: Postgres stores the integer
/// and does not validate the value list (no CHECK constraint, AGENTS.md
/// database conventions). Unknown values read from the database map to
/// `Cannot` so a future enum value never silently widens access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum PermissionEffect {
    Can = 0,
    Cannot = 1,
}

impl PermissionEffect {
    pub fn as_i16(self) -> i16 {
        self as i16
    }

    /// Fail closed: any value the model does not define reads as cannot.
    pub fn from_i16(value: i16) -> Self {
        match value {
            0 => PermissionEffect::Can,
            _ => PermissionEffect::Cannot,
        }
    }
}

/// The effective permissions of a subject: the class level authorities
/// granted and the ones denied (the CanCanCan `permissions`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Permissions {
    /// The granted `resource:action` authorities.
    pub can: std::collections::HashSet<String>,
    /// The denied `resource:action` authorities.
    pub cannot: std::collections::HashSet<String>,
}

/// Write time metadata of a scoped role: which resource type the role
/// restricts to and which record id. `resource_id = None` means the role is
/// scoped to the authenticated subject itself (self scope).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleScope {
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::{ALL_ACTIONS, AbilityAction, AbilityResource, DOMAIN_RESOURCES, PermissionEffect};

    #[test]
    fn from_i16_maps_can_and_cannot() {
        assert_eq!(PermissionEffect::from_i16(0), PermissionEffect::Can);
        assert_eq!(PermissionEffect::from_i16(1), PermissionEffect::Cannot);
    }

    #[test]
    fn from_i16_unknown_value_is_cannot() {
        for value in [-1, 2, 7, 999] {
            assert_eq!(PermissionEffect::from_i16(value), PermissionEffect::Cannot);
        }
    }

    #[test]
    fn resource_codes_round_trip_and_parse_unknown_to_none() {
        for resource in DOMAIN_RESOURCES {
            assert_eq!(
                AbilityResource::from_code(resource.as_code()),
                Some(resource)
            );
        }
        assert_eq!(
            AbilityResource::from_code("all"),
            Some(AbilityResource::All)
        );
        assert_eq!(AbilityResource::from_code("documnets"), None);
    }

    #[test]
    fn action_codes_round_trip() {
        for action in ALL_ACTIONS {
            assert_eq!(AbilityAction::from_code(action.as_code()), Some(action));
        }
    }

    #[test]
    fn domain_resources_exclude_all() {
        assert!(!DOMAIN_RESOURCES.contains(&AbilityResource::All));
        assert_eq!(DOMAIN_RESOURCES.len(), 6);
    }
}
