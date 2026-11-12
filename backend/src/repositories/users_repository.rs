use std::collections::HashSet;
use std::sync::Arc;

use chrono::NaiveDateTime;
use diesel::BoolExpressionMethods;
use diesel::JoinOnDsl;
use diesel::helper_types::Or;
use diesel::result::QueryResult;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ipnet::IpNet;
use uuid::Uuid;

use crate::db::database::DBPool;
use crate::db::functions::{FragmentMatch, matches_fragment};
use crate::db::schema::profiles;
use crate::db::schema::roles as roles_table;
use crate::db::schema::users as users_table;
use crate::models::profile::Profile;
use crate::models::role::Role;
use crate::models::user::{NewUser, User};
use crate::models::user_role::NewUserRole;
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::users_trait::{
    AdminUserItem, AdminUserLookupItem, IUserRepository, IUserRepositoryTransaction,
};
use crate::security::SecurityService;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

/// The fragment predicate of the admin user search: the four profile name
/// columns, case insensitively. The address itself is encrypted, so a fragment
/// of it never matches and the term reaches here as a name fragment.
///
/// The count and the page of the admin list are two statements of the same
/// result set, so the predicate is written once and both carry it.
type ProfileFragmentPredicate = Or<
    Or<
        Or<FragmentMatch<profiles::full_name>, FragmentMatch<profiles::first_name>>,
        FragmentMatch<profiles::last_name>,
    >,
    FragmentMatch<profiles::nickname>,
>;

fn profile_fragment_predicate(term: &str) -> ProfileFragmentPredicate {
    let pattern = format!("%{}%", term.to_lowercase());
    matches_fragment(profiles::full_name, &pattern)
        .or(matches_fragment(profiles::first_name, &pattern))
        .or(matches_fragment(profiles::last_name, &pattern))
        .or(matches_fragment(profiles::nickname, &pattern))
}

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

    async fn count(&self) -> diesel::QueryResult<i64> {
        self.base
            .run(|conn| {
                Box::pin(async move { users_table::table.count().first::<i64>(conn).await })
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
                            reset_password_token_digest.eq::<Option<String>>(None),
                            reset_password_sent_at.eq::<Option<chrono::NaiveDateTime>>(None),
                            failed_attempts.eq(0),
                            locked_at.eq::<Option<chrono::NaiveDateTime>>(None),
                            unlock_token_digest.eq::<Option<String>>(None),
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
        refresh_token_hash_salt: &str,
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        let salt = refresh_token_hash_salt.to_string();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    // Generate unlock token digest when account gets locked
                    let unlock_token = Uuid::new_v4().to_string();
                    let new_unlock_token_digest = crate::services::token_service::hash_token(
                        &unlock_token,
                        &salt,
                    );

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
                            unlock_token_digest.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<
                                diesel::sql_types::Varchar,
                            >>(&format!(
                                "CASE WHEN failed_attempts + 1 >= {max_attempts} THEN '{}' ELSE unlock_token_digest END",
                                new_unlock_token_digest
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
                            unlock_token_digest.eq::<Option<String>>(None),
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

    async fn get_user_grants(
        &self,
        user_id: &Uuid,
    ) -> diesel::QueryResult<Vec<crate::repositories::traits::users_trait::RoleGrantRow>> {
        use crate::db::schema::{permissions, roles, roles_permissions, users_roles};
        use diesel::expression_methods::NullableExpressionMethods;

        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    // A role without links still answers with NULL fields, so
                    // the two outer joins name their ON clause and the order
                    // keeps the linked codes first.
                    users_roles::table
                        .inner_join(roles::table)
                        .left_join(
                            roles_permissions::table.on(roles_permissions::role_id.eq(roles::id)),
                        )
                        .left_join(
                            permissions::table
                                .on(permissions::id.eq(roles_permissions::permission_id)),
                        )
                        .filter(users_roles::user_id.eq(user_id))
                        .select((
                            roles::name,
                            roles::resource_type,
                            roles::resource_id,
                            permissions::code.nullable(),
                            roles_permissions::effect.nullable(),
                        ))
                        .load::<(
                            String,
                            Option<String>,
                            Option<Uuid>,
                            Option<String>,
                            Option<i16>,
                        )>(conn)
                        .await
                        .map(|rows: Vec<_>| {
                            // A role without links comes first, then its linked
                            // codes in order: the set is one user's grants, so
                            // the comparison happens in Rust.
                            let mut grants: Vec<
                                crate::repositories::traits::users_trait::RoleGrantRow,
                            > = rows
                                .into_iter()
                                .map(
                                    |(
                                        role_name,
                                        resource_type,
                                        resource_id,
                                        permission_code,
                                        effect,
                                    )| {
                                        crate::repositories::traits::users_trait::RoleGrantRow {
                                            role_name,
                                            resource_type,
                                            resource_id,
                                            permission_code,
                                            effect,
                                        }
                                    },
                                )
                                .collect();

                            grants.sort_by(|left, right| {
                                left.role_name.cmp(&right.role_name).then_with(|| {
                                    match (&left.permission_code, &right.permission_code) {
                                        (None, None) => std::cmp::Ordering::Equal,
                                        (None, Some(_)) => std::cmp::Ordering::Less,
                                        (Some(_), None) => std::cmp::Ordering::Greater,
                                        (Some(left_code), Some(right_code)) => {
                                            left_code.cmp(right_code)
                                        },
                                    }
                                })
                            });
                            grants
                        })
                })
            })
            .await
    }

    /// The roles assigned to a user, as the admin interface reads them back.
    async fn find_user_role_assignments(&self, user_id: &Uuid) -> QueryResult<Vec<(Uuid, String)>> {
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::{roles, users_roles};
                    users_roles::table
                        .filter(users_roles::dsl::user_id.eq(user_id))
                        .inner_join(roles::table)
                        .select((roles::dsl::id, roles::dsl::name))
                        .load::<(Uuid, String)>(conn)
                        .await
                })
            })
            .await
    }

    /// Replaces every role assignment of a user in one transaction: the old
    /// links are deleted and the new ones inserted, so a refused insert leaves
    /// the user with the previous roles instead of none.
    async fn set_user_roles(&self, user_id: &Uuid, role_ids: &[Uuid]) -> QueryResult<usize> {
        use crate::db::schema::users_roles;

        let user_id = *user_id;
        let role_ids = role_ids.to_vec();

        self.base
            .run_transaction(move |conn| {
                let role_ids = role_ids.clone();
                Box::pin(async move {
                    diesel::delete(users_roles::table)
                        .filter(users_roles::user_id.eq(user_id))
                        .execute(conn)
                        .await?;

                    if role_ids.is_empty() {
                        return Ok(0);
                    }

                    let links: Vec<NewUserRole> = role_ids
                        .into_iter()
                        .map(|role_id| NewUserRole { user_id, role_id })
                        .collect();
                    diesel::insert_into(users_roles::table)
                        .values(&links)
                        .execute(conn)
                        .await?;

                    Ok(links.len())
                })
            })
            .await
    }

    async fn user_ids_holding_all_codes(&self, codes: &[String]) -> diesel::QueryResult<Vec<Uuid>> {
        use crate::db::schema::{permissions, roles_permissions, users_roles};
        use diesel::AggregateExpressionMethods;

        let codes = codes.to_vec();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    users_roles::table
                        // The link between a user and a grant runs through the
                        // role id, so the join names its own ON clause.
                        .inner_join(
                            roles_permissions::table.on(roles_permissions::role_id
                                .eq(users_roles::role_id)),
                        )
                        .inner_join(
                            permissions::table.on(permissions::id
                                .eq(roles_permissions::permission_id)),
                        )
                        .filter(
                            roles_permissions::effect
                                .eq(0)
                                .and(permissions::code.eq_any(&codes)),
                        )
                        .group_by(users_roles::user_id)
                        .select(users_roles::user_id)
                        .having(
                            diesel::dsl::count(permissions::code).aggregate_distinct()
                                .eq(codes.len() as i64),
                        )
                        .load(conn)
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
                    use crate::db::schema::{roles, users_roles};
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

    async fn find_assignable_roles_by_name(&self, name: &str) -> diesel::QueryResult<Vec<Role>> {
        let role_name = name.to_string();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::roles::dsl::*;

                    roles_table::table
                        .filter(name.eq(&role_name))
                        .filter(resource_id.is_null())
                        .load::<Role>(conn)
                        .await
                })
            })
            .await
    }

    /// The effective allow codes of a role, expanded over the wildcards.
    async fn role_allow_codes(&self, role_id: &Uuid) -> QueryResult<HashSet<String>> {
        use crate::db::schema::{permissions, roles_permissions};

        let role_id = *role_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let codes: Vec<String> = roles_permissions::table
                        .inner_join(permissions::table)
                        .filter(
                            roles_permissions::role_id
                                .eq(role_id)
                                .and(roles_permissions::effect.eq(0)),
                        )
                        .select(permissions::code)
                        .load(conn)
                        .await?;

                    Ok(crate::authz::ability::expand_allow_codes(&codes))
                })
            })
            .await
    }

    async fn get_user_permissions(&self, user_id: &Uuid) -> diesel::QueryResult<Vec<String>> {
        let user_id = *user_id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    use crate::db::schema::{permissions, roles_permissions, users_roles};
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
        backup_codes: &[Option<String>],
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

    async fn update_backup_codes(
        &self,
        user_id: &Uuid,
        backup_codes: &[Option<String>],
    ) -> diesel::QueryResult<usize> {
        let user_id = *user_id;
        let codes = backup_codes.to_vec();
        use crate::db::schema::users::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(users_table::table.find(user_id))
                        .set((
                            otp_backup_codes.eq(Some(codes)),
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
        search: Option<String>,
        security: Arc<SecurityService>,
    ) -> diesel::QueryResult<PaginatedResponse<AdminUserLookupItem>> {
        use crate::db::schema::{profiles, roles, users_roles};
        use diesel::expression_methods::NullableExpressionMethods;

        let params = params.validated();
        let limit = params.limit();
        let offset = params.offset();
        let descending = params.sort_dir.as_deref() == Some("desc");

        self.base
            .run(move |conn| {
                let security = security.clone();
                Box::pin(async move {
                    // The shape of the admin user list: one row per user with
                    // its optional profile. The count and the page read the
                    // same result set, so the metadata describes what the
                    // client sees and not the whole table.
                    let filtered = || {
                        users_table::table
                            .left_join(profiles::table.on(profiles::user_id.eq(users_table::id)))
                            .select((
                                users_table::id,
                                users_table::email_encrypted,
                                users_table::encryption_key_version,
                                // A left joined column is nullable, so the
                                // select clause has to say so.
                                profiles::first_name.nullable(),
                                profiles::last_name.nullable(),
                                profiles::full_name.nullable(),
                                profiles::nickname.nullable(),
                            ))
                            .into_boxed()
                    };

                    // ── 1. Total count, single COUNT(*) with no data fetched,
                    // over the very filter the page carries.
                    let mut counting = filtered();
                    if let Some(term) = search
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        match security.email_blind_index(term) {
                            Ok(index) => {
                                counting =
                                    counting.filter(users_table::email_blind_index.eq(index));
                            },
                            Err(_) => {
                                counting = counting.filter(profile_fragment_predicate(term));
                            },
                        }
                    }
                    let total: i64 = counting.count().get_result(conn).await?;

                    // ── 2. Page query — one JOIN, ordered and paginated in the
                    // database. The boxed query lets the ORDER BY column come
                    // from an allowlist without any string interpolation.
                    let mut query = filtered();

                    let search = search.as_deref();
                    if let Some(term) = search.map(str::trim).filter(|value| !value.is_empty()) {
                        match security.email_blind_index(term) {
                            Ok(index) => {
                                query = query.filter(users_table::email_blind_index.eq(index));
                            },
                            Err(_) => {
                                query = query.filter(profile_fragment_predicate(term));
                            },
                        }
                    }

                    // The handler already refused a sort field outside its
                    // allowlist, so this branch only picks the column. The email
                    // is encrypted: its blind index order is not the alphabetical
                    // order of the address, which is why the field is absent.
                    query = match (params.sort_by.as_deref(), descending) {
                        (Some("display_name"), false) => query.order_by(profiles::full_name.asc()),
                        (Some("display_name"), true) => query.order_by(profiles::full_name.desc()),
                        (Some("created_at"), true) => {
                            query.order_by(users_table::created_at.desc())
                        },
                        (Some("created_at"), false) => {
                            query.order_by(users_table::created_at.asc())
                        },
                        (_, true) => query.order_by(users_table::created_at.desc()),
                        (_, false) => query.order_by(users_table::created_at.asc()),
                    };

                    type PageRow = (
                        uuid::Uuid,
                        Vec<u8>,
                        i32,
                        Option<String>,
                        Option<String>,
                        Option<String>,
                        Option<String>,
                    );
                    let rows: Vec<PageRow> = query.limit(limit).offset(offset).load(conn).await?;

                    // ── 3. The role names of the page, one query for all of
                    // them: the link table is keyed by user, so the ids of the
                    // page answer with a single index scan.
                    let page_ids: Vec<uuid::Uuid> = rows.iter().map(|(id, ..)| *id).collect();
                    let role_rows: Vec<(uuid::Uuid, String)> = if page_ids.is_empty() {
                        Vec::new()
                    } else {
                        users_roles::table
                            .inner_join(roles::table)
                            .filter(users_roles::user_id.eq_any(&page_ids))
                            .select((users_roles::user_id, roles::name))
                            .order_by(roles::name.asc())
                            .load(conn)
                            .await?
                    };

                    // ── 4. Decrypt the page and group its role names ──────────
                    let mut items = Vec::with_capacity(rows.len());
                    for (
                        id,
                        email_encrypted,
                        key_version,
                        first_name,
                        last_name,
                        full_name,
                        nickname,
                    ) in rows
                    {
                        let email = security
                            .decrypt_email_fields(&email_encrypted, key_version)
                            .map_err(|e| {
                                diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::Unknown,
                                    Box::new(e.to_string()),
                                )
                            })?;

                        let roles: Vec<String> = role_rows
                            .iter()
                            .filter(|(user_id, _)| *user_id == id)
                            .map(|(_, name)| name.clone())
                            .collect();

                        items.push(AdminUserLookupItem {
                            id,
                            email,
                            first_name,
                            last_name,
                            full_name,
                            nickname,
                            roles,
                        });
                    }

                    Ok(PaginatedResponse::new(
                        items,
                        total,
                        params.page,
                        params.per_page,
                    ))
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
