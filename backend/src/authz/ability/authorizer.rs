use actix_web_grants::authorities::{AuthDetails, AuthoritiesCheck};

use crate::errors::AppError;

use super::{
    engine::authority_for,
    types::{AbilityAction, AbilityResource},
};

pub fn authorize(
    details: &AuthDetails,
    resource: AbilityResource,
    action: AbilityAction,
) -> Result<(), AppError> {
    let authority = authority_for(resource, action);

    if details.has_authority(authority.as_str()) {
        return Ok(());
    }

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
        AbilityResource::Companies => "authorization.resources.companies",
        AbilityResource::CompanyDomains => "authorization.resources.company_domains",
        AbilityResource::CompanySettings => "authorization.resources.company_settings",
        AbilityResource::CustomerUsers => "authorization.resources.customer_users",
        AbilityResource::Customers => "authorization.resources.customers",
        AbilityResource::DebtCategories => "authorization.resources.debt_categories",
        AbilityResource::Debts => "authorization.resources.debts",
        AbilityResource::Documents => "authorization.resources.documents",
        AbilityResource::InvoiceRequests => "authorization.resources.invoice_requests",
        AbilityResource::IssuedInvoices => "authorization.resources.issued_invoices",
        AbilityResource::PaymentTransactions => "authorization.resources.payment_transactions",
        AbilityResource::Roles => "authorization.resources.roles",
        AbilityResource::StorageObjects => "authorization.resources.storage_objects",
        AbilityResource::Users => "authorization.resources.users",
    };
    let action_label = t!(action_key);
    let resource_label = t!(resource_key);

    Err(AppError::Forbidden(
        t!(
            "authorization.forbidden_action",
            action = action_label.as_ref(),
            resource = resource_label.as_ref()
        )
        .into_owned(),
    ))
}
