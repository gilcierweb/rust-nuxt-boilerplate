use crate::DBPool;
use crate::db::schema::audit_logs as audit_logs_table;
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::audit_logs_trait::IAuditLogRepository;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct AuditLogsRepository {
    base: BaseRepo,
}

impl AuditLogsRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::audit_logs_trait::MockIAuditLogRepository;

#[async_trait::async_trait]
impl IAuditLogRepository for AuditLogsRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<AuditLog>> {
        self.base
            .run(|conn| {
                Box::pin(async move {
                    audit_logs_table::table
                        .load::<AuditLog>(conn)
                        .await
                })
            })
            .await
    }

    async fn find(&self, id: &Uuid) -> diesel::QueryResult<AuditLog> {
        let id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    audit_logs_table::table
                        .find(id)
                        .first::<AuditLog>(conn)
                        .await
                })
            })
            .await
    }

    async fn create(&self, item: &NewAuditLog) -> diesel::QueryResult<AuditLog> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::insert_into(audit_logs_table::table)
                        .values(&item)
                        .returning(AuditLog::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn update(&self, id: &Uuid, item: &NewAuditLog) -> diesel::QueryResult<AuditLog> {
        let id = *id;
        let item = item.clone();
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::update(audit_logs_table::table.find(id))
                        .set(&item)
                        .returning(AuditLog::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn destroy(&self, id: &Uuid) -> diesel::QueryResult<usize> {
        let id = *id;
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    diesel::delete(audit_logs_table::table.find(id))
                        .execute(conn)
                        .await
                })
            })
            .await
    }
}
