use crate::DBPool;
use crate::db::schema::profiles as profiles_table;
use crate::models::profile::{NewProfile, Profile};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::profiles_trait::IProfileRepository;
use uuid::Uuid;

pub struct ProfilesRepository {
    base: BaseRepo,
}

impl ProfilesRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::profiles_trait::MockIProfileRepository;

#[async_trait::async_trait]
impl IProfileRepository for ProfilesRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<Profile>> {
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(|conn| {
                profiles_table::table
                    .select(Profile::as_select())
                    .load::<Profile>(conn)
            })
            .await
    }

    async fn find(&self, pid: &Uuid) -> diesel::QueryResult<Profile> {
        let pid = *pid;
        use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(move |conn| {
                profiles_table::table
                    .find(pid)
                    .select(Profile::as_select())
                    .first::<Profile>(conn)
            })
            .await
    }

    async fn create(&self, item: &NewProfile) -> diesel::QueryResult<Profile> {
        use diesel::{RunQueryDsl, SelectableHelper};
        let item = item.clone();
        self.base
            .run(move |conn| {
                diesel::insert_into(profiles_table::table)
                    .values(&item)
                    .returning(Profile::as_returning())
                    .get_result(conn)
            })
            .await
    }

    async fn update(&self, pid: &Uuid, item: &NewProfile) -> diesel::QueryResult<Profile> {
        let item = item.clone();
        let pid = *pid;
        use crate::db::schema::profiles::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
        self.base
            .run(move |conn| {
                diesel::update(profiles_table::table.find(pid))
                    .set((
                        first_name.eq(item.first_name),
                        last_name.eq(item.last_name),
                        full_name.eq(item.full_name),
                        nickname.eq(item.nickname),
                        bio.eq(item.bio),
                        birthday.eq(item.birthday),
                        cpf_encrypted.eq(item.cpf_encrypted),
                        cpf_blind_index.eq(item.cpf_blind_index),
                        phone_encrypted.eq(item.phone_encrypted),
                        phone_blind_index.eq(item.phone_blind_index),
                        whatsapp_encrypted.eq(item.whatsapp_encrypted),
                        whatsapp_blind_index.eq(item.whatsapp_blind_index),
                        avatar.eq(item.avatar),
                        status.eq(item.status),
                        social_network.eq(item.social_network),
                        encryption_key_version.eq(item.encryption_key_version),
                    ))
                    .returning(Profile::as_returning())
                    .get_result(conn)
            })
            .await
    }

    async fn destroy(&self, pid: &Uuid) -> diesel::QueryResult<usize> {
        let pid = *pid;
        use diesel::{QueryDsl, RunQueryDsl};
        self.base
            .run(move |conn| diesel::delete(profiles_table::table.find(pid)).execute(conn))
            .await
    }

    async fn find_by_user_id(&self, uid: &Uuid) -> diesel::QueryResult<Option<Profile>> {
        let uid = *uid;
        use crate::db::schema::profiles::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        self.base
            .run(move |conn| {
                profiles_table::table
                    .filter(user_id.eq(uid))
                    .select(Profile::as_select())
                    .first::<Profile>(conn)
                    .optional()
            })
            .await
    }

    async fn find_by_cpf_blind_index(
        &self,
        blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<Profile>> {
        use crate::db::schema::profiles::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        let blind_index = blind_index_param.to_vec();
        self.base
            .run(move |conn| {
                profiles_table::table
                    .filter(cpf_blind_index.eq(blind_index))
                    .select(Profile::as_select())
                    .first::<Profile>(conn)
                    .optional()
            })
            .await
    }

    async fn find_by_phone_blind_index(
        &self,
        blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<Profile>> {
        use crate::db::schema::profiles::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        let blind_index = blind_index_param.to_vec();
        self.base
            .run(move |conn| {
                profiles_table::table
                    .filter(phone_blind_index.eq(blind_index))
                    .select(Profile::as_select())
                    .first::<Profile>(conn)
                    .optional()
            })
            .await
    }

    async fn find_by_whatsapp_blind_index(
        &self,
        blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<Profile>> {
        use crate::db::schema::profiles::dsl::*;
        use diesel::{
            ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
        };
        let blind_index = blind_index_param.to_vec();
        self.base
            .run(move |conn| {
                profiles_table::table
                    .filter(whatsapp_blind_index.eq(blind_index))
                    .select(Profile::as_select())
                    .first::<Profile>(conn)
                    .optional()
            })
            .await
    }
}
