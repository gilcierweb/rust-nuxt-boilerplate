use std::collections::HashSet;

use actix_web_grants::authorities::{AuthDetails, AuthoritiesCheck};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    repositories::container::AppContainer,
};

fn customer_scope_denied() -> AppError {
    AppError::Forbidden(t!("authorization.customer_scope_denied").into_owned())
}

pub fn is_customer(details: &AuthDetails) -> bool {
    details.has_authority("ROLE_CUSTOMER")
}

pub async fn customer_scope_ids(
    details: &AuthDetails,
    user: &AuthUser,
    container: &AppContainer,
) -> AppResult<Option<HashSet<Uuid>>> {
    if !is_customer(details) {
        return Ok(None);
    }

    let customer_ids = container
        .users
        .get_user_customer_ids(&user.claims().sub)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .collect::<HashSet<_>>();

    if customer_ids.is_empty() {
        return Err(customer_scope_denied());
    }

    Ok(Some(customer_ids))
}

pub fn ensure_customer_in_scope(scope: Option<&HashSet<Uuid>>, customer_id: Uuid) -> AppResult<()> {
    match scope {
        Some(ids) if !ids.contains(&customer_id) => Err(customer_scope_denied()),
        _ => Ok(()),
    }
}

pub fn ensure_optional_customer_in_scope(
    scope: Option<&HashSet<Uuid>>,
    customer_id: Option<Uuid>,
) -> AppResult<()> {
    match (scope, customer_id) {
        (Some(_), None) => Err(customer_scope_denied()),
        (Some(ids), Some(id)) if !ids.contains(&id) => Err(customer_scope_denied()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use actix_web_grants::authorities::AuthDetails;
    use uuid::Uuid;

    use super::{ensure_customer_in_scope, ensure_optional_customer_in_scope, is_customer};

    #[test]
    fn customer_role_is_detected() {
        let details = AuthDetails::new(["ROLE_CUSTOMER".to_string()]);
        assert!(is_customer(&details));
    }

    #[test]
    fn admin_role_is_not_customer() {
        let details = AuthDetails::new(["ROLE_ADMIN".to_string()]);
        assert!(!is_customer(&details));
    }

    #[test]
    fn out_of_scope_customer_is_denied() {
        let allowed = Uuid::new_v4();
        let denied = Uuid::new_v4();
        let mut scope = HashSet::new();
        scope.insert(allowed);

        let result = ensure_customer_in_scope(Some(&scope), denied);
        assert!(result.is_err());
    }

    #[test]
    fn missing_target_customer_is_denied_for_scoped_user() {
        let scope = HashSet::from([Uuid::new_v4()]);
        let result = ensure_optional_customer_in_scope(Some(&scope), None);
        assert!(result.is_err());
    }

    #[test]
    fn in_scope_target_customer_is_allowed() {
        let customer_id = Uuid::new_v4();
        let scope = HashSet::from([customer_id]);
        let result = ensure_optional_customer_in_scope(Some(&scope), Some(customer_id));
        assert!(result.is_ok());
    }
}
