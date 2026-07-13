/// Macro that generates CRUD implementations for a repository wrapper containing a `BaseRepo`.
///
/// Usage:
/// ```
/// impl_crud!(UsersRepository, base, IUserRepository, User, NewUser, users::table);
/// ```
///
/// This generates implementations for: `all()`, `find()`, `create()`, `update()`, `destroy()`.
#[macro_export]
macro_rules! impl_crud {
    ($repo:ty, $field:ident, $trait_name:ident, $model:ty, $new_model:ty, $table:expr) => {
        #[async_trait::async_trait]
        impl $trait_name for $repo {
            async fn all(&self) -> diesel::QueryResult<Vec<$model>> {
                use diesel::RunQueryDsl;
                self.$field.run(|conn| $table.load::<$model>(conn)).await
            }

            async fn find(&self, id: &uuid::Uuid) -> diesel::QueryResult<$model> {
                use diesel::{QueryDsl, RunQueryDsl};
                let id = *id;
                self.$field
                    .run(move |conn| $table.find(id).first::<$model>(conn))
                    .await
            }

            async fn create(&self, item: &$new_model) -> diesel::QueryResult<$model> {
                use diesel::RunQueryDsl;
                let item = item.clone();
                self.$field
                    .run(move |conn| diesel::insert_into($table).values(&item).get_result(conn))
                    .await
            }

            async fn update(
                &self,
                id: &uuid::Uuid,
                item: &$new_model,
            ) -> diesel::QueryResult<$model> {
                use diesel::{QueryDsl, RunQueryDsl};
                let item = item.clone();
                let id = *id;
                self.$field
                    .run(move |conn| diesel::update($table.find(id)).set(&item).get_result(conn))
                    .await
            }

            async fn destroy(&self, id: &uuid::Uuid) -> diesel::QueryResult<usize> {
                use diesel::{QueryDsl, RunQueryDsl};
                let id = *id;
                self.$field
                    .run(move |conn| diesel::delete($table.find(id)).execute(conn))
                    .await
            }
        }
    };
}
