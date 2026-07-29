use crate::db::database::DBPool;
use crate::db::schema::users as users_table;
use crate::models::profile::Profile;
use crate::models::user::{NewUser, User};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::users_trait::{
    AdminUserItem, AdminUserLookupItem, IUserRepository, IUserRepositoryTransaction,
};
use crate::security::SecurityService;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ipnet::IpNet;
use std::sync::Arc;
use uuid::Uuid;

pub struct UsersRepository {
    base: BaseRepo,
}

impl UsersRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::users_trait::MockIUserRepository;

#[async_trait::async_trait]
impl IUserRepository for UsersRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<User>> {
        self.base
            .run(|conn| {
                Box::pin(async move {
                    users_table::table
                        .select(User::as_select())
                        .load::<User>(conn)
                        .await
                })
            })
            .await
    }

    async fn find(&self, uid: &Uuid) -> diesel::QueryResult<User> {
        let uid_val = *uid;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_table::table
                        .find(uid_val)
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                })
            })
            .await
    }

    async fn create(&self, item: &NewUser) -> diesel::QueryResult<User> {
        let item = item.clone();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(users_table::table)
                        .values((
                            id.eq(item.id),
                            email_blind_index.eq(item.email_blind_index),
                            email_encrypted.eq(item.email_encrypted),
                            encrypted_password.eq(&item.encrypted_password),
                            confirmation_token_digest.eq(item.confirmation_token_digest),
                            unconfirmed_email_blind_index.eq(item.unconfirmed_email_blind_index),
                            unconfirmed_email_encrypted.eq(item.unconfirmed_email_encrypted),
                            encryption_key_version.eq(item.encryption_key_version),
                            created_at.eq(item.created_at),
                            updated_at.eq(item.updated_at),
                        ))
                        .returning(User::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn update(&self, uid: &Uuid, item: &NewUser) -> diesel::QueryResult<User> {
        let item = item.clone();
        let uid = *uid;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(uid))
                        .set((
                            email_blind_index.eq(item.email_blind_index),
                            email_encrypted.eq(item.email_encrypted),
                            encrypted_password.eq(&item.encrypted_password),
                            unconfirmed_email_blind_index.eq(item.unconfirmed_email_blind_index),
                            unconfirmed_email_encrypted.eq(item.unconfirmed_email_encrypted),
                            encryption_key_version.eq(item.encryption_key_version),
                            updated_at.eq(item.updated_at),
                        ))
                        .returning(User::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, uid: &Uuid) -> diesel::QueryResult<usize> {
        let uid = *uid;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(users_table::table.find(uid))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_username_or_email(
        &self,
        _username_or_email: &str,
        email_blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<User>> {
        let blind_index = email_blind_index_param.to_vec();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_table::table
                        .filter(email_blind_index.eq(blind_index))
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_by_email(
        &self,
        email_blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<User>> {
        let blind_index = email_blind_index_param.to_vec();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_table::table
                        .filter(email_blind_index.eq(blind_index))
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_by_reset_token_digest(
        &self,
        token_digest_param: &str,
    ) -> diesel::QueryResult<Option<User>> {
        let token_digest = token_digest_param.to_string();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_table::table
                        .filter(reset_password_token_digest.eq(token_digest))
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn update_login_info(
        &self,
        user_id: &Uuid,
        current_sign_in_at_val: Option<NaiveDateTime>,
        last_sign_in_at_val: Option<NaiveDateTime>,
        current_sign_in_ip_val: Option<IpNet>,
        last_sign_in_ip_val: Option<IpNet>,
    ) -> diesel::QueryResult<User> {
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::users::dsl::*;
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            current_sign_in_at.eq(current_sign_in_at_val),
                            last_sign_in_at.eq(last_sign_in_at_val),
                            current_sign_in_ip.eq(current_sign_in_ip_val),
                            last_sign_in_ip.eq(last_sign_in_ip_val),
                            sign_in_count.eq(diesel::dsl::sql::<diesel::sql_types::Integer>(
                                "sign_in_count + 1",
                            )),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .returning(User::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn update_password(
        &self,
        user_id: &Uuid,
        new_encrypted_password: &str,
    ) -> diesel::QueryResult<usize> {
        let pwd = new_encrypted_password.to_string();
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            encrypted_password.eq(pwd),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn update_reset_token(
        &self,
        user_id: &Uuid,
        token_digest: Option<String>,
        sent_at: Option<NaiveDateTime>,
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            reset_password_token_digest.eq(token_digest),
                            reset_password_sent_at.eq(sent_at),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn update_pending_email(
        &self,
        user_id: &Uuid,
        blind_index: &[u8],
        encrypted_email: &[u8],
        token_digest: &str,
        sent_at: NaiveDateTime,
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        let bi = blind_index.to_vec();
        let ee = encrypted_email.to_vec();
        let td = token_digest.to_string();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            unconfirmed_email_blind_index.eq(bi),
                            unconfirmed_email_encrypted.eq(ee),
                            confirmation_token_digest.eq(Some(td)),
                            confirmation_sent_at.eq(Some(sent_at)),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn confirm_email(&self, token_digest: &str) -> diesel::QueryResult<usize> {
        let td = token_digest.to_string();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(
                        users_table::table.filter(confirmation_token_digest.eq(Some(td))),
                    )
                    .set((
                        email_blind_index.eq(diesel::dsl::sql::<diesel::sql_types::Binary>(
                            "unconfirmed_email_blind_index",
                        )),
                        email_encrypted.eq(diesel::dsl::sql::<diesel::sql_types::Binary>(
                            "unconfirmed_email_encrypted",
                        )),
                        unconfirmed_email_blind_index.eq::<Option<Vec<u8>>>(None),
                        unconfirmed_email_encrypted.eq::<Option<Vec<u8>>>(None),
                        confirmation_token_digest.eq::<Option<String>>(None),
                        confirmation_sent_at.eq::<Option<NaiveDateTime>>(None),
                        confirmed_at.eq(Some(chrono::Utc::now().naive_utc())),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .await
                })
            })
            .await
    }

    async fn record_failed_login(
        &self,
        user_id: &Uuid,
        max_attempts: i32,
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            failed_attempts.eq(diesel::dsl::sql::<diesel::sql_types::Integer>(
                                "failed_attempts + 1",
                            )),
                            locked_at.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<
                                diesel::sql_types::Timestamptz,
                            >>(&format!(
                                "CASE WHEN failed_attempts + 1 >= {max_attempts} THEN NOW() ELSE locked_at END"
                            ))),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn record_successful_login(
        &self,
        user_id: &Uuid,
        ip: Option<IpNet>,
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::users::dsl::*;
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            failed_attempts.eq(0),
                            locked_at.eq::<Option<chrono::NaiveDateTime>>(None),
                            current_sign_in_at.eq(Some(chrono::Utc::now().naive_utc())),
                            last_sign_in_at.eq(diesel::dsl::sql::<
                                diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>,
                            >("current_sign_in_at")),
                            current_sign_in_ip.eq(ip),
                            sign_in_count.eq(diesel::dsl::sql::<diesel::sql_types::Integer>(
                                "sign_in_count + 1",
                            )),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn get_user_roles(&self, user_id: &Uuid) -> diesel::QueryResult<Vec<String>> {
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::roles;
                    use crate::db::schema::users_roles;
                    users_roles::table
                        .filter(users_roles::dsl::user_id.eq(user_id))
                        .inner_join(roles::table)
                        .select(roles::dsl::name)
                        .load::<String>(conn)
                        .await
                })
            })
            .await
    }

    async fn get_user_permissions(&self, user_id: &Uuid) -> diesel::QueryResult<Vec<String>> {
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::permissions;
                    use crate::db::schema::roles_permissions;
                    use crate::db::schema::users_roles;
                    use diesel::JoinOnDsl;
                    users_roles::table
                        .filter(users_roles::dsl::user_id.eq(user_id))
                        .inner_join(
                            roles_permissions::table
                                .on(users_roles::dsl::role_id.eq(roles_permissions::dsl::role_id)),
                        )
                        .inner_join(
                            permissions::table
                                .on(roles_permissions::dsl::permission_id.eq(permissions::dsl::id)),
                        )
                        .select(permissions::dsl::code)
                        .distinct()
                        .load::<String>(conn)
                        .await
                })
            })
            .await
    }

    async fn create_password_reset_token(
        &self,
        user_id: &Uuid,
        token_digest: &str,
        sent_at: NaiveDateTime,
    ) -> diesel::QueryResult<usize> {
        let tok = token_digest.to_string();
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            reset_password_token_digest.eq(Some(tok)),
                            reset_password_sent_at.eq(Some(sent_at)),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn reset_password(
        &self,
        token_digest_param: &str,
        new_password: &str,
    ) -> diesel::QueryResult<usize> {
        let tok = token_digest_param.to_string();
        let pwd = new_password.to_string();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.filter(reset_password_token_digest.eq(&tok)))
                        .set((
                            encrypted_password.eq(pwd),
                            reset_password_token_digest.eq::<Option<String>>(None),
                            reset_password_sent_at.eq::<Option<chrono::NaiveDateTime>>(None),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn set_otp_secret(&self, user_id: &Uuid, otp_secret: &str) -> diesel::QueryResult<usize> {
        let sec = otp_secret.to_string();
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::users::dsl::*;
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            otp_secret.eq(Some(sec)),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn enable_2fa(
        &self,
        user_id: &Uuid,
        backup_codes: &[String],
    ) -> diesel::QueryResult<usize> {
        let codes = backup_codes.to_vec();
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            otp_enabled_at.eq(Some(chrono::Utc::now().naive_utc())),
                            otp_backup_codes.eq(Some(codes)),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn disable_2fa(&self, user_id: &Uuid) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            otp_secret.eq::<Option<String>>(None),
                            otp_enabled_at.eq::<Option<chrono::NaiveDateTime>>(None),
                            otp_backup_codes.eq::<Option<Vec<String>>>(None),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn list_paginated(
        &self,
        params: &PaginationParams,
        security: Arc<SecurityService>,
    ) -> diesel::QueryResult<PaginatedResponse<AdminUserLookupItem>> {
        let params = params.validated();
        let limit = params.limit();
        let offset = params.offset();
        let sort_by = params.sort_by.clone().unwrap_or_else(|| "email".to_string());
        let sort_dir = params.sort_direction().to_string();

        self.base
            .run(move |conn| {
                let security = security.clone();
                Box::pin(async move {
                    // ── 1. Total count — single COUNT(*) query, no data fetched ──────────
                    let total: i64 = users_table::table
                        .count()
                        .get_result(conn)
                        .await?;

                    // ── 2. Page query — JOIN + ORDER + LIMIT + OFFSET in the database ────
                    //
                    // Column allow-list prevents SQL injection via `sort_by`.
                    // Any unrecognised value falls back to `u.created_at DESC`.
                    let order_sql = match sort_by.as_str() {
                        "email"      => "u.email_blind_index ASC",
                        "email_desc" => "u.email_blind_index DESC",
                        "first_name" => "p.first_name ASC NULLS LAST",
                        "first_name_desc" => "p.first_name DESC NULLS LAST",
                        "last_name"  => "p.last_name ASC NULLS LAST",
                        "last_name_desc" => "p.last_name DESC NULLS LAST",
                        "id"  => if sort_dir == "desc" { "u.id DESC" } else { "u.id ASC" },
                        _    => "u.created_at DESC",
                    };

                    // diesel does not support dynamic ORDER BY via the query
                    // builder for multi-table aliases, so we use sql_query here.
                    // The query is parameterised for LIMIT/OFFSET; the ORDER BY
                    // clause comes from the allow-list above and is never built
                    // from user input directly.
                    let raw_sql = format!(
                        "SELECT \
                            u.id            AS user_id, \
                            u.email_encrypted, \
                            u.encryption_key_version, \
                            p.first_name, \
                            p.last_name, \
                            p.full_name, \
                            p.nickname \
                         FROM users u \
                         LEFT JOIN profiles p ON p.user_id = u.id \
                         ORDER BY {order_sql} \
                         LIMIT $1 OFFSET $2"
                    );

                    #[derive(diesel::QueryableByName)]
                    struct UserProfileRow {
                        #[diesel(sql_type = diesel::sql_types::Uuid)]
                        user_id: uuid::Uuid,
                        #[diesel(sql_type = diesel::sql_types::Bytea)]
                        email_encrypted: Vec<u8>,
                        #[diesel(sql_type = diesel::sql_types::Integer)]
                        encryption_key_version: i32,
                        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                        first_name: Option<String>,
                        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                        last_name: Option<String>,
                        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                        full_name: Option<String>,
                        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                        nickname: Option<String>,
                    }

                    let rows: Vec<UserProfileRow> = diesel::sql_query(raw_sql)
                        .bind::<diesel::sql_types::BigInt, _>(limit)
                        .bind::<diesel::sql_types::BigInt, _>(offset)
                        .load(conn)
                        .await?;

                    // ── 3. Decrypt only the page (≤100 rows) ──────────────────────────────
                    let mut items = Vec::with_capacity(rows.len());
                    for row in rows {
                        // Reconstruct a minimal User-like value for decrypt_user_email.
                        // We only need the encrypted email fields + key version.
                        let email = security
                            .decrypt_email_fields(&row.email_encrypted, row.encryption_key_version)
                            .map_err(|e| {
                                diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::Unknown,
                                    Box::new(e.to_string()),
                                )
                            })?;

                        items.push(AdminUserLookupItem {
                            id: row.user_id,
                            email,
                            first_name: row.first_name,
                            last_name: row.last_name,
                            full_name: row.full_name,
                            nickname: row.nickname,
                        });
                    }

                    Ok(PaginatedResponse::new(items, total, params.page, params.per_page))
                })
            })
            .await
    }

    async fn find_by_id_with_profile(
        &self,
        uid: &Uuid,
        security: Arc<SecurityService>,
    ) -> diesel::QueryResult<AdminUserItem> {
        let uid_val = *uid;
        self.base
            .run(move |conn| {
                let security = security.clone();
                Box::pin(async move {
                    let user = users_table::table
                        .find(uid_val)
                        .select(User::as_select())
                        .first::<User>(conn)
                        .await?;

                    let profile = crate::db::schema::profiles::table
                        .filter(crate::db::schema::profiles::dsl::user_id.eq(uid_val))
                        .select(Profile::as_select())
                        .first::<Profile>(conn)
                        .await
                        .optional()?;

                    // Use the injected SecurityService — no AppConfig::from_env() per request.
                    let email = security.decrypt_user_email(&user).map_err(|e| {
                        diesel::result::Error::DatabaseError(
                            diesel::result::DatabaseErrorKind::Unknown,
                            Box::new(e.to_string()),
                        )
                    })?;

                    let first_name = profile.as_ref().and_then(|p| p.first_name.clone());
                    let last_name = profile.as_ref().and_then(|p| p.last_name.clone());
                    let full_name = profile.as_ref().and_then(|p| p.full_name.clone());
                    let nickname = profile.as_ref().and_then(|p| p.nickname.clone());

                    let display_name =
                        profile
                            .as_ref()
                            .and_then(|p| p.full_name.clone())
                            .or_else(|| {
                                let parts: Vec<&str> =
                                    [first_name.as_deref(), last_name.as_deref()]
                                        .iter()
                                        .filter_map(|s| *s)
                                        .collect();
                                if parts.is_empty() {
                                    None
                                } else {
                                    Some(parts.join(" "))
                                }
                            });

                    Ok(AdminUserItem {
                        id: user.id,
                        email,
                        first_name,
                        last_name,
                        full_name,
                        nickname,
                        display_name,
                    })
                })
            })
            .await
    }
}

#[async_trait::async_trait]
impl IUserRepositoryTransaction for UsersRepository {
    async fn run_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(
                &'a mut diesel_async::AsyncPgConnection,
            ) -> futures::future::BoxFuture<'a, Result<T, E>>
            + Send,
        T: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static,
    {
        self.base.run_transaction(f).await
    }
}
