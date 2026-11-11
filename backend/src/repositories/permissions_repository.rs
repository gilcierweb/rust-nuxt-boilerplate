use diesel::BoolExpressionMethods;
use diesel::expression_methods::{NullableExpressionMethods, TextExpressionMethods};
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
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

/// One row of the catalog query: a permission and one of its role links (the
/// links are grouped in Rust after the join).
type PermissionLinkRow = (
    Uuid,
    String,
    String,
    Option<String>,
    Option<Uuid>,
    Option<String>,
    Option<i16>,
);

/// Splits a code into its resource and action parts. A code the enums do not
/// name cannot appear (the repository only writes codes built from them), but
/// the API never reports a wrong half.
fn code_parts(code: &str) -> (String, String) {
    match code.split_once(':') {
        Some((resource, action)) => (resource.to_string(), action.to_string()),
        None => (code.to_string(), String::new()),
    }
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
        use crate::db::schema::roles_permissions;

        let permission_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    roles_permissions::table
                        .filter(roles_permissions::permission_id.eq(permission_id))
                        .count()
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn find_with_roles(&self, id: &Uuid) -> diesel::QueryResult<PermissionWithRoles> {
        use crate::db::schema::{permissions, roles, roles_permissions};

        let permission_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    // Left joins: a permission nobody links yet still answers
                    // with an empty role list, which is the case of a just
                    // created permission.
                    let rows: Vec<(
                        Uuid,
                        String,
                        String,
                        Option<String>,
                        Option<Uuid>,
                        Option<String>,
                        Option<i16>,
                    )> = permissions::table
                        .left_join(
                            roles_permissions::table
                                .on(roles_permissions::permission_id.eq(permissions::id)),
                        )
                        .left_join(roles::table.on(roles::id.eq(roles_permissions::role_id)))
                        .filter(permissions::id.eq(permission_id))
                        .select((
                            permissions::id,
                            permissions::code,
                            permissions::name,
                            permissions::description,
                            roles_permissions::role_id.nullable(),
                            roles::name.nullable(),
                            roles_permissions::effect.nullable(),
                        ))
                        .load(conn)
                        .await?;

                    if rows.is_empty() {
                        return Err(diesel::result::Error::NotFound);
                    }
                    let (_, code, name, description, _, _, _) = rows[0].clone();
                    let roles: Vec<PermissionRole> = rows
                        .iter()
                        .filter_map(|(_, _, _, _, role_id, role_name, effect)| {
                            Some(PermissionRole {
                                role_id: (*role_id)?,
                                role_name: role_name.clone()?,
                                effect: (*effect)?,
                            })
                        })
                        .collect();
                    let (resource, action) = code_parts(&code);

                    Ok(PermissionWithRoles {
                        id: permission_id,
                        code,
                        name,
                        description,
                        resource,
                        action,
                        roles,
                    })
                })
            })
            .await
    }

    async fn list(
        &self,
        params: &PaginationParams,
        search: Option<String>,
    ) -> diesel::QueryResult<PaginatedResponse<PermissionWithRoles>> {
        use crate::db::schema::{permissions, roles, roles_permissions};

        let list_params = ListParams::from(params.clone());
        let search = search
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());

        self.base
            .run(move |conn| {
                let search = search.clone();
                Box::pin(async move {
                    // The catalog is small (tens of rows), so the page is loaded
                    // with one join and sorted in memory: a dynamic ORDER BY
                    // would need raw SQL for no gain at this size.
                    let mut query = permissions::table
                        .left_join(
                            roles_permissions::table
                                .on(roles_permissions::permission_id.eq(permissions::id)),
                        )
                        .left_join(roles::table.on(roles::id.eq(roles_permissions::role_id)))
                        .select((
                            permissions::id,
                            permissions::code,
                            permissions::name,
                            permissions::description,
                            roles_permissions::role_id.nullable(),
                            roles::name.nullable(),
                            roles_permissions::effect.nullable(),
                        ))
                        .into_boxed();

                    if let Some(term) = search {
                        let pattern = format!("%{term}%");
                        query = query.filter(
                            permissions::code
                                .like(pattern.clone())
                                .or(permissions::name.like(pattern)),
                        );
                    }

                    let rows: Vec<PermissionLinkRow> = query.load(conn).await?;

                    let mut items: Vec<PermissionWithRoles> = Vec::new();
                    for (permission_id, code, name, description, role_id, role_name, effect) in rows
                    {
                        match items.iter_mut().find(|item| item.id == permission_id) {
                            Some(item) => {
                                if let (Some(role_id), Some(role_name), Some(effect)) =
                                    (role_id, role_name, effect)
                                {
                                    item.roles.push(PermissionRole {
                                        role_id,
                                        role_name,
                                        effect,
                                    });
                                }
                            },
                            None => {
                                let (resource, action) = code_parts(&code);
                                items.push(PermissionWithRoles {
                                    id: permission_id,
                                    code,
                                    name,
                                    description,
                                    resource,
                                    action,
                                    roles: role_id
                                        .map(|role_id| {
                                            vec![PermissionRole {
                                                role_id,
                                                role_name: role_name.unwrap_or_default(),
                                                effect: effect.unwrap_or_default(),
                                            }]
                                        })
                                        .unwrap_or_default(),
                                });
                            },
                        }
                    }

                    // Sort in memory: the allowlist keeps the comparison explicit
                    // and the handler already refused anything outside it.
                    match list_params.sort_by.as_deref() {
                        Some("code") => items.sort_by(|a, b| a.code.cmp(&b.code)),
                        _ => items.sort_by(|a, b| a.name.cmp(&b.name)),
                    }
                    if list_params.sort_direction() == "desc" {
                        items.reverse();
                    }

                    let total = items.len() as i64;
                    let offset = list_params.offset() as usize;
                    let limit = list_params.limit() as usize;
                    let items: Vec<_> = items.into_iter().skip(offset).take(limit).collect();

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
