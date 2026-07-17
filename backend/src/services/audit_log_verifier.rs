use std::sync::Arc;
use std::time::Duration;

use crate::repositories::audit_logs_repository::IAuditLogRepository;

/// Default audit log chain verification interval: 1 hour.
const DEFAULT_VERIFY_INTERVAL_SECS: u64 = 3600;

/// Run the background audit log chain verifier.
///
/// Periodically fetches all audit logs from the repository, verifies the
/// cryptographic hash chain, and logs results. On failure, emits structured
/// `tracing::error!` events so alerting pipelines can react.
///
/// This function runs forever and should be spawned via `actix::spawn`.
pub async fn run_audit_log_verifier(audit_repo: Arc<dyn IAuditLogRepository>) {
    let interval_secs = std::env::var("AUDIT_LOG_VERIFY_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_VERIFY_INTERVAL_SECS);

    tracing::info!(
        event = "audit_log_verifier.started",
        interval_secs,
        "Audit log chain verifier started"
    );

    loop {
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;

        match audit_repo.all().await {
            Ok(logs) => {
                let count = logs.len();
                match super::audit_log_service::verify_audit_log_chain(&logs).await {
                    Ok(verified) => {
                        tracing::info!(
                            event = "audit_log_verifier.success",
                            total_entries = count,
                            verified_entries = verified,
                            "Audit log chain verified successfully"
                        );
                    },
                    Err(e) => {
                        tracing::error!(
                            event = "audit_log_verifier.chain_broken",
                            total_entries = count,
                            error = %e,
                            "Audit log chain integrity check FAILED — possible tampering detected"
                        );
                    },
                }
            },
            Err(e) => {
                tracing::error!(
                    event = "audit_log_verifier.fetch_failed",
                    error = %e,
                    "Failed to fetch audit logs for chain verification"
                );
            },
        }
    }
}
