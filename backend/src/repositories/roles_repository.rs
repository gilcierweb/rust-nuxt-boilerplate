use std::sync::Arc;

use diesel::AggregateExpressionMethods;
use diesel::BoolExpressionMethods;
use diesel::JoinOnDsl;
use diesel::expression_methods::NullableExpressionMethods;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::DBPool;
use crate::db::schema::roles as roles_table;
use crate::models::permission::{NewRolePermissionLink, RolePermissionItem};
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
                    // The catalog is small: load the rows, sort in memory and
                    // paginate, so the sort stays a plain allowlist comparison.
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
        use crate::db::schema::{permissions, roles_permissions};

        let role_id = *role_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let rows: Vec<(String, i16)> = roles_permissions::table
                        .inner_join(permissions::table)
                        .filter(roles_permissions::role_id.eq(role_id))
                        .select((permissions::code, roles_permissions::effect))
                        .order_by(permissions::code.asc())
                        .load(conn)
                        .await?;

                    Ok(rows
                        .into_iter()
                        .map(|(code, effect)| RolePermissionItem { code, effect })
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
        use crate::db::schema::{profiles, users, users_roles};

        let role_id = *role_id;
        self.base
            .run(move |conn| {
                let security = security.clone();
                Box::pin(async move {
                    let rows: Vec<(
                        Uuid,
                        Vec<u8>,
                        i32,
                        Option<String>,
                        Option<String>,
                        Option<String>,
                    )> = users_roles::table
                        .inner_join(users::table)
                        // A profile is optional, so the join names its own ON
                        // clause: the link table carries no relation to it.
                        .left_join(profiles::table.on(profiles::user_id.eq(users::id)))
                        .filter(users_roles::role_id.eq(role_id))
                        .select((
                            users::id,
                            users::email_encrypted,
                            users::encryption_key_version,
                            // A left joined column is nullable, so the
                            // select clause has to say so.
                            profiles::first_name.nullable(),
                            profiles::last_name.nullable(),
                            profiles::full_name.nullable(),
                        ))
                        .order_by(users::created_at.asc())
                        .load(conn)
                        .await?;

                    let mut members = Vec::with_capacity(rows.len());
                    for (id, email_encrypted, key_version, first_name, last_name, full_name) in rows
                    {
                        let email = security
                            .decrypt_email_fields(&email_encrypted, key_version)
                            .map_err(|error| {
                                diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::Unknown,
                                    Box::new(error.to_string()),
                                )
                            })?;

                        members.push(RoleMemberItem {
                            id,
                            email,
                            display_name: display_name(
                                first_name.as_deref(),
                                last_name.as_deref(),
                                full_name.as_deref(),
                            ),
                        });
                    }

                    Ok(members)
                })
            })
            .await
    }

    async fn role_ids_holding_all_codes(&self, codes: &[String]) -> diesel::QueryResult<Vec<Uuid>> {
        use crate::db::schema::{permissions, roles, roles_permissions};

        let codes = codes.to_vec();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    roles_permissions::table
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
                        .load(conn)
                        .await
                })
            })
            .await
    }

    async fn replace_permission_links(
        &self,
        role_id: &Uuid,
        items: &[RolePermissionItem],
    ) -> diesel::QueryResult<usize> {
        use crate::db::schema::{permissions, roles_permissions};

        let role_id = *role_id;
        let items = items.to_vec();
        // An unmapped effect never reaches the database: the handler refuses it
        // before calling here, and the fail closed model keeps it impossible.
        if items.iter().any(|item| item.effect().is_none()) {
            return Err(diesel::result::Error::QueryBuilderError(
                "unknown permission effect".into(),
            ));
        }

        self.base
            .run_transaction(move |conn| {
                let items = items.clone();
                Box::pin(async move {
                    diesel::delete(roles_permissions::table)
                        .filter(roles_permissions::role_id.eq(role_id))
                        .execute(conn)
                        .await?;

                    if items.is_empty() {
                        return Ok(0);
                    }

                    // One query resolves the id of every code, so the insert
                    // itself is a single batch statement.
                    let codes: Vec<String> = items.iter().map(|item| item.code.clone()).collect();
                    let resolved: Vec<(Uuid, String)> = permissions::table
                        .filter(permissions::code.eq_any(&codes))
                        .select((permissions::id, permissions::code))
                        .load(conn)
                        .await?;

                    let links: Vec<NewRolePermissionLink> = resolved
                        .iter()
                        .filter_map(|(permission_id, code)| {
                            items.iter().find(|item| &item.code == code).map(|item| {
                                NewRolePermissionLink {
                                    role_id,
                                    permission_id: *permission_id,
                                    effect: item.effect,
                                }
                            })
                        })
                        .collect();

                    diesel::insert_into(roles_permissions::table)
                        .values(&links)
                        .execute(conn)
                        .await?;

                    Ok(links.len())
                })
            })
            .await
    }
}
