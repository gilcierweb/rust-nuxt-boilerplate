//! The JSON boundary must speak RFC 3339 for timestamps, matching the wire
//! format the frontend (and the previous chrono-based models) expect.
//!
//! `time::OffsetDateTime` serializes as a tuple of date/time components by
//! default; `crate::utils::wire_time` pins the RFC 3339 string format.

use serde_json::Value;
use time::macros::datetime;
use uuid::Uuid;

use backend::models::audit_log::AuditLog;
use backend::models::role::Role;

fn sample_role() -> Role {
    Role {
        id: Uuid::nil(),
        name: "admin".to_string(),
        resource_type: None,
        resource_id: None,
        created_at: datetime!(2026-11-20 15:37:36.177671 UTC),
        updated_at: datetime!(2026-11-20 15:37:36.177671 UTC),
    }
}

fn assert_rfc3339_string(value: &Value, field: &str) {
    let raw = value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{field} must serialize as a string, got {value}"));

    assert!(
        time::OffsetDateTime::parse(raw, &time::format_description::well_known::Rfc3339).is_ok(),
        "{field} must be RFC 3339, got {raw}"
    );
}

#[test]
fn role_timestamps_serialize_as_rfc3339() {
    let value = serde_json::to_value(sample_role()).expect("role must serialize");

    assert_rfc3339_string(&value, "created_at");
    assert_rfc3339_string(&value, "updated_at");
}

#[test]
fn audit_log_timestamp_serializes_as_rfc3339() {
    let log = AuditLog {
        id: Uuid::nil(),
        actor_user_id: None,
        actor_role_snapshot: None,
        action: "seed.create".to_string(),
        resource_type: "user".to_string(),
        resource_id: None,
        ip_address: None,
        user_agent: None,
        request_id: None,
        changes: serde_json::json!({}),
        metadata: serde_json::json!({}),
        created_at: datetime!(2026-11-20 15:37:36.177671 UTC),
        prev_hash: None,
        hash: "hash".to_string(),
    };

    let value = serde_json::to_value(log).expect("audit log must serialize");

    assert_rfc3339_string(&value, "created_at");
}
