use crate::DBPool;
use crate::db::schema::users as users_table;
use crate::models::user::{NewUser, User};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::users_trait::IUserRepository;
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
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(|conn| {
                users_table::table
                    .select(User::as_select())
                    .load::<User>(conn)
            })
            .await
    }

    async fn find(&self, uid: &Uuid) -> diesel::QueryResult<User> {
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
        let uid_val = *uid;
        self.base
            .run(move |conn| {
                users_table::table
                    .find(uid_val)
                    .select(User::as_select())
                    .first::<User>(conn)
            })
            .await
    }

    async fn create(&self, item: &NewUser) -> diesel::QueryResult<User> {
        let item = item.clone();
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, RunQueryDsl, SelectableHelper};
        self.base
            .run(move |conn| {
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
            })
            .await
    }

    async fn update(&self, uid: &Uuid, item: &NewUser) -> diesel::QueryResult<User> {
        let item = item.clone();
        let uid = *uid;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(move |conn| {
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
            })
            .await
    }

    async fn destroy(&self, uid: &Uuid) -> diesel::QueryResult<usize> {
        use diesel::{QueryDsl, RunQueryDsl};
        let uid_val = *uid;
        self.base
            .run(move |conn| diesel::delete(users_table::table.find(uid_val)).execute(conn))
            .await
    }

    async fn find_by_username_or_email(
        &self,
        _username_or_email: &str,
        email_blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<User>> {
        use crate::db::schema::users::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        let blind_index = email_blind_index_param.to_vec();
        self.base
            .run(move |conn| {
                users
                    .filter(email_blind_index.eq(blind_index))
                    .select(User::as_select())
                    .first::<User>(conn)
                    .optional()
            })
            .await
    }

    async fn find_by_email(
        &self,
        email_blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<User>> {
        use crate::db::schema::users::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        let blind_index = email_blind_index_param.to_vec();
        self.base
            .run(move |conn| {
                users
                    .filter(email_blind_index.eq(blind_index))
                    .select(User::as_select())
                    .first::<User>(conn)
                    .optional()
            })
            .await
    }

    async fn find_by_reset_token_digest(
        &self,
        token_digest_param: &str,
    ) -> diesel::QueryResult<Option<User>> {
        use crate::db::schema::users::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        let search = token_digest_param.to_string();
        self.base
            .run(move |conn| {
                users
                    .filter(reset_password_token_digest.eq(&search))
                    .select(User::as_select())
                    .first::<User>(conn)
                    .optional()
            })
            .await
    }

    async fn update_login_info(
        &self,
        user_id: &Uuid,
        curr_sign_in_at: Option<chrono::NaiveDateTime>,
        last_sign_in_at_opt: Option<chrono::NaiveDateTime>,
        curr_sign_in_ip: Option<ipnet::IpNet>,
        last_sign_in_ip_opt: Option<ipnet::IpNet>,
    ) -> diesel::QueryResult<User> {
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        current_sign_in_at.eq(curr_sign_in_at),
                        last_sign_in_at.eq(last_sign_in_at_opt),
                        current_sign_in_ip.eq(curr_sign_in_ip),
                        last_sign_in_ip.eq(last_sign_in_ip_opt),
                        sign_in_count.eq(sign_in_count + 1),
                    ))
                    .returning(User::as_returning())
                    .get_result::<User>(conn)
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
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        encrypted_password.eq(pwd),
                        reset_password_token_digest.eq::<Option<String>>(None),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn update_reset_token(
        &self,
        user_id: &Uuid,
        token_digest: Option<String>,
        sent_at: Option<chrono::NaiveDateTime>,
    ) -> diesel::QueryResult<usize> {
        let tok = token_digest;
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        reset_password_token_digest.eq(tok),
                        reset_password_sent_at.eq(sent_at),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn update_pending_email(
        &self,
        user_id: &Uuid,
        blind_index_value: &[u8],
        encrypted_email_value: &[u8],
        token_digest: &str,
        sent_at: chrono::NaiveDateTime,
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        let blind_index = blind_index_value.to_vec();
        let encrypted_email = encrypted_email_value.to_vec();
        let token_digest = token_digest.to_string();
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        unconfirmed_email_blind_index.eq(blind_index),
                        unconfirmed_email_encrypted.eq(encrypted_email),
                        confirmation_token_digest.eq(Some(token_digest)),
                        confirmation_sent_at.eq(Some(sent_at)),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn confirm_email(&self, token_digest_param: &str) -> diesel::QueryResult<usize> {
        use crate::db::schema::users::dsl::*;
        use diesel::dsl::sql;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        let token_digest = token_digest_param.to_string();
        self.base
            .run(move |conn| {
                diesel::update(users.filter(confirmation_token_digest.eq(&token_digest)))
                    .set((
                        confirmed_at.eq(Some(chrono::Utc::now().naive_utc())),
                        email_blind_index.eq(sql::<diesel::sql_types::Bytea>(
                            "COALESCE(unconfirmed_email_blind_index, email_blind_index)",
                        )),
                        email_encrypted.eq(sql::<diesel::sql_types::Bytea>(
                            "COALESCE(unconfirmed_email_encrypted, email_encrypted)",
                        )),
                        confirmation_token_digest.eq::<Option<String>>(None),
                        unconfirmed_email_blind_index.eq::<Option<Vec<u8>>>(None),
                        unconfirmed_email_encrypted.eq::<Option<Vec<u8>>>(None),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
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
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        failed_attempts.eq(failed_attempts + 1),
                        locked_at.eq(diesel::dsl::case_when(
                            failed_attempts.ge(max_attempts - 1),
                            Some(chrono::Utc::now().naive_utc()),
                        )
                        .otherwise(None::<chrono::NaiveDateTime>)),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn record_successful_login(
        &self,
        user_id: &Uuid,
        ip: Option<ipnet::IpNet>,
    ) -> diesel::QueryResult<usize> {
        let ip_clone = ip.clone();
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        sign_in_count.eq(sign_in_count + 1),
                        current_sign_in_at.eq(Some(chrono::Utc::now().naive_utc())),
                        last_sign_in_at.eq(current_sign_in_at),
                        current_sign_in_ip.eq(ip_clone.clone()),
                        last_sign_in_ip.eq(current_sign_in_ip),
                        failed_attempts.eq(0),
                        locked_at.eq::<Option<chrono::NaiveDateTime>>(None),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn get_user_roles(&self, user_id: &Uuid) -> diesel::QueryResult<Vec<String>> {
        let user_id = *user_id;
        use crate::db::schema::{roles, users_roles};
        use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                users_roles::table
                    .inner_join(roles::table.on(users_roles::role_id.eq(roles::id)))
                    .filter(users_roles::user_id.eq(user_id))
                    .select(roles::name)
                    .load::<String>(conn)
            })
            .await
    }

    async fn get_user_permissions(&self, user_id: &Uuid) -> diesel::QueryResult<Vec<String>> {
        #[derive(diesel::QueryableByName)]
        struct PermissionCodeRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            code: String,
        }

        use diesel::RunQueryDsl;

        let user_id = *user_id;
        self.base
            .run(move |conn| {
                let rows = diesel::sql_query(
                    "SELECT DISTINCT p.code
                     FROM users_roles ur
                     INNER JOIN roles_permissions rp ON rp.role_id = ur.role_id
                     INNER JOIN permissions p ON p.id = rp.permission_id
                     WHERE ur.user_id = $1
                     ORDER BY p.code",
                )
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .load::<PermissionCodeRow>(conn)?;

                Ok(rows.into_iter().map(|row| row.code).collect())
            })
            .await
    }

    async fn create_password_reset_token(
        &self,
        user_id: &Uuid,
        token_digest: &str,
        sent_at: chrono::NaiveDateTime,
    ) -> diesel::QueryResult<usize> {
        let tok = token_digest.to_string();
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        reset_password_token_digest.eq(Some(tok)),
                        reset_password_sent_at.eq(Some(sent_at)),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn reset_password(
        &self,
        token_digest_param: &str,
        new_password: &str,
    ) -> diesel::QueryResult<usize> {
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        let tok = token_digest_param.to_string();
        let pwd = new_password.to_string();
        self.base
            .run(move |conn| {
                diesel::update(users.filter(reset_password_token_digest.eq(&tok)))
                    .set((
                        encrypted_password.eq(pwd),
                        reset_password_token_digest.eq::<Option<String>>(None),
                        reset_password_sent_at.eq::<Option<chrono::NaiveDateTime>>(None),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn set_otp_secret(&self, user_id: &Uuid, secret: &str) -> diesel::QueryResult<usize> {
        let sec = secret.to_string();
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        otp_secret.eq(Some(sec)),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
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
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        otp_enabled_at.eq(Some(chrono::Utc::now().naive_utc())),
                        otp_backup_codes.eq(Some(codes)),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }

    async fn disable_2fa(&self, user_id: &Uuid) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        use crate::db::schema::users::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| {
                diesel::update(users.find(user_id))
                    .set((
                        otp_secret.eq::<Option<String>>(None),
                        otp_enabled_at.eq::<Option<chrono::NaiveDateTime>>(None),
                        otp_backup_codes.eq::<Option<Vec<String>>>(None),
                        updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)
            })
            .await
    }
}
