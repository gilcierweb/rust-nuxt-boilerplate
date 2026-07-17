use crate::DBPool;
use crate::db::schema::refresh_tokens as refresh_tokens_table;
use crate::db::schema::refresh_tokens::dsl::{expires_at, revoked_at};
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::refresh_tokens_trait::IRefreshTokenRepository;
use crate::services::token_service::{generate_random_token, hash_token, verify_token_hash};
use chrono::Utc;
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
        use crate::db::schema::refresh_tokens::dsl::*;
        use crate::services::token_service::verify_token_hash;
        use chrono::Utc;

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
        old_token_plaintext: &str,
        expires_in_secs: i64,
        hash_salt: &str,
    ) -> diesel::QueryResult<Option<(RefreshToken, String)>> {
        let plaintext = old_token_plaintext.to_string();
        let expires_secs = expires_in_secs;
        let salt = hash_salt.to_string();

        self.base
            .run_transaction(move |conn| {
                Box::pin(async move {
                    let now = Utc::now();

                    // Find the old valid token by verifying against stored hashes
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

                    // Immediately revoke the old token
                    diesel::update(refresh_tokens_table::table.find(existing_token.id))
                        .set(revoked_at.eq(Some(now)))
                        .execute(conn)
                        .await?;

                    // Generate new plain token and hash it with the provided salt
                    let new_token_plain = generate_random_token(48);
                    let new_token_hash = hash_token(&new_token_plain, &salt);
                    let new_expires_at = now + chrono::Duration::seconds(expires_secs);

                    // Create new token preserving user_id, device_info, and ip_address from old token
                    let new_token = NewRefreshToken {
                        id: Uuid::new_v4(),
                        user_id: existing_token.user_id,
                        token_hash: new_token_hash,
                        device_info: existing_token.device_info,
                        ip_address: existing_token.ip_address,
                        expires_at: new_expires_at,
                        created_at: now,
                        updated_at: now,
                    };

                    let created_token: RefreshToken =
                        diesel::insert_into(refresh_tokens_table::table)
                            .values(&new_token)
                            .returning(RefreshToken::as_returning())
                            .get_result(conn)
                            .await?;

                    Ok(Some((created_token, new_token_plain)))
                })
            })
            .await
    }
}
