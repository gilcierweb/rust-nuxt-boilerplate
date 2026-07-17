use crate::DBPool;
use crate::db::schema::users_roles as users_roles_table;
use crate::models::user_role::{NewUserRole, UserRole};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::user_roles_trait::IUserRoleRepository;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct UserRolesRepository {
    base: BaseRepo,
}

impl UserRolesRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::user_roles_trait::MockIUserRoleRepository;

#[async_trait::async_trait]
impl IUserRoleRepository for UserRolesRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<UserRole>> {
        self.base
            .run(|conn| {
                Box::pin(async move { users_roles_table::table.load::<UserRole>(conn).await })
            })
            .await
    }

    async fn find(&self, user: &Uuid, role: &Uuid) -> diesel::QueryResult<UserRole> {
        let user = *user;
        let role = *role;
        use crate::db::schema::users_roles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_roles_table::table
                        .filter(user_id.eq(user))
                        .filter(role_id.eq(role))
                        .first::<UserRole>(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_user(&self, uid: &Uuid) -> diesel::QueryResult<Vec<UserRole>> {
        let uid = *uid;
        use crate::db::schema::users_roles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_roles_table::table
                        .filter(user_id.eq(uid))
                        .load::<UserRole>(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_role(&self, rid: &Uuid) -> diesel::QueryResult<Vec<UserRole>> {
        let rid = *rid;
        use crate::db::schema::users_roles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_roles_table::table
                        .filter(role_id.eq(rid))
                        .load::<UserRole>(conn)
                        .await
                })
            })
            .await
    }

    async fn create(&self, item: &NewUserRole) -> diesel::QueryResult<UserRole> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(users_roles_table::table)
                        .values(&item)
                        .returning(UserRole::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, user: &Uuid, role: &Uuid) -> diesel::QueryResult<usize> {
        let user = *user;
        let role = *role;
        use crate::db::schema::users_roles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(
                        users_roles_table::table
                            .filter(user_id.eq(user))
                            .filter(role_id.eq(role)),
                    )
                    .execute(conn)
                    .await
                })
            })
            .await
    }
}
