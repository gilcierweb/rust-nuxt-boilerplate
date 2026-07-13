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
    Companies,
    CompanyDomains,
    CompanySettings,
    CustomerUsers,
    Customers,
    DebtCategories,
    Debts,
    Documents,
    InvoiceRequests,
    IssuedInvoices,
    PaymentTransactions,
    Roles,
    StorageObjects,
    Users,
}

impl AbilityResource {
    pub fn as_code(self) -> &'static str {
        match self {
            AbilityResource::All => "all",
            AbilityResource::AuditLogs => "audit_logs",
            AbilityResource::Companies => "companies",
            AbilityResource::CompanyDomains => "company_domains",
            AbilityResource::CompanySettings => "company_settings",
            AbilityResource::CustomerUsers => "customer_users",
            AbilityResource::Customers => "customers",
            AbilityResource::DebtCategories => "debt_categories",
            AbilityResource::Debts => "debts",
            AbilityResource::Documents => "documents",
            AbilityResource::InvoiceRequests => "invoice_requests",
            AbilityResource::IssuedInvoices => "issued_invoices",
            AbilityResource::PaymentTransactions => "payment_transactions",
            AbilityResource::Roles => "roles",
            AbilityResource::StorageObjects => "storage_objects",
            AbilityResource::Users => "users",
        }
    }
}
