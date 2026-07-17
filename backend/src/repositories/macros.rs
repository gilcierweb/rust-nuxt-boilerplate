/// Macro that generates CRUD implementations for a repository wrapper containing a `BaseRepo`.
#[macro_export]
macro_rules! impl_crud {
    ($repo:ty, $field:ident, $trait_name:ident, $model:ty, $new_model:ty, $table:expr) => {
        #[async_trait::async_trait]
        impl $trait_name for $repo {
            async fn all(&self) -> diesel::result::QueryResult<Vec<$model>> {
                use diesel::QueryDsl;
                use diesel::SelectableHelper;
                use diesel_async::RunQueryDsl;
                self.$field
                    .run(|conn| Box::pin(async move { $table.load::<$model>(conn).await }))
                    .await
            }

            async fn find(&self, id: &uuid::Uuid) -> diesel::result::QueryResult<$model> {
                use diesel::QueryDsl;
                use diesel::SelectableHelper;
                use diesel_async::RunQueryDsl;
                let id = *id;
                self.$field
                    .run(move |conn| {
                        Box::pin(async move { $table.find(id).first::<$model>(conn).await })
                    })
                    .await
            }

            async fn create(&self, item: &$new_model) -> diesel::result::QueryResult<$model> {
                use diesel::SelectableHelper;
                use diesel_async::RunQueryDsl;
                let item = item.clone();
                self.$field
                    .run(move |conn| {
                        Box::pin(async move {
                            diesel::insert_into($table)
                                .values(&item)
                                .get_result(conn)
                                .await
                        })
                    })
                    .await
            }

            async fn update(
                &self,
                id: &uuid::Uuid,
                item: &$new_model,
            ) -> diesel::result::QueryResult<$model> {
                use diesel::QueryDsl;
                use diesel::SelectableHelper;
                use diesel_async::RunQueryDsl;
                let item = item.clone();
                let id = *id;
                self.$field
                    .run(move |conn| {
                        Box::pin(async move {
                            diesel::update($table.find(id))
                                .set(&item)
                                .get_result(conn)
                                .await
                        })
                    })
                    .await
            }

            async fn destroy(&self, id: &uuid::Uuid) -> diesel::result::QueryResult<usize> {
                use diesel::QueryDsl;
                use diesel_async::RunQueryDsl;
                let id = *id;
                self.$field
                    .run(move |conn| {
                        Box::pin(async move { diesel::delete($table.find(id)).execute(conn).await })
                    })
                    .await
            }
        }
    };
}
