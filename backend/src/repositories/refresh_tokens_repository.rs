use crate::DBPool;
use crate::db::schema::refresh_tokens as refresh_tokens_table;
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::refresh_tokens_trait::IRefreshTokenRepository;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
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
        self.base
            .run(|conn| {
                Box::pin(async move {
                    refresh_tokens_table::table
                        .select(RefreshToken::as_select())
                        .load::<RefreshToken>(conn)
                        .await
                })
            })
            .await
    }

    async fn find(&self, tid: &Uuid) -> diesel::QueryResult<RefreshToken> {
        let tid = *tid;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    refresh_tokens_table::table
                        .find(tid)
                        .select(RefreshToken::as_select())
                        .first::<RefreshToken>(conn)
                        .await
                })
            })
            .await
    }

    async fn create(&self, item: &NewRefreshToken) -> diesel::QueryResult<RefreshToken> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(refresh_tokens_table::table)
                        .values(&item)
                        .returning(RefreshToken::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, tid: &Uuid) -> diesel::QueryResult<usize> {
        let tid = *tid;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(refresh_tokens_table::table.find(tid))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_token_hash(
        &self,
        token_hash_str: &str,
    ) -> diesel::QueryResult<Option<RefreshToken>> {
        let hash = token_hash_str.to_string();
        use crate::db::schema::refresh_tokens::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    refresh_tokens_table::table
                        .filter(token_hash.eq(hash))
                        .select(RefreshToken::as_select())
                        .first::<RefreshToken>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_valid_by_token(
        &self,
        plaintext_token: &str,
    ) -> diesel::QueryResult<Option<RefreshToken>> {
        use crate::services::token_service::verify_token_hash;
        use chrono::Utc;
        use crate::db::schema::refresh_tokens::dsl::*;

        let token = plaintext_token.to_string();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let now = Utc::now();
                    let candidates: Vec<RefreshToken> = refresh_tokens_table::table
                        .filter(revoked_at.is_null())
                        .filter(expires_at.gt(now))
                        .select(RefreshToken::as_select())
                        .load::<RefreshToken>(conn)
                        .await?;

                    Ok(candidates
                        .into_iter()
                        .find(|t| verify_token_hash(&token, &t.token_hash)))
                })
            })
            .await
    }

    async fn revoke(&self, tid: &Uuid) -> diesel::QueryResult<usize> {
        let tid = *tid;
        use crate::db::schema::refresh_tokens::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(refresh_tokens_table::table.find(tid))
                        .set(revoked_at.eq(Some(chrono::Utc::now())))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn revoke_all_for_user(&self, uid: &Uuid) -> diesel::QueryResult<usize> {
        let uid = *uid;
        use crate::db::schema::refresh_tokens::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(refresh_tokens_table::table.filter(user_id.eq(uid)))
                        .set(revoked_at.eq(Some(chrono::Utc::now())))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn rotate_token(
        &self,
        token_hash_str: &str,
        new_token: &NewRefreshToken,
    ) -> diesel::QueryResult<Option<RefreshToken>> {
        let plaintext = token_hash_str.to_string();
        let new_token = new_token.clone();
        use crate::db::schema::refresh_tokens::dsl::*;
        use crate::services::token_service::verify_token_hash;
        use chrono::Utc;

        self.base
            .run_transaction(move |conn| {
                Box::pin(async move {
                    let now = Utc::now();

                    let candidates: Vec<RefreshToken> = refresh_tokens_table::table
                        .filter(revoked_at.is_null())
                        .filter(expires_at.gt(now))
                        .select(RefreshToken::as_select())
                        .load::<RefreshToken>(conn)
                        .await?;

                    let existing_token = candidates
                        .into_iter()
                        .find(|t| verify_token_hash(&plaintext, &t.token_hash));

                    let existing_token = match existing_token {
                        Some(t) => t,
                        None => return Ok(None),
                    };

                    diesel::update(refresh_tokens_table::table.find(existing_token.id))
                        .set(revoked_at.eq(Some(Utc::now())))
                        .execute(conn)
                        .await?;

                    let created_token: RefreshToken =
                        diesel::insert_into(refresh_tokens_table::table)
                            .values(&new_token)
                            .returning(RefreshToken::as_returning())
                            .get_result(conn)
                            .await?;

                    Ok(Some(created_token))
                })
            })
            .await
    }
}
