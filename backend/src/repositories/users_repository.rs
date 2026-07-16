use crate::DBPool;
use crate::db::schema::users as users_table;
use crate::models::user::{NewUser, User};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::users_trait::IUserRepository;
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ipnet::IpNet;
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
                            last_sign_in_at.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<
                                diesel::sql_types::Timestamptz,
                            >>("current_sign_in_at")),
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
                        .inner_join(permissions::table.on(
                            roles_permissions::dsl::permission_id.eq(permissions::dsl::id),
                        ))
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
                            reset_password_sent_at
                                .eq::<Option<chrono::NaiveDateTime>>(None),
                            updated_at.eq(chrono::Utc::now().naive_utc()),
                        ))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn set_otp_secret(
        &self,
        user_id: &Uuid,
        otp_secret: &str,
    ) -> diesel::QueryResult<usize> {
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
}
