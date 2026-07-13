use crate::DBPool;
use crate::db::schema::audit_logs as audit_logs_table;
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::audit_logs_trait::IAuditLogRepository;
use async_trait::async_trait;
use diesel::prelude::*;
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

#[async_trait]
impl IAuditLogRepository for AuditLogsRepository {
    async fn all(&self) -> diesel::QueryResult<Vec<AuditLog>> {
        self.base
            .run(|conn| audit_logs_table::table.load::<AuditLog>(conn))
            .await
    }

    async fn all_by_target_customer_ids(
        &self,
        customer_ids: &[Uuid],
    ) -> diesel::QueryResult<Vec<AuditLog>> {
        if customer_ids.is_empty() {
            return Ok(Vec::new());
        }

        let target_ids = customer_ids
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<Option<Uuid>>>();

        self.base
            .run(move |conn| {
                audit_logs_table::table
                    .filter(audit_logs_table::target_customer_id.eq_any(target_ids))
                    .load::<AuditLog>(conn)
            })
            .await
    }

    async fn find(&self, id: &Uuid) -> diesel::QueryResult<AuditLog> {
        let id = *id;
        self.base
            .run(move |conn| audit_logs_table::table.find(id).first::<AuditLog>(conn))
            .await
    }

    async fn create(&self, item: &NewAuditLog) -> diesel::QueryResult<AuditLog> {
        let item = item.clone();
        self.base
            .run(move |conn| {
                diesel::insert_into(audit_logs_table::table)
                    .values(&item)
                    .get_result(conn)
            })
            .await
    }

    async fn update(&self, id: &Uuid, item: &NewAuditLog) -> diesel::QueryResult<AuditLog> {
        let id = *id;
        let item = item.clone();
        self.base
            .run(move |conn| {
                diesel::update(audit_logs_table::table.find(id))
                    .set(&item)
                    .get_result(conn)
            })
            .await
    }

    async fn destroy(&self, id: &Uuid) -> diesel::QueryResult<usize> {
        let id = *id;
        self.base
            .run(move |conn| diesel::delete(audit_logs_table::table.find(id)).execute(conn))
            .await
    }
}
