use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text, Uuid as DieselUuid};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::DBPool;
use crate::db::schema::permissions as permissions_table;
use crate::models::permission::{
    NewPermission, Permission, PermissionPatch, PermissionRole, PermissionWithRoles,
};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::permissions_trait::{
    IPermissionRepository, PERMISSION_SORT_FIELDS,
};
use crate::utils::pagination::{ListParams, PaginatedResponse, PaginationParams};

pub struct PermissionsRepository {
    base: BaseRepo,
}

impl PermissionsRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::permissions_trait::MockIPermissionRepository;

/// A row of the catalog query: one permission and one of its role links (the
/// links are grouped in Rust after the join).
#[derive(diesel::QueryableByName)]
struct PermissionLinkRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    permission_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = Nullable<diesel::sql_types::Uuid>)]
    role_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<diesel::sql_types::Text>)]
    role_name: Option<String>,
    #[diesel(sql_type = Nullable<diesel::sql_types::Int2>)]
    effect: Option<i16>,
}

/// Splits a code into its resource and action parts. A code the enums do not
/// name cannot appear (the repository only writes codes built from them), but
/// the API never reports a wrong half.
fn code_parts(code: &str) -> (String, String) {
    match code.split_once(':') {
        Some((resource, action)) => (resource.to_string(), action.to_string()),
        None => (code.to_string(), String::new()),
    }
}

/// Groups the joined rows of one permission into the enriched item.
fn item_from_rows(rows: Vec<PermissionLinkRow>) -> Option<PermissionWithRoles> {
    let (first, rest) = rows.split_first()?;
    let (resource, action) = code_parts(&first.code);
    let mut item = PermissionWithRoles {
        id: first.permission_id,
        code: first.code.clone(),
        name: first.name.clone(),
        description: first.description.clone(),
        resource,
        action,
        roles: Vec::new(),
    };

    // The first row carries the first link, when there is one.
    for row in std::iter::once(first).chain(rest) {
        if let (Some(role_id), Some(role_name), Some(effect)) =
            (row.role_id, &row.role_name, row.effect)
        {
            item.roles.push(PermissionRole {
                role_id,
                role_name: role_name.clone(),
                effect,
            });
        }
    }

    Some(item)
}

/// The ORDER BY clause of the catalog query, built from the sort allowlist.
/// An unknown field falls back to the name order so the statement stays valid;
/// the handler refuses it before the query runs.
fn order_clause(sort_by: Option<&str>, direction: &str) -> String {
    let column = match sort_by {
        Some("code") => "p.code",
        Some("updated_at") => "p.updated_at",
        _ => "p.name",
    };
    format!("{column} {direction}")
}

#[async_trait::async_trait]
impl IPermissionRepository for PermissionsRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<Permission>> {
        self.base
            .run(|conn| {
                Box::pin(async move {
                    permissions_table::table
                        .select(Permission::as_select())
                        .load::<Permission>(conn)
                        .await
                })
            })
            .await
    }

    async fn find(&self, id: &Uuid) -> diesel::QueryResult<Permission> {
        let permission_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    permissions_table::table
                        .find(permission_id)
                        .select(Permission::as_select())
                        .first::<Permission>(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_code(&self, code: &str) -> diesel::QueryResult<Option<Permission>> {
        let code = code.to_string();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    permissions_table::table
                        .filter(crate::db::schema::permissions::code.eq(code))
                        .select(Permission::as_select())
                        .first::<Permission>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn create(&self, item: &NewPermission) -> diesel::QueryResult<Permission> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(permissions_table::table)
                        .values(&item)
                        .returning(Permission::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn update(&self, id: &Uuid, patch: &PermissionPatch) -> diesel::QueryResult<Permission> {
        let permission_id = *id;
        let patch = patch.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(permissions_table::table.find(permission_id))
                        .set(&patch)
                        .returning(Permission::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, id: &Uuid) -> diesel::QueryResult<usize> {
        let permission_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(permissions_table::table.find(permission_id))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn linked_role_count(&self, id: &Uuid) -> diesel::QueryResult<i64> {
        let permission_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    #[derive(diesel::QueryableByName)]
                    struct CountRow {
                        #[diesel(sql_type = diesel::sql_types::BigInt)]
                        total: i64,
                    }

                    diesel::sql_query(
                        "SELECT COUNT(*) AS total FROM roles_permissions WHERE permission_id = $1",
                    )
                    .bind::<DieselUuid, _>(permission_id)
                    .get_result::<CountRow>(conn)
                    .await
                    .map(|row| row.total)
                })
            })
            .await
    }

    async fn find_with_roles(&self, id: &Uuid) -> diesel::QueryResult<PermissionWithRoles> {
        let permission_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let rows: Vec<PermissionLinkRow> = sql_query(
                        "SELECT p.id AS permission_id, p.code, p.name, p.description, \
                                rp.role_id, r.name AS role_name, rp.effect \
                         FROM permissions p \
                         LEFT JOIN roles_permissions rp ON rp.permission_id = p.id \
                         LEFT JOIN roles r ON r.id = rp.role_id \
                         WHERE p.id = $1",
                    )
                    .bind::<DieselUuid, _>(permission_id)
                    .load(conn)
                    .await?;

                    item_from_rows(rows).ok_or(diesel::result::Error::NotFound)
                })
            })
            .await
    }

    async fn list(
        &self,
        params: &PaginationParams,
        search: Option<String>,
    ) -> diesel::QueryResult<PaginatedResponse<PermissionWithRoles>> {
        let list_params = ListParams::from(params.clone());

        self.base
            .run(move |conn| {
                let search = search.clone();
                Box::pin(async move {
                    let order_sql =
                        order_clause(list_params.sort_by.as_deref(), list_params.sort_direction());
                    let limit = list_params.limit();
                    let offset = list_params.offset();

                    #[derive(diesel::QueryableByName)]
                    struct CountRow {
                        #[diesel(sql_type = diesel::sql_types::BigInt)]
                        total: i64,
                    }

                    // The count measures the permissions, never the joined rows.
                    let total: i64 = sql_query(
                        "SELECT COUNT(DISTINCT p.id) AS total FROM permissions p \
                         WHERE $1::text IS NULL \
                            OR p.code ILIKE '%' || $1 || '%' \
                            OR p.name ILIKE '%' || $1 || '%'",
                    )
                    .bind::<Nullable<Text>, _>(search.clone())
                    .get_result::<CountRow>(conn)
                    .await?
                    .total;

                    let rows: Vec<PermissionLinkRow> = sql_query(format!(
                        "SELECT p.id AS permission_id, p.code, p.name, p.description, \
                                rp.role_id, r.name AS role_name, rp.effect \
                         FROM permissions p \
                         LEFT JOIN roles_permissions rp ON rp.permission_id = p.id \
                         LEFT JOIN roles r ON r.id = rp.role_id \
                         WHERE $1::text IS NULL \
                            OR p.code ILIKE '%' || $1 || '%' \
                            OR p.name ILIKE '%' || $1 || '%' \
                         ORDER BY {order_sql} \
                         LIMIT $2 OFFSET $3"
                    ))
                    .bind::<Nullable<Text>, _>(search)
                    .bind::<BigInt, _>(limit)
                    .bind::<BigInt, _>(offset)
                    .load(conn)
                    .await?;

                    // Group the joined rows back into one item per permission,
                    // keeping the order the query produced.
                    let mut grouped: Vec<Vec<PermissionLinkRow>> = Vec::new();
                    let mut positions: std::collections::HashMap<Uuid, usize> =
                        std::collections::HashMap::new();
                    for row in rows {
                        let position = *positions.entry(row.permission_id).or_insert_with(|| {
                            grouped.push(Vec::new());
                            grouped.len() - 1
                        });
                        grouped[position].push(row);
                    }
                    let items: Vec<PermissionWithRoles> =
                        grouped.into_iter().filter_map(item_from_rows).collect();

                    Ok(PaginatedResponse::new(
                        items,
                        total,
                        list_params.page,
                        list_params.per_page,
                    ))
                })
            })
            .await
    }
}
