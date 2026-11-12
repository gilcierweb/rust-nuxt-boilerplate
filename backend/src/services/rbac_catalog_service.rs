//! Canonical RBAC catalog seeding.
//!
//! The catalog is application data versioned here, not in a migration:
//! migrations keep schema only (see the `add_rbac_catalog` migration). Every
//! operation is idempotent (select first, insert when missing) so repeated
//! seed runs converge to the same state without revoking anything.

use std::collections::HashMap;

use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

/// Canonical roles: five unscoped admin domain roles plus the self scoped
/// customer (`resource_type = 'user'`, `resource_id = NULL` means self).
const CANONICAL_ROLES: [(&str, Option<&str>); 6] = [
    ("admin", None),
    ("manager", None),
    ("editor", None),
    ("viewer", None),
    ("support", None),
    ("customer", Some("user")),
];

/// Canonical permissions: 6 resources x 4 CRUD actions, human readable name
/// first, machine code second.
const CANONICAL_PERMISSIONS: [(&str, &str); 24] = [
    ("users:read", "Users read"),
    ("users:create", "Users create"),
    ("users:update", "Users update"),
    ("users:delete", "Users delete"),
    ("roles:read", "Roles read"),
    ("roles:create", "Roles create"),
    ("roles:update", "Roles update"),
    ("roles:delete", "Roles delete"),
    ("audit_logs:read", "Audit logs read"),
    ("audit_logs:create", "Audit logs create"),
    ("audit_logs:update", "Audit logs update"),
    ("audit_logs:delete", "Audit logs delete"),
    ("profiles:read", "Profiles read"),
    ("profiles:create", "Profiles create"),
    ("profiles:update", "Profiles update"),
    ("profiles:delete", "Profiles delete"),
    ("upload:read", "Upload read"),
    ("upload:create", "Upload create"),
    ("upload:update", "Upload update"),
    ("upload:delete", "Upload delete"),
    ("permissions:read", "Permissions read"),
    ("permissions:create", "Permissions create"),
    ("permissions:update", "Permissions update"),
    ("permissions:delete", "Permissions delete"),
];

/// Canonical allow links per role (effect defaults to allow). The customer
/// links grant the subject read and update access to their own profile only.
const CANONICAL_ROLE_LINKS: &[(&str, &[&str])] = &[
    (
        "admin",
        &[
            "users:read",
            "users:create",
            "users:update",
            "users:delete",
            "roles:read",
            "roles:create",
            "roles:update",
            "roles:delete",
            "audit_logs:read",
            "audit_logs:create",
            "audit_logs:update",
            "audit_logs:delete",
            "profiles:read",
            "profiles:create",
            "profiles:update",
            "profiles:delete",
            "upload:read",
            "upload:create",
            "upload:update",
            "upload:delete",
            "permissions:read",
            "permissions:create",
            "permissions:update",
            "permissions:delete",
        ],
    ),
    (
        "manager",
        &[
            "users:read",
            "users:create",
            "users:update",
            "roles:read",
            "audit_logs:read",
        ],
    ),
    (
        "editor",
        &["users:read", "profiles:read", "profiles:update"],
    ),
    ("viewer", &["users:read", "profiles:read"]),
    ("support", &["users:read", "audit_logs:read"]),
    ("customer", &["profiles:read", "profiles:update"]),
];

/// Role id map returned by [`ensure_canonical_catalog`], keyed by role name.
pub type RoleIds = HashMap<String, Uuid>;

/// Converges the two protected role rows to their canonical shape.
///
/// The catalog is additive everywhere else, on purpose: an operator may rename
/// a permission through the admin and the seed never rewrites a name. The two
/// protected roles are different, because the system itself reads their names:
/// the ability builder expands `admin` into the full catalog and the
/// registration assigns `customer`.
///
/// Two shapes break the system and are repaired here, before the seed reuses
/// the rows. A scope on the administrator row first: a scoped rule grants
/// nothing at the class level, so every holder would lose the roles read and
/// the roles update at once. A rename of the administrator second: it leaves
/// the assignments on the renamed row, so the seed would otherwise create a
/// second administrator row and split the holders between the two.
pub fn converge_protected_roles(conn: &mut PgConnection) -> diesel::QueryResult<()> {
    use crate::db::schema::{permissions, roles, roles_permissions};
    use diesel::AggregateExpressionMethods;

    // The canonical shape of the two protected roles: a class level admin and
    // a self scoped customer, so the two scope columns travel together.
    for (name, resource_type) in [("admin", None), ("customer", Some("user"))] {
        diesel::update(roles::table.filter(roles::name.eq(name)))
            .set((
                roles::resource_type.eq(resource_type),
                roles::resource_id.eq(None::<Uuid>),
            ))
            .execute(conn)?;
    }

    // A rename of the administrator row: the row is recognized by what it
    // grants, not by its name. A class level role holding every canonical code
    // is the administrator, and when no row carries the canonical name it takes
    // it back.
    let canonical_admin = roles::table
        .filter(roles::name.eq("admin"))
        .select(roles::id)
        .first::<Uuid>(conn)
        .optional()?;
    if canonical_admin.is_none() {
        let codes: Vec<String> = CANONICAL_PERMISSIONS
            .iter()
            .map(|(code, _)| (*code).to_string())
            .collect();
        let holders: Vec<Uuid> = roles_permissions::table
            .inner_join(permissions::table)
            .inner_join(roles::table)
            .filter(
                roles_permissions::effect
                    .eq(0)
                    .and(permissions::code.eq_any(&codes))
                    .and(roles::resource_id.is_null()),
            )
            .group_by(roles_permissions::role_id)
            .select(roles_permissions::role_id)
            .having(
                diesel::dsl::count(permissions::code)
                    .aggregate_distinct()
                    .eq(codes.len() as i64),
            )
            .load(conn)?;

        // Exactly one holder: anything else is ambiguous, and guessing would
        // promote the wrong row.
        if let [holder] = holders.as_slice() {
            diesel::update(roles::table.find(holder))
                .set(roles::name.eq("admin"))
                .execute(conn)?;
        }
    }

    Ok(())
}

/// Seeds the canonical roles, permissions, and allow links. Idempotent and
/// additive: running it twice changes nothing, existing grants are kept.
///
/// The two protected role rows are the exception: they converge first, because
/// a scope on the administrator row or a rename of either one breaks the system
/// in a way no other row can.
pub fn ensure_canonical_catalog(conn: &mut PgConnection) -> diesel::QueryResult<RoleIds> {
    converge_protected_roles(conn)?;

    conn.transaction(|tx| {
        let mut permission_ids: HashMap<&str, Uuid> = HashMap::new();
        for (code, name) in CANONICAL_PERMISSIONS {
            let id = ensure_permission(tx, code, name, None)?;
            permission_ids.insert(code, id);
        }

        let mut role_ids: RoleIds = HashMap::new();
        for (name, resource_type) in CANONICAL_ROLES {
            let id = ensure_role(tx, name, resource_type, None)?;
            role_ids.insert(name.to_string(), id);
        }

        for (role_name, codes) in CANONICAL_ROLE_LINKS {
            let role_id = role_ids[*role_name];
            for code in codes.iter() {
                let permission_id = permission_ids[*code];
                ensure_role_permission(tx, role_id, permission_id)?;
            }
        }

        Ok(role_ids)
    })
}

/// Returns the id of the permission with the given code, creating it when
/// missing. The name is the human readable label shown by the admin.
pub fn ensure_permission(
    conn: &mut PgConnection,
    permission_code: &str,
    permission_name: &str,
    permission_description: Option<&str>,
) -> diesel::QueryResult<Uuid> {
    use crate::db::schema::permissions::dsl::*;

    if let Some(existing) = permissions
        .filter(code.eq(permission_code))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing);
    }

    diesel::insert_into(permissions)
        .values((
            id.eq(Uuid::new_v4()),
            code.eq(permission_code),
            name.eq(permission_name),
            description.eq(permission_description),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)
}

/// Returns the id of the role with the given name and scope, creating it when
/// missing. `resource_id = None` with a `resource_type` means self scope.
pub fn ensure_role(
    conn: &mut PgConnection,
    role_name: &str,
    scope_resource_type: Option<&str>,
    scope_resource_id: Option<Uuid>,
) -> diesel::QueryResult<Uuid> {
    use crate::db::schema::roles::dsl::*;

    // `eq(None)` binds a literal NULL comparison that never matches; an
    // optional filter must express the absent case as `IS NULL`.
    let mut query = roles.filter(name.eq(role_name)).select(id).into_boxed();

    query = match scope_resource_type {
        Some(value) => query.filter(resource_type.eq(value)),
        None => query.filter(resource_type.is_null()),
    };

    query = match scope_resource_id {
        Some(value) => query.filter(resource_id.eq(value)),
        None => query.filter(resource_id.is_null()),
    };

    if let Some(existing) = query.first::<Uuid>(conn).optional()? {
        return Ok(existing);
    }

    diesel::insert_into(roles)
        .values((
            id.eq(Uuid::new_v4()),
            name.eq(role_name),
            resource_type.eq(scope_resource_type),
            resource_id.eq(scope_resource_id),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)
}

/// Links a role to a permission when the link is missing. The effect column
/// defaults to allow; deny links are written by the admin api, never here.
pub fn ensure_role_permission(
    conn: &mut PgConnection,
    role_uuid: Uuid,
    permission_uuid: Uuid,
) -> diesel::QueryResult<()> {
    use crate::db::schema::roles_permissions::dsl::*;

    let existing = roles_permissions
        .filter(role_id.eq(role_uuid))
        .filter(permission_id.eq(permission_uuid))
        .select(role_id)
        .first::<Uuid>(conn)
        .optional()?;

    if existing.is_some() {
        return Ok(());
    }

    diesel::insert_into(roles_permissions)
        .values((
            role_id.eq(role_uuid),
            permission_id.eq(permission_uuid),
            created_at.eq(Utc::now()),
        ))
        .execute(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_catalog_shapes_are_consistent() {
        assert_eq!(CANONICAL_ROLES.len(), 6);
        assert_eq!(CANONICAL_PERMISSIONS.len(), 24);
        assert_eq!(CANONICAL_ROLE_LINKS.len(), 6);

        let customer_scoped = CANONICAL_ROLES
            .iter()
            .any(|(name, resource_type)| *name == "customer" && *resource_type == Some("user"));
        assert!(customer_scoped, "customer must be self scoped");

        let all_roles_linked: Vec<&str> = CANONICAL_ROLE_LINKS.iter().map(|(n, _)| *n).collect();
        for (name, _) in CANONICAL_ROLES {
            assert!(
                all_roles_linked.contains(&name),
                "role {name} must have canonical links"
            );
        }

        let admin_links = CANONICAL_ROLE_LINKS
            .iter()
            .find(|(name, _)| *name == "admin")
            .expect("admin links")
            .1;
        assert_eq!(admin_links.len(), 24, "admin holds the full catalog");

        for (_, codes) in CANONICAL_ROLE_LINKS {
            for code in codes.iter() {
                assert!(
                    CANONICAL_PERMISSIONS.iter().any(|(c, _)| c == code),
                    "link code {code} must exist in the catalog"
                );
            }
        }
    }
}
