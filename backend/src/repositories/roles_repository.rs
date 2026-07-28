use crate::DBPool;
use crate::db::schema::roles as roles_table;
use crate::models::role::{NewRole, Role};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::roles_trait::IRoleRepository;
use crate::utils::pagination::{ListParams, PaginatedResponse, PaginationParams};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

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
                Box::pin(async move { roles_table::table.find(id).first::<Role>(conn).await })
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
}
