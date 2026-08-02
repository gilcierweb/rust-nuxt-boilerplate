use async_trait::async_trait;
use diesel::QueryResult;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::schema::magic_link_tokens as table;
use crate::models::magic_link_token::{MagicLinkToken, NewMagicLinkToken};
use crate::repositories::base::BaseRepo;

pub use crate::repositories::traits::magic_link_token_trait::IMagicLinkTokenRepository;

#[cfg(test)]
pub use crate::repositories::traits::magic_link_token_trait::MockIMagicLinkTokenRepository;

pub struct MagicLinkTokenRepository {
    base: BaseRepo,
}

impl MagicLinkTokenRepository {
    pub fn new(pool: crate::db::database::DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[async_trait]
impl IMagicLinkTokenRepository for MagicLinkTokenRepository {
    async fn create(&self, item: &NewMagicLinkToken) -> QueryResult<MagicLinkToken> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(table::table)
                        .values(item)
                        .returning(MagicLinkToken::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_digest(&self, token_digest: &str) -> QueryResult<Option<MagicLinkToken>> {
        let digest = token_digest.to_string();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    table::table
                        .filter(table::token_digest.eq(digest))
                        .select(MagicLinkToken::as_select())
                        .first::<MagicLinkToken>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn mark_consumed(&self, id: &Uuid) -> QueryResult<usize> {
        let id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(table::table.find(id))
                        .set(table::consumed_at.eq(chrono::Utc::now().naive_utc()))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn delete_expired(&self) -> QueryResult<usize> {
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(
                        table::table
                            .filter(table::expires_at.lt(chrono::Utc::now().naive_utc()))
                            .filter(table::consumed_at.is_null()),
                    )
                    .execute(conn)
                    .await
                })
            })
            .await
    }
}
