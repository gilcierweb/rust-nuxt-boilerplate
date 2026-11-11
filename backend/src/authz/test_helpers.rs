//! Test helpers to build an ability without touching the database.

use uuid::Uuid;

use crate::authz::ability::{CurrentAbility, build_ability_from_grants};
use crate::authz::grants::RoleGrant;

/// Builds an ability granting the given permission codes at class level.
pub fn ability_from_codes(subject: Uuid, codes: &[&str]) -> CurrentAbility {
    let grants: Vec<RoleGrant> = codes
        .iter()
        .map(|code| RoleGrant {
            role_name: "test".to_string(),
            resource_type: None,
            resource_id: None,
            permission_code: Some((*code).to_string()),
            effect: Some(0),
        })
        .collect();

    CurrentAbility::new(build_ability_from_grants(subject, &grants))
}
