use crate::models::audit_log::NewAuditLog;
use serde_json::json;

/// Batch size for audit log chain verification to prevent OOM on large tables.
const AUDIT_LOG_VERIFY_BATCH_SIZE: i64 = 1000;

/// Compute SHA-256 hash for audit log entry with cryptographic chaining.
///
/// The hash is computed from the canonical JSON representation of the audit log
/// content plus the previous hash (if any). This creates an immutable chain
/// where any modification to any entry will invalidate all subsequent hashes.
pub fn compute_audit_log_hash(
    item: &NewAuditLog,
    prev_hash: Option<&str>,
) -> (Option<String>, String) {
    // Create a deterministic representation of the audit log for hashing
    let mut canonical = json!({
        "actor_user_id": item.actor_user_id,
        "actor_role_snapshot": item.actor_role_snapshot,
        "action": item.action,
        "resource_type": item.resource_type,
        "resource_id": item.resource_id,
        "ip_address": item.ip_address.as_ref().map(|ip| ip.to_string()),
        "user_agent": item.user_agent,
        "request_id": item.request_id,
        "changes": item.changes,
        "metadata": item.metadata,
    });

    // Include previous hash in the chain
    if let Some(prev) = prev_hash {
        canonical["prev_hash"] = json!(prev.to_string());
    } else {
        canonical["prev_hash"] = json!(null);
    }

    // Canonicalize JSON (sorted keys, no whitespace)
    let canonical_str =
        serde_json::to_string(&canonical).expect("Failed to serialize canonical audit log");

    // Compute SHA-256
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(canonical_str.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    (prev_hash.map(|s| s.to_string()), hash_hex)
}

/// Verify the integrity of the audit log chain.
///
/// Uses cursor-based pagination to fetch logs in batches, preventing OOM on large tables.
/// Returns the total number of entries verified and any errors found.
pub async fn verify_audit_log_chain_batched<R>(repo: &R) -> Result<usize, String>
where
    R: crate::repositories::traits::audit_logs_trait::IAuditLogRepository + ?Sized,
{
    let mut total_verified = 0;
    let mut prev_hash: Option<String> = None;
    let mut cursor_id: Option<uuid::Uuid> = None;

    loop {
        let batch = repo
            .find_batch_ordered_by_created_at(cursor_id, AUDIT_LOG_VERIFY_BATCH_SIZE)
            .await
            .map_err(|e| format!("Failed to fetch batch: {}", e))?;

        if batch.is_empty() {
            break;
        }

        for log in &batch {
            let canonical = serde_json::json!({
                "actor_user_id": log.actor_user_id,
                "actor_role_snapshot": log.actor_role_snapshot,
                "action": log.action,
                "resource_type": log.resource_type,
                "resource_id": log.resource_id,
                "ip_address": log.ip_address.as_ref().map(|ip| ip.to_string()),
                "user_agent": log.user_agent,
                "request_id": log.request_id,
                "changes": log.changes,
                "metadata": log.metadata,
                "prev_hash": log.prev_hash,
            });

            let canonical_str = serde_json::to_string(&canonical)
                .map_err(|e| format!("Serialization error: {}", e))?;

            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(canonical_str.as_bytes());
            let computed_hash = hex::encode(hasher.finalize());

            if computed_hash != log.hash {
                return Err(format!(
                    "Hash mismatch at entry {}: computed {} != stored {}",
                    log.id, computed_hash, log.hash
                ));
            }

            if prev_hash != log.prev_hash {
                return Err(format!(
                    "Chain break at entry {}: expected prev_hash {:?} != stored {:?}",
                    log.id, prev_hash, log.prev_hash
                ));
            }

            prev_hash = Some(log.hash.clone());
            cursor_id = Some(log.id);
            total_verified += 1;
        }

        if batch.len() < AUDIT_LOG_VERIFY_BATCH_SIZE as usize {
            break;
        }
    }

    Ok(total_verified)
}

/// Verify the integrity of the audit log chain (legacy in-memory version).
///
/// DEPRECATED: Use `verify_audit_log_chain_batched` for production use.
/// This version loads all logs into memory and should only be used for testing.
pub async fn verify_audit_log_chain(
    audit_logs: &[crate::models::audit_log::AuditLog],
) -> Result<usize, String> {
    if audit_logs.is_empty() {
        return Ok(0);
    }

    let mut sorted = audit_logs.to_vec();
    sorted.sort_by_key(|a| a.created_at);

    let mut verified = 0;
    let mut prev_hash: Option<String> = None;

    for log in sorted {
        let canonical = serde_json::json!({
            "actor_user_id": log.actor_user_id,
            "actor_role_snapshot": log.actor_role_snapshot,
            "action": log.action,
            "resource_type": log.resource_type,
            "resource_id": log.resource_id,
            "ip_address": log.ip_address.as_ref().map(|ip| ip.to_string()),
            "user_agent": log.user_agent,
            "request_id": log.request_id,
            "changes": log.changes,
            "metadata": log.metadata,
            "prev_hash": log.prev_hash,
        });

        let canonical_str =
            serde_json::to_string(&canonical).map_err(|e| format!("Serialization error: {}", e))?;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(canonical_str.as_bytes());
        let computed_hash = hex::encode(hasher.finalize());

        if computed_hash != log.hash {
            return Err(format!(
                "Hash mismatch at entry {}: computed {} != stored {}",
                log.id, computed_hash, log.hash
            ));
        }

        if prev_hash != log.prev_hash {
            return Err(format!(
                "Chain break at entry {}: expected prev_hash {:?} != stored {:?}",
                log.id, prev_hash, log.prev_hash
            ));
        }

        prev_hash = Some(log.hash.clone());
        verified += 1;
    }

    Ok(verified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    fn make_test_audit_log() -> NewAuditLog {
        NewAuditLog {
            actor_user_id: Some(Uuid::new_v4()),
            actor_role_snapshot: Some("admin".to_string()),
            action: "create".to_string(),
            resource_type: "User".to_string(),
            resource_id: Some(Uuid::new_v4()),
            ip_address: None,
            user_agent: Some("test-agent".to_string()),
            request_id: Some(Uuid::new_v4()),
            changes: json!({}),
            metadata: json!({}),
            prev_hash: None,
            hash: String::new(),
        }
    }

    #[test]
    fn test_compute_audit_log_hash_chain() {
        let item1 = make_test_audit_log();

        let (prev1, hash1) = compute_audit_log_hash(&item1, None);
        assert!(prev1.is_none());
        assert!(!hash1.is_empty());

        let mut item2 = make_test_audit_log();
        item2.action = "update".to_string();

        let (prev2, hash2) = compute_audit_log_hash(&item2, Some(&hash1));
        assert_eq!(prev2, Some(hash1.clone()));
        assert!(!hash2.is_empty());
        assert_ne!(hash1, hash2);

        // Verify chain integrity
        let (prev3, hash3) = compute_audit_log_hash(&item1, None);
        assert_eq!(hash3, hash1);
        assert_eq!(prev3, None);

        let (prev4, hash4) = compute_audit_log_hash(&item2, Some(&hash1));
        assert_eq!(hash4, hash2);
        assert_eq!(prev4, Some(hash1));
    }

    #[test]
    fn test_hash_chain_tamper_detection() {
        let item1 = make_test_audit_log();

        let (_, hash1) = compute_audit_log_hash(&item1, None);

        let mut item2 = make_test_audit_log();
        item2.action = "update".to_string();

        let (_, hash2) = compute_audit_log_hash(&item2, Some(&hash1));

        // Tamper with item2 by changing action
        let mut tampered = item2.clone();
        tampered.action = "delete".to_string();
        let (_, tampered_hash) = compute_audit_log_hash(&tampered, Some(&hash1));

        // Hash should be different
        assert_ne!(hash2, tampered_hash);
    }
}
