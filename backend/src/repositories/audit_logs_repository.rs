use crate::DBPool;
use crate::db::schema::audit_logs as audit_logs_table;
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::audit_logs_trait::IAuditLogRepository;
use crate::services::audit_log_service::compute_audit_log_hash;
use crate::utils::pagination::{ListParams, PaginatedResponse, PaginationParams};
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

    async fn list_paginated(
        &self,
        params: &PaginationParams,
    ) -> diesel::QueryResult<PaginatedResponse<AuditLog>> {
        let list_params = ListParams::from(params.clone());

        self.base
            .run(move |conn| {
                Box::pin(async move {
                    // Load all data then sort in memory
                    let mut data = audit_logs_table::table.load::<AuditLog>(conn).await?;

                    if let Some(sort_by) = list_params.sort_by.as_deref() {
                        let desc = list_params.sort_dir.as_deref() == Some("desc");
                        data.sort_by(|a, b| {
                            let ord = match sort_by {
                                "action" => a.action.cmp(&b.action),
                                "resource_type" => a.resource_type.cmp(&b.resource_type),
                                "created_at" => a.created_at.cmp(&b.created_at),
                                "id" => a.id.cmp(&b.id),
                                _ => a.created_at.cmp(&b.created_at),
                            };
                            if desc { ord.reverse() } else { ord }
                        });
                    } else {
                        data.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                    }

                    // Apply pagination
                    let total_count = data.len() as i64;
                    let offset = list_params.offset() as usize;
                    let limit = list_params.limit() as usize;
                    let data: Vec<_> = data.into_iter().skip(offset).take(limit).collect();

                    Ok(PaginatedResponse::new(
                        data,
                        total_count,
                        list_params.page,
                        list_params.per_page,
                    ))
                })
            })
            .await
    }
}
