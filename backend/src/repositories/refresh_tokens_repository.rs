use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::DBPool;
use crate::DbPoolExt;
use crate::db::schema::refresh_tokens as refresh_tokens_table;
use crate::db::schema::refresh_tokens::dsl::{expires_at, revoked_at};
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
pub use crate::repositories::traits::refresh_tokens_trait::IRefreshTokenRepository;
use crate::services::token_service::{generate_random_token, hash_token};

pub struct RefreshTokensRepository {
    db: DBPool,
}

impl RefreshTokensRepository {
    pub fn new(pool: DBPool) -> Self {
        Self { db: pool }
    }
}

/// The rotation grace window.
///
/// A token revoked more than this ago belongs to a lineage that already
/// rotated legitimately long before: presenting it now says nothing about a
/// compromise and only the normal rejection applies. Inside the window the
/// row was rotated recently, so the old plaintext coming back is either a
/// client race between two tabs or a thief replaying a stolen cookie. The
/// two are indistinguishable, and both cost the family everything.
pub const ROTATION_GRACE_SECS: i64 = 3600;

/// What a rotation request found.
#[derive(Debug, Clone)]
pub enum RotationOutcome {
    /// The presented token rotated: the old row is revoked and a new one
    /// created.
    Rotated {
        token: RefreshToken,
        plaintext: String,
    },
    /// The presented token matches a row revoked inside the grace window:
    /// every refresh token of the named user is already revoked by the
    /// rotation itself, and the caller must also invalidate the user's
    /// access tokens.
    ReuseDetected { user_id: Uuid },
    /// No row matches the presented token: unknown or an expired lineage.
    NotFound,
}

/// Whether the matched row is a token revoked inside the grace window, the
/// situation that reads as a reuse.
pub fn matched_token_is_reuse(matched: &RefreshToken, now: OffsetDateTime) -> bool {
    match matched.revoked_at {
        Some(revoked_instant) => (now - revoked_instant).whole_seconds() <= ROTATION_GRACE_SECS,
        None => false,
    }
}

#[cfg(test)]
pub use crate::repositories::traits::refresh_tokens_trait::MockIRefreshTokenRepository;

#[async_trait::async_trait]
impl IRefreshTokenRepository for RefreshTokensRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<RefreshToken>> {
        self.db
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
        self.db
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
        self.db
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
        self.db
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
        self.db
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
        hash_key: &str,
    ) -> diesel::QueryResult<Option<RefreshToken>> {
        use crate::db::schema::refresh_tokens::dsl::*;
        use crate::services::token_service::hash_token;

        let token = plaintext_token.to_string();
        let key = hash_key.to_string();
        self.db
            .run(move |conn| {
                Box::pin(async move {
                    let now = OffsetDateTime::now_utc();

                    // Fast path: HMAC tokens are deterministic — compute hash and lookup directly
                    let hmac_hash = hash_token(&token, &key);
                    if let Ok(Some(found)) = refresh_tokens_table::table
                        .filter(revoked_at.is_null())
                        .filter(expires_at.gt(now))
                        .filter(token_hash.eq(&hmac_hash))
                        .select(RefreshToken::as_select())
                        .first::<RefreshToken>(conn)
                        .await
                        .optional()
                    {
                        return Ok(Some(found));
                    }

                    // Slow path: legacy Argon2id tokens — iterate and verify
                    let candidates: Vec<RefreshToken> = refresh_tokens_table::table
                        .filter(revoked_at.is_null())
                        .filter(expires_at.gt(now))
                        .select(RefreshToken::as_select())
                        .load::<RefreshToken>(conn)
                        .await?;

                    use crate::services::token_service::verify_token_hash;
                    Ok(candidates
                        .into_iter()
                        .find(|t| verify_token_hash(&token, &t.token_hash, &key)))
                })
            })
            .await
    }

    async fn revoke(&self, tid: &Uuid) -> diesel::QueryResult<usize> {
        let tid = *tid;
        use crate::db::schema::refresh_tokens::dsl::*;
        self.db
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(refresh_tokens_table::table.find(tid))
                        .set(revoked_at.eq(Some(OffsetDateTime::now_utc())))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn revoke_all_for_user(&self, uid: &Uuid) -> diesel::QueryResult<usize> {
        let uid = *uid;
        use crate::db::schema::refresh_tokens::dsl::*;
        self.db
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(refresh_tokens_table::table.filter(user_id.eq(uid)))
                        .set(revoked_at.eq(Some(OffsetDateTime::now_utc())))
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
    ) -> diesel::QueryResult<RotationOutcome> {
        let plaintext = old_token_plaintext.to_string();
        let expires_secs = expires_in_secs;
        let salt = hash_salt.to_string();
        let key = hash_salt.to_string();

        self.db
            .run_transaction(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::refresh_tokens::dsl::{
                        token_hash as token_hash_col, user_id,
                    };

                    let now = OffsetDateTime::now_utc();
                    let grace_start = now - Duration::seconds(ROTATION_GRACE_SECS);

                    // Refresh tokens are stored as HMAC-SHA256 hashes, which are
                    // deterministic for a given plaintext and salt, so the matching
                    // row (active or revoked inside the grace window) can be
                    // found and locked directly. There is no legacy Argon2id
                    // path here: `verify_token_hash` rejects any stored hash
                    // without the `hmac:` prefix, so a scan could never match.
                    let hmac_hash = hash_token(&plaintext, &key);
                    let existing_token: Option<RefreshToken> = refresh_tokens_table::table
                        .filter(token_hash_col.eq(&hmac_hash))
                        .filter(expires_at.gt(now))
                        .filter(revoked_at.is_null().or(revoked_at.gt(grace_start)))
                        .select(RefreshToken::as_select())
                        .for_update()
                        .first::<RefreshToken>(conn)
                        .await
                        .optional()?;

                    let existing_token = match existing_token {
                        Some(token) => token,
                        None => return Ok(RotationOutcome::NotFound),
                    };

                    // A row revoked inside the grace window means the lineage
                    // already rotated: the old plaintext coming back is a
                    // replay, so the whole family dies here, atomically.
                    if matched_token_is_reuse(&existing_token, now) {
                        diesel::update(
                            refresh_tokens_table::table.filter(user_id.eq(existing_token.user_id)),
                        )
                        .set(revoked_at.eq(Some(now)))
                        .execute(conn)
                        .await?;
                        return Ok(RotationOutcome::ReuseDetected {
                            user_id: existing_token.user_id,
                        });
                    }

                    // Immediately revoke the old token
                    diesel::update(refresh_tokens_table::table.find(existing_token.id))
                        .set(revoked_at.eq(Some(now)))
                        .execute(conn)
                        .await?;

                    // Generate new plain token and hash it with the provided salt
                    let new_token_plain = generate_random_token(48);
                    let new_token_hash = hash_token(&new_token_plain, &salt);
                    let new_expires_at = now + Duration::seconds(expires_secs);

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

                    Ok(RotationOutcome::Rotated {
                        token: created_token,
                        plaintext: new_token_plain,
                    })
                })
            })
            .await
    }
}

#[cfg(test)]
mod rotation_outcome_tests {
    use super::*;
    use time::Duration;

    fn stored_token(revoked_instant: Option<OffsetDateTime>) -> RefreshToken {
        RefreshToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "hmac:deadbeef".to_string(),
            device_info: None,
            ip_address: None,
            expires_at: OffsetDateTime::now_utc() + Duration::days(30),
            revoked_at: revoked_instant,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn a_token_revoked_inside_the_grace_window_signals_a_reuse() {
        let now = OffsetDateTime::now_utc();
        let matched = stored_token(Some(now - Duration::seconds(60)));
        assert!(matched_token_is_reuse(&matched, now));
    }

    #[test]
    fn a_token_revoked_outside_the_grace_window_is_an_expired_lineage() {
        let now = OffsetDateTime::now_utc();
        let matched = stored_token(Some(now - Duration::seconds(ROTATION_GRACE_SECS + 60)));
        assert!(!matched_token_is_reuse(&matched, now));
    }

    #[test]
    fn a_live_token_is_never_a_reuse() {
        let now = OffsetDateTime::now_utc();
        assert!(!matched_token_is_reuse(&stored_token(None), now));
    }

    #[test]
    fn a_row_revoked_in_the_future_is_treated_as_inside_the_window() {
        // A clock skew backwards must not reopen a replayed token.
        let now = OffsetDateTime::now_utc();
        let matched = stored_token(Some(now + Duration::seconds(30)));
        assert!(matched_token_is_reuse(&matched, now));
    }

    #[test]
    fn the_reuse_outcome_names_the_user_whose_family_must_be_revoked() {
        let expected_user_id = Uuid::new_v4();
        let outcome = RotationOutcome::ReuseDetected {
            user_id: expected_user_id,
        };
        match outcome {
            RotationOutcome::ReuseDetected { user_id } => assert_eq!(user_id, expected_user_id),
            other => panic!("unexpected outcome: {other:?}"),
        }
    }
}
