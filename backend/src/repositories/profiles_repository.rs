use crate::DBPool;
use crate::db::schema::profiles as profiles_table;
use crate::models::profile::{NewProfile, Profile};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::profiles_trait::IProfileRepository;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
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
        self.base
            .run(|conn| {
                Box::pin(async move {
                    profiles_table::table
                        .select(Profile::as_select())
                        .load::<Profile>(conn)
                        .await
                })
            })
            .await
    }

    async fn find(&self, pid: &Uuid) -> diesel::QueryResult<Profile> {
        let pid = *pid;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    profiles_table::table
                        .find(pid)
                        .select(Profile::as_select())
                        .first::<Profile>(conn)
                        .await
                })
            })
            .await
    }

    async fn create(&self, item: &NewProfile) -> diesel::QueryResult<Profile> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(profiles_table::table)
                        .values(&item)
                        .returning(Profile::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn update(&self, pid: &Uuid, item: &NewProfile) -> diesel::QueryResult<Profile> {
        let item = item.clone();
        let pid = *pid;
        use crate::db::schema::profiles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
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
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, pid: &Uuid) -> diesel::QueryResult<usize> {
        let pid = *pid;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(profiles_table::table.find(pid))
                        .execute(conn)
                        .await
                })
            })
            .await
    }

    async fn find_by_user_id(&self, uid: &Uuid) -> diesel::QueryResult<Option<Profile>> {
        let uid = *uid;
        use crate::db::schema::profiles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    profiles_table::table
                        .filter(user_id.eq(uid))
                        .select(Profile::as_select())
                        .first::<Profile>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_by_cpf_blind_index(
        &self,
        blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<Profile>> {
        let blind_index = blind_index_param.to_vec();
        use crate::db::schema::profiles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    profiles_table::table
                        .filter(cpf_blind_index.eq(blind_index))
                        .select(Profile::as_select())
                        .first::<Profile>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_by_phone_blind_index(
        &self,
        blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<Profile>> {
        let blind_index = blind_index_param.to_vec();
        use crate::db::schema::profiles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    profiles_table::table
                        .filter(phone_blind_index.eq(blind_index))
                        .select(Profile::as_select())
                        .first::<Profile>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_by_whatsapp_blind_index(
        &self,
        blind_index_param: &[u8],
    ) -> diesel::QueryResult<Option<Profile>> {
        let blind_index = blind_index_param.to_vec();
        use crate::db::schema::profiles::dsl::*;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    profiles_table::table
                        .filter(whatsapp_blind_index.eq(blind_index))
                        .select(Profile::as_select())
                        .first::<Profile>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }
}
