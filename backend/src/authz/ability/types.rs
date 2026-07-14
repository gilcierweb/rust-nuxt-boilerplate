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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityResource {
    All,
    AuditLogs,
    Roles,
    Users,
}

impl AbilityResource {
    pub fn as_code(self) -> &'static str {
        match self {
            AbilityResource::All => "all",
            AbilityResource::AuditLogs => "audit_logs",
            AbilityResource::Roles => "roles",
            AbilityResource::Users => "users",
        }
    }
}
