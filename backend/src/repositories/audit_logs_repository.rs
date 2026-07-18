use crate::DBPool;
use crate::db::schema::audit_logs as audit_logs_table;
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::audit_logs_trait::IAuditLogRepository;
use crate::services::audit_log_service::compute_audit_log_hash;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
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
                        .order(audit_logs_table::created_at.desc())
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
                    // Get the previous log's hash for chaining
                    let prev_hash: Option<String> = audit_logs_table::table
                        .order(audit_logs_table::created_at.desc())
                        .select(audit_logs_table::hash)
                        .first::<String>(conn)
                        .await
                        .optional()?;

                    // Compute hash for this entry
                    let (prev_hash_str, hash) = compute_audit_log_hash(&item, prev_hash.as_deref());

                    // Create new item with hash chain
                    let mut new_item = item;
                    new_item.prev_hash = prev_hash_str;
                    new_item.hash = hash;

                    diesel::insert_into(audit_logs_table::table)
                        .values(&new_item)
                        .returning(AuditLog::as_returning())
                        .get_result(conn)
                        .await
                })
            })
            .await
    }

    async fn find_latest_hash(&self) -> diesel::QueryResult<Option<String>> {
        self.base
            .run(|conn| {
                Box::pin(async move {
                    audit_logs_table::table
                        .order(audit_logs_table::created_at.desc())
                        .select(audit_logs_table::hash)
                        .first::<String>(conn)
                        .await
                        .optional()
                })
            })
            .await
    }

    async fn find_batch_ordered_by_created_at(
        &self,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> diesel::QueryResult<Vec<AuditLog>> {
        self.base
            .run(move |conn| {
                Box::pin(async move {
                    let mut query = audit_logs_table::table
                        .order(audit_logs_table::created_at.asc())
                        .into_boxed();

                    if let Some(cursor) = cursor_id {
                        query = query.filter(audit_logs_table::id.gt(cursor));
                    }

                    query.limit(limit).load::<AuditLog>(conn).await
                })
            })
            .await
    }
}
