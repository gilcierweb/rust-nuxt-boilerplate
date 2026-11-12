use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::DBPool;
use crate::db::functions::matches_fragment;
use crate::db::schema::audit_logs as audit_logs_table;
use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::audit_logs_trait::IAuditLogRepository;
use crate::services::audit_log_service::compute_audit_log_hash;
use crate::utils::pagination::{ListParams, PaginatedResponse, PaginationParams};

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
        search: Option<String>,
    ) -> diesel::QueryResult<PaginatedResponse<AuditLog>> {
        use diesel::BoolExpressionMethods;
        use diesel::expression_methods::NullableExpressionMethods;

        let list_params = ListParams::from(params.clone());
        let search = search
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());

        self.base
            .run(move |conn| {
                let search = search.clone();
                Box::pin(async move {
                    // The log is append only and grows without bound, so the
                    // filter and the order run in the database: the count and
                    // the page are two statements.
                    let pattern = search.map(|term| format!("%{term}%"));
                    let matches = |pattern: &Option<String>| {
                        pattern.as_ref().map(|pattern| {
                            matches_fragment(audit_logs_table::action.nullable(), pattern).or(
                                matches_fragment(
                                    audit_logs_table::resource_type.nullable(),
                                    pattern,
                                ),
                            )
                        })
                    };

                    let total: i64 = {
                        let mut query = audit_logs_table::table.into_boxed();
                        if let Some(predicate) = matches(&pattern) {
                            query = query.filter(predicate);
                        }
                        query.count().get_result(conn).await?
                    };

                    let mut query = audit_logs_table::table.into_boxed();
                    if let Some(predicate) = matches(&pattern) {
                        query = query.filter(predicate);
                    }

                    // The allowlist of the handler already refused anything
                    // outside the list, so this branch only picks the column.
                    let descending = list_params.sort_dir.as_deref() == Some("desc");
                    query = match list_params.sort_by.as_deref() {
                        Some("action") if descending => {
                            query.order_by(audit_logs_table::action.desc())
                        },
                        Some("action") => query.order_by(audit_logs_table::action.asc()),
                        Some("resource_type") if descending => {
                            query.order_by(audit_logs_table::resource_type.desc())
                        },
                        Some("resource_type") => {
                            query.order_by(audit_logs_table::resource_type.asc())
                        },
                        _ if descending => query.order_by(audit_logs_table::created_at.desc()),
                        _ => query.order_by(audit_logs_table::created_at.asc()),
                    };

                    let data = query
                        .select(AuditLog::as_select())
                        .limit(list_params.limit())
                        .offset(list_params.offset())
                        .load::<AuditLog>(conn)
                        .await?;

                    Ok(PaginatedResponse::new(
                        data,
                        total,
                        list_params.page,
                        list_params.per_page,
                    ))
                })
            })
            .await
    }
}
