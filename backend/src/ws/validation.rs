use serde::{Deserialize, Serialize};

/// Maximum raw WebSocket message size in bytes (64 KiB).
pub const MAX_WS_MESSAGE_BYTES: usize = 64 * 1024;

/// Maximum `action` field length.
pub const MAX_ACTION_LENGTH: usize = 50;

/// Maximum room name length.
pub const MAX_ROOM_NAME_LENGTH: usize = 100;

/// Maximum chat message content length.
pub const MAX_CHAT_CONTENT_LENGTH: usize = 10_000;

/// Error response sent back to the client on validation failure.
#[derive(Debug, Clone, Serialize)]
pub struct WsValidationError {
    pub error: WsErrorBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsErrorBody {
    pub code: &'static str,
    pub message: String,
}

impl WsValidationError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            error: WsErrorBody {
                code,
                message: message.into(),
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"error":{"code":"INTERNAL","message":"validation error"}}"#.to_string()
        })
    }
}

/// Raw client message before validation.
#[derive(Debug, Deserialize)]
pub struct RawClientMessage {
    pub action: Option<String>,
    pub data: serde_json::Value,
}

/// Validated client message — each variant carries typed fields.
#[derive(Debug, Clone)]
pub enum WsClientAction {
    JoinRoom { room: String },
    LeaveRoom,
    Chat { content: String },
    Typing,
    StopTyping,
    Ping,
}

/// Validate and parse a raw WebSocket text message into a typed action.
///
/// # Validation layers
/// 1. Size check — reject messages exceeding `MAX_WS_MESSAGE_BYTES`
/// 2. JSON parse — reject malformed JSON
/// 3. Action field — non-empty, within length limit, known action
/// 4. Per-action data schema — required fields present, types correct, lengths valid
pub fn validate_client_message(raw: &str) -> Result<WsClientAction, WsValidationError> {
    // Layer 1: size check
    if raw.len() > MAX_WS_MESSAGE_BYTES {
        return Err(WsValidationError::new(
            "MESSAGE_TOO_LARGE",
            format!(
                "Message exceeds {} bytes limit",
                MAX_WS_MESSAGE_BYTES
            ),
        ));
    }

    if raw.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_MESSAGE",
            "Message must not be empty",
        ));
    }

    // Layer 2: JSON parse
    let parsed: RawClientMessage = serde_json::from_str(raw).map_err(|e| {
        WsValidationError::new(
            "INVALID_JSON",
            format!("Malformed JSON: {}", e),
        )
    })?;

    // Layer 3: action field validation
    let action_str = parsed.action.as_deref().ok_or_else(|| {
        WsValidationError::new(
            "MISSING_FIELD",
            "Field 'action' is required",
        )
    })?;

    if action_str.is_empty() {
        return Err(WsValidationError::new(
            "MISSING_ACTION",
            "Field 'action' must not be empty",
        ));
    }

    if action_str.len() > MAX_ACTION_LENGTH {
        return Err(WsValidationError::new(
            "ACTION_TOO_LONG",
            format!(
                "Field 'action' exceeds {} characters",
                MAX_ACTION_LENGTH
            ),
        ));
    }

    // Layer 4: per-action schema validation
    match action_str {
        "join_room" => validate_join_room(parsed.data),
        "leave_room" => Ok(WsClientAction::LeaveRoom),
        "chat" => validate_chat(parsed.data),
        "typing" => Ok(WsClientAction::Typing),
        "stop_typing" => Ok(WsClientAction::StopTyping),
        "ping" => Ok(WsClientAction::Ping),
        other => Err(WsValidationError::new(
            "UNKNOWN_ACTION",
            format!("Unknown action: '{}'", other),
        )),
    }
}

fn validate_join_room(data: serde_json::Value) -> Result<WsClientAction, WsValidationError> {
    let room = data
        .get("room")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            WsValidationError::new(
                "MISSING_FIELD",
                "Field 'data.room' is required and must be a string",
            )
        })?;

    if room.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_FIELD",
            "Field 'data.room' must not be empty",
        ));
    }

    if room.len() > MAX_ROOM_NAME_LENGTH {
        return Err(WsValidationError::new(
            "FIELD_TOO_LONG",
            format!(
                "Field 'data.room' exceeds {} characters",
                MAX_ROOM_NAME_LENGTH
            ),
        ));
    }

    Ok(WsClientAction::JoinRoom {
        room: room.to_string(),
    })
}

fn validate_chat(data: serde_json::Value) -> Result<WsClientAction, WsValidationError> {
    let content = data
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            WsValidationError::new(
                "MISSING_FIELD",
                "Field 'data.content' is required and must be a string",
            )
        })?;

    if content.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_FIELD",
            "Field 'data.content' must not be empty",
        ));
    }

    if content.len() > MAX_CHAT_CONTENT_LENGTH {
        return Err(WsValidationError::new(
            "FIELD_TOO_LONG",
            format!(
                "Field 'data.content' exceeds {} characters",
                MAX_CHAT_CONTENT_LENGTH
            ),
        ));
    }

    Ok(WsClientAction::Chat {
        content: content.to_string(),
    })
}

/// Validate that `data` is a JSON object (used for actions that accept optional fields).
fn validate_data_is_object(data: &serde_json::Value) -> Result<(), WsValidationError> {
    if !data.is_object() {
        return Err(WsValidationError::new(
            "INVALID_DATA_TYPE",
            "Field 'data' must be a JSON object",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_join_room() {
        let msg = r#"{"action":"join_room","data":{"room":"general"}}"#;
        let result = validate_client_message(msg).unwrap();
        match result {
            WsClientAction::JoinRoom { room } => assert_eq!(room, "general"),
            _ => panic!("expected JoinRoom"),
        }
    }

    #[test]
    fn valid_leave_room() {
        let msg = r#"{"action":"leave_room","data":{}}"#;
        assert!(matches!(
            validate_client_message(msg).unwrap(),
            WsClientAction::LeaveRoom
        ));
    }

    #[test]
    fn valid_chat() {
        let msg = r#"{"action":"chat","data":{"content":"hello world"}}"#;
        let result = validate_client_message(msg).unwrap();
        match result {
            WsClientAction::Chat { content } => assert_eq!(content, "hello world"),
            _ => panic!("expected Chat"),
        }
    }

    #[test]
    fn valid_typing() {
        let msg = r#"{"action":"typing","data":{}}"#;
        assert!(matches!(
            validate_client_message(msg).unwrap(),
            WsClientAction::Typing
        ));
    }

    #[test]
    fn valid_stop_typing() {
        let msg = r#"{"action":"stop_typing","data":{}}"#;
        assert!(matches!(
            validate_client_message(msg).unwrap(),
            WsClientAction::StopTyping
        ));
    }

    #[test]
    fn valid_ping() {
        let msg = r#"{"action":"ping","data":{}}"#;
        assert!(matches!(
            validate_client_message(msg).unwrap(),
            WsClientAction::Ping
        ));
    }

    #[test]
    fn empty_message_rejected() {
        let err = validate_client_message("").unwrap_err();
        assert_eq!(err.error.code, "EMPTY_MESSAGE");
    }

    #[test]
    fn oversized_message_rejected() {
        let big = "x".repeat(MAX_WS_MESSAGE_BYTES + 1);
        let err = validate_client_message(&big).unwrap_err();
        assert_eq!(err.error.code, "MESSAGE_TOO_LARGE");
    }

    #[test]
    fn malformed_json_rejected() {
        let err = validate_client_message("{invalid}").unwrap_err();
        assert_eq!(err.error.code, "INVALID_JSON");
    }

    #[test]
    fn missing_action_rejected() {
        let err = validate_client_message(r#"{"data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn empty_action_rejected() {
        let err = validate_client_message(r#"{"action":"","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_ACTION");
    }

    #[test]
    fn long_action_rejected() {
        let long_action = "a".repeat(MAX_ACTION_LENGTH + 1);
        let msg = format!(r#"{{"action":"{}","data":{{}}}}"#, long_action);
        let err = validate_client_message(&msg).unwrap_err();
        assert_eq!(err.error.code, "ACTION_TOO_LONG");
    }

    #[test]
    fn unknown_action_rejected() {
        let err =
            validate_client_message(r#"{"action":"fly_to_moon","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "UNKNOWN_ACTION");
    }

    #[test]
    fn join_room_missing_room_rejected() {
        let err =
            validate_client_message(r#"{"action":"join_room","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn join_room_empty_room_rejected() {
        let err = validate_client_message(
            r#"{"action":"join_room","data":{"room":""}}"#,
        )
        .unwrap_err();
        assert_eq!(err.error.code, "EMPTY_FIELD");
    }

    #[test]
    fn join_room_long_room_rejected() {
        let long_room = "a".repeat(MAX_ROOM_NAME_LENGTH + 1);
        let msg = format!(r#"{{"action":"join_room","data":{{"room":"{}"}}}}"#, long_room);
        let err = validate_client_message(&msg).unwrap_err();
        assert_eq!(err.error.code, "FIELD_TOO_LONG");
    }

    #[test]
    fn join_room_room_must_be_string() {
        let err = validate_client_message(
            r#"{"action":"join_room","data":{"room":123}}"#,
        )
        .unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn chat_missing_content_rejected() {
        let err =
            validate_client_message(r#"{"action":"chat","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn chat_empty_content_rejected() {
        let err = validate_client_message(
            r#"{"action":"chat","data":{"content":""}}"#,
        )
        .unwrap_err();
        assert_eq!(err.error.code, "EMPTY_FIELD");
    }

    #[test]
    fn chat_long_content_rejected() {
        let long_content = "a".repeat(MAX_CHAT_CONTENT_LENGTH + 1);
        let msg = format!(
            r#"{{"action":"chat","data":{{"content":"{}"}}}}"#,
            long_content
        );
        let err = validate_client_message(&msg).unwrap_err();
        assert_eq!(err.error.code, "FIELD_TOO_LONG");
    }

    #[test]
    fn chat_content_must_be_string() {
        let err = validate_client_message(
            r#"{"action":"chat","data":{"content":42}}"#,
        )
        .unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn error_response_is_valid_json() {
        let err = validate_client_message("").unwrap_err();
        let json = err.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[test]
    fn valid_action_borderline_length() {
        let borderline_action = "a".repeat(MAX_ACTION_LENGTH);
        let msg = format!(r#"{{"action":"{}","data":{{}}}}"#, borderline_action);
        let err = validate_client_message(&msg).unwrap_err();
        assert_eq!(err.error.code, "UNKNOWN_ACTION");
    }

    #[test]
    fn valid_room_borderline_length() {
        let borderline_room = "a".repeat(MAX_ROOM_NAME_LENGTH);
        let msg = format!(
            r#"{{"action":"join_room","data":{{"room":"{}"}}}}"#,
            borderline_room
        );
        assert!(validate_client_message(&msg).is_ok());
    }

    #[test]
    fn valid_chat_borderline_length() {
        let borderline_content = "a".repeat(MAX_CHAT_CONTENT_LENGTH);
        let msg = format!(
            r#"{{"action":"chat","data":{{"content":"{}"}}}}"#,
            borderline_content
        );
        assert!(validate_client_message(&msg).is_ok());
    }

    #[test]
    fn data_must_be_object_for_actions_needing_data() {
        let err = validate_client_message(
            r#"{"action":"join_room","data":"not-an-object"}"#,
        )
        .unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }
}
