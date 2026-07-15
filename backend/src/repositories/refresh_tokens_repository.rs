use crate::DBPool;
use crate::db::schema::refresh_tokens as refresh_tokens_table;
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::refresh_tokens_trait::IRefreshTokenRepository;
use diesel::prelude::*;
use uuid::Uuid;

pub struct RefreshTokensRepository {
    base: BaseRepo,
}

impl RefreshTokensRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::refresh_tokens_trait::MockIRefreshTokenRepository;

#[async_trait::async_trait]
impl IRefreshTokenRepository for RefreshTokensRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<RefreshToken>> {
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(|conn| {
                refresh_tokens_table::table
                    .select(RefreshToken::as_select())
                    .load::<RefreshToken>(conn)
            })
            .await
    }

    async fn find(&self, tid: &Uuid) -> diesel::QueryResult<RefreshToken> {
        let tid = *tid;
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(move |conn| {
                refresh_tokens_table::table
                    .find(tid)
                    .select(RefreshToken::as_select())
                    .first::<RefreshToken>(conn)
            })
            .await
    }

    async fn create(&self, item: &NewRefreshToken) -> diesel::QueryResult<RefreshToken> {
        use diesel::{RunQueryDsl, SelectableHelper};
        let item = item.clone();
        self.base
            .run(move |conn| {
                diesel::insert_into(refresh_tokens_table::table)
                    .values(&item)
                    .returning(RefreshToken::as_returning())
                    .get_result(conn)
            })
            .await
    }

    async fn destroy(&self, tid: &Uuid) -> diesel::QueryResult<usize> {
        let tid = *tid;
        use diesel::{QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| diesel::delete(refresh_tokens_table::table.find(tid)).execute(conn))
            .await
    }

    async fn find_by_token_hash(
        &self,
        token_hash_str: &str,
    ) -> diesel::QueryResult<Option<RefreshToken>> {
        let hash = token_hash_str.to_string();
        use crate::db::schema::refresh_tokens::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        self.base
            .run(move |conn| {
                refresh_tokens_table::table
                    .filter(token_hash.eq(hash))
                    .select(RefreshToken::as_select())
                    .first::<RefreshToken>(conn)
                    .optional()
            })
            .await
    }

    async fn revoke(&self, tid: &Uuid) -> diesel::QueryResult<usize> {
        let tid = *tid;
        use crate::db::schema::refresh_tokens::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(refresh_tokens_table::table.find(tid))
                    .set(revoked_at.eq(Some(chrono::Utc::now())))
                    .execute(conn)
            })
            .await
    }

    async fn revoke_all_for_user(&self, uid: &Uuid) -> diesel::QueryResult<usize> {
        let uid = *uid;
        use crate::db::schema::refresh_tokens::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(refresh_tokens_table::table.filter(user_id.eq(uid)))
                    .set(revoked_at.eq(Some(chrono::Utc::now())))
                    .execute(conn)
            })
            .await
    }

    async fn rotate_token(
        &self,
        token_hash_str: &str,
        new_token: &NewRefreshToken,
    ) -> diesel::QueryResult<Option<RefreshToken>> {
        let hash = token_hash_str.to_string();
        let new_token = new_token.clone();
        use crate::db::schema::refresh_tokens::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

        // Use a transaction to ensure atomicity: revoke old token AND create new token
        self.base
            .run_transaction(move |conn| {
                // First, find the existing token
                let existing_token: Option<RefreshToken> = refresh_tokens_table::table
                    .filter(token_hash.eq(&hash))
                    .select(RefreshToken::as_select())
                    .first::<RefreshToken>(conn)
                    .optional()?;

                let existing_token = match existing_token {
                    Some(t) => t,
                    None => return Ok(None),
                };

                // Check if token is already revoked or expired
                if existing_token.revoked_at.is_some() {
                    return Ok(None);
                }
                if existing_token.expires_at < chrono::Utc::now() {
                    return Ok(None);
                }

                // Revoke the old token
                diesel::update(refresh_tokens_table::table.find(existing_token.id))
                    .set(revoked_at.eq(Some(chrono::Utc::now())))
                    .execute(conn)?;

                // Create the new token
                let created_token: RefreshToken = diesel::insert_into(refresh_tokens_table::table)
                    .values(&new_token)
                    .returning(RefreshToken::as_returning())
                    .get_result(conn)?;

                Ok(Some(created_token))
            })
            .await
    }
}