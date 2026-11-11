use std::sync::Arc;

use diesel::sql_query;
use diesel::sql_types::{Array, Nullable, Text, Uuid as DieselUuid};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::DBPool;
use crate::db::schema::roles as roles_table;
use crate::models::permission::RolePermissionItem;
use crate::models::role::{NewRole, Role};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::roles_trait::{IRoleRepository, RoleMemberItem};
use crate::security::SecurityService;
use crate::utils::pagination::{ListParams, PaginatedResponse, PaginationParams};

pub struct RolesRepository {
    base: BaseRepo,
}

impl RolesRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::roles_trait::MockIRoleRepository;

#[derive(diesel::QueryableByName)]
struct PermissionLinkRow {
    #[diesel(sql_type = Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Int2)]
    effect: i16,
}

#[derive(diesel::QueryableByName)]
struct RoleIdRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
}

/// One row of the member query: the user, the encrypted email and the profile
/// fields the display name is built from.
#[derive(diesel::QueryableByName)]
struct MemberRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Bytea)]
    email_encrypted: Vec<u8>,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    encryption_key_version: i32,
    #[diesel(sql_type = Nullable<Text>)]
    first_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    last_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    full_name: Option<String>,
}

/// The display name of a member: the full name, else first and last joined.
fn display_name(
    first_name: Option<&str>,
    last_name: Option<&str>,
    full_name: Option<&str>,
) -> Option<String> {
    full_name.map(str::to_string).or_else(|| {
        let parts: Vec<&str> = [first_name, last_name]
            .into_iter()
            .flatten()
            .filter(|part| !part.is_empty())
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    })
}

#[async_trait::async_trait]
impl IRoleRepository for RolesRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<Role>> {
        self.base
            .run(|conn| Box::pin(async move { roles_table::table.load::<Role>(conn).await }))
            .await
    }

    async fn find(&self, id: &Uuid) -> diesel::QueryResult<Role> {
        let id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    roles_table::table
                        .find(id)
                        .select(Role::as_select())
                        .first::<Role>(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_name(&self, name: &str) -> diesel::QueryResult<Option<Role>> {
        let name = name.to_string();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    roles_table::table
                        .filter(roles_table::dsl::name.eq(name))
                        .select(Role::as_select())
                        .first::<Role>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn create(&self, item: &NewRole) -> diesel::QueryResult<Role> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(roles_table::table)
                        .values(&item)
                        .returning(Role::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn update(&self, id: &Uuid, item: &NewRole) -> diesel::QueryResult<Role> {
        let item = item.clone();
        let role_id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::roles::dsl::*;
                    diesel::update(roles_table::table.find(role_id))
                        .set((
                            name.eq(item.name),
                            resource_type.eq(item.resource_type),
                            resource_id.eq(item.resource_id),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .returning(Role::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, id: &Uuid) -> diesel::QueryResult<usize> {
        let id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(roles_table::table.find(id))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn list_paginated(
        &self,
        params: &PaginationParams,
    ) -> diesel::QueryResult<PaginatedResponse<Role>> {
        let list_params = ListParams::from(params.clone());

        self.base
            .run(move |conn| {
                Box::pin(async move {
                    // Load all data then sort in memory
                    let mut data = roles_table::table.load::<Role>(conn).await?;

                    if let Some(sort_by) = list_params.sort_by.as_deref() {
                        let desc = list_params.sort_dir.as_deref() == Some("desc");
                        data.sort_by(|a, b| {
                            let ord = match sort_by {
                                "name" => a.name.cmp(&b.name),
                                "resource_type" => a.resource_type.cmp(&b.resource_type),
                                "created_at" => a.created_at.cmp(&b.created_at),
                                "updated_at" => a.updated_at.cmp(&b.updated_at),
                                "id" => a.id.cmp(&b.id),
                                _ => a.name.cmp(&b.name),
                            };
                            if desc { ord.reverse() } else { ord }
                        });
                    } else {
                        data.sort_by_key(|a| a.name.clone());
                    }

                    // Apply pagination
                    let total_count = data.len() as i64;
                    let offset = list_params.offset() as usize;
                    let limit = list_params.limit() as usize;
                    let data: Vec<_> = data.into_iter().skip(offset).take(limit).collect();

                    Ok(PaginatedResponse::new(
                        data,
                        total_count,
                        list_params.page,
                        list_params.per_page,
                    ))
                })
            })
            .await
    }

    async fn list_permission_links(
        &self,
        role_id: &Uuid,
    ) -> diesel::QueryResult<Vec<RolePermissionItem>> {
        let role_id = *role_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let rows: Vec<PermissionLinkRow> = sql_query(
                        "SELECT p.code, rp.effect \
                         FROM roles_permissions rp \
                         JOIN permissions p ON p.id = rp.permission_id \
                         WHERE rp.role_id = $1 \
                         ORDER BY p.code",
                    )
                    .bind::<DieselUuid, _>(role_id)
                    .load(conn)
                    .await?;

                    Ok(rows
                        .into_iter()
                        .map(|row| RolePermissionItem {
                            code: row.code,
                            effect: row.effect,
                        })
                        .collect())
                })
            })
            .await
    }

    async fn list_members(
        &self,
        role_id: &Uuid,
        security: Arc<SecurityService>,
    ) -> diesel::QueryResult<Vec<RoleMemberItem>> {
        let role_id = *role_id;
        self.base
            .run(move |conn| {
                let security = security.clone();
                Box::pin(async move {
                    let rows: Vec<MemberRow> = sql_query(
                        "SELECT u.id, u.email_encrypted, u.encryption_key_version, \
                                p.first_name, p.last_name, p.full_name, p.nickname \
                         FROM users_roles ur \
                         JOIN users u ON u.id = ur.user_id \
                         LEFT JOIN profiles p ON p.user_id = u.id \
                         WHERE ur.role_id = $1 \
                         ORDER BY u.created_at",
                    )
                    .bind::<DieselUuid, _>(role_id)
                    .load(conn)
                    .await?;

                    let mut members = Vec::with_capacity(rows.len());
                    for row in rows {
                        let email = security
                            .decrypt_email_fields(&row.email_encrypted, row.encryption_key_version)
                            .map_err(|error| {
                                diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::Unknown,
                                    Box::new(error.to_string()),
                                )
                            })?;

                        members.push(RoleMemberItem {
                            id: row.id,
                            email,
                            display_name: display_name(
                                row.first_name.as_deref(),
                                row.last_name.as_deref(),
                                row.full_name.as_deref(),
                            ),
                        });
                    }

                    Ok(members)
                })
            })
            .await
    }

    async fn role_ids_holding_all_codes(&self, codes: &[String]) -> diesel::QueryResult<Vec<Uuid>> {
        let codes = codes.to_vec();
        let expected = codes.len() as i64;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let rows: Vec<RoleIdRow> = sql_query(
                        "SELECT rp.role_id AS id \
                         FROM roles_permissions rp \
                         JOIN permissions p ON p.id = rp.permission_id \
                         JOIN roles r ON r.id = rp.role_id \
                         WHERE rp.effect = 0 AND p.code = ANY($1) AND r.resource_id IS NULL \
                         GROUP BY rp.role_id \
                         HAVING count(DISTINCT p.code) = $2",
                    )
                    .bind::<Array<Text>, _>(&codes)
                    .bind::<diesel::sql_types::BigInt, _>(expected)
                    .load(conn)
                    .await?;

                    Ok(rows.into_iter().map(|row| row.id).collect())
                })
            })
            .await
    }

    async fn replace_permission_links(
        &self,
        role_id: &Uuid,
        items: &[RolePermissionItem],
    ) -> diesel::QueryResult<usize> {
        let role_id = *role_id;
        let items = items.to_vec();

        self.base
            .run_transaction(move |conn| {
                let items = items.clone();
                Box::pin(async move {
                    diesel::sql_query("DELETE FROM roles_permissions WHERE role_id = $1")
                        .bind::<DieselUuid, _>(role_id)
                        .execute(conn)
                        .await?;

                    for item in &items {
                        let effect = item.effect();
                        // An unmapped effect never reaches the database: the
                        // handler refuses it before calling here.
                        if effect.is_none() {
                            return Err(diesel::result::Error::QueryBuilderError(
                                "unknown permission effect".into(),
                            ));
                        }

                        diesel::sql_query(
                            "INSERT INTO roles_permissions (role_id, permission_id, effect, created_at) \
                             SELECT $1, p.id, $3, NOW() FROM permissions p WHERE p.code = $2",
                        )
                        .bind::<DieselUuid, _>(role_id)
                        .bind::<Text, _>(item.code.clone())
                        .bind::<diesel::sql_types::Int2, _>(item.effect)
                        .execute(conn)
                        .await?;
                    }

                    Ok(items.len())
                })
            })
            .await
    }
}
