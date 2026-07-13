use super::{Ability, AbilityAction, AbilityResource};

pub fn apply_customer_ability(ability: &mut Ability) {
    ability.can(AbilityAction::Read, AbilityResource::Customers);
    ability.can(AbilityAction::Read, AbilityResource::CustomerUsers);
    ability.can(AbilityAction::Read, AbilityResource::DebtCategories);
    ability.can(AbilityAction::Read, AbilityResource::Debts);
    ability.can(AbilityAction::Read, AbilityResource::Documents);
    ability.can(AbilityAction::Read, AbilityResource::InvoiceRequests);
    ability.can(AbilityAction::Create, AbilityResource::InvoiceRequests);
    ability.can(AbilityAction::Read, AbilityResource::IssuedInvoices);
    ability.can(AbilityAction::Read, AbilityResource::PaymentTransactions);

    // Explicit deny precedence rule (CanCanCan-style).
    ability.cannot(AbilityAction::Delete, AbilityResource::Customers);
}
