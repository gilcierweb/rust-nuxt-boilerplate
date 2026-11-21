use serde::{Deserialize, Serialize};

use crate::utils::sanitize::{sanitize_input, strip_html};

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
            format!(
                r#"{{"error":{{"code":"INTERNAL","message":"{}"}}}}"#,
                t!("ws.internal_error").into_owned()
            )
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
            t!("ws.message_too_large", limit = MAX_WS_MESSAGE_BYTES).into_owned(),
        ));
    }

    if raw.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_MESSAGE",
            t!("ws.empty_message").into_owned(),
        ));
    }

    // Layer 2: JSON parse. The serde reason stays in the server log: it names the
    // shape of the internal request struct, which is reconnaissance for a
    // client probing the parser.
    let parsed: RawClientMessage = serde_json::from_str(raw).map_err(|e| {
        tracing::debug!(error = %e, "websocket message failed to parse");
        WsValidationError::new("INVALID_JSON", t!("ws.invalid_json").into_owned())
    })?;

    // Layer 3: action field validation
    let action_str = parsed.action.as_deref().ok_or_else(|| {
        WsValidationError::new(
            "MISSING_FIELD",
            t!("ws.missing_field", field = "action").into_owned(),
        )
    })?;

    if action_str.is_empty() {
        return Err(WsValidationError::new(
            "MISSING_ACTION",
            t!("ws.empty_action").into_owned(),
        ));
    }

    if action_str.len() > MAX_ACTION_LENGTH {
        return Err(WsValidationError::new(
            "ACTION_TOO_LONG",
            t!("ws.action_too_long", limit = MAX_ACTION_LENGTH).into_owned(),
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
        other => {
            tracing::debug!(action = %other, "websocket message named an unknown action");
            Err(WsValidationError::new(
                "UNKNOWN_ACTION",
                t!("ws.unknown_action").into_owned(),
            ))
        },
    }
}

/// The prefix of a room that belongs to one subject.
const USER_ROOM_PREFIX: &str = "user:";

/// Whether a subject may join a room.
///
/// A room name without an owner is a name any subject can guess, and the
/// broadcast of a room reaches every member of it: without this check one
/// authenticated subject reads another subject's chat by guessing the name.
/// A `user:`-prefixed room belongs to the subject it names and nobody else;
/// every other name is a project room, public by definition.
pub fn room_is_joinable(room: &str, subject: &uuid::Uuid) -> bool {
    if let Some(owner) = room.strip_prefix(USER_ROOM_PREFIX) {
        return owner == subject.to_string();
    }
    true
}

fn validate_join_room(data: serde_json::Value) -> Result<WsClientAction, WsValidationError> {
    let room = data.get("room").and_then(|v| v.as_str()).ok_or_else(|| {
        WsValidationError::new(
            "MISSING_FIELD",
            t!("ws.missing_field", field = "data.room").into_owned(),
        )
    })?;

    if room.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_FIELD",
            t!("ws.empty_field", field = "data.room").into_owned(),
        ));
    }

    let room = sanitize_input(&strip_html(room));

    if room.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_FIELD",
            t!("ws.empty_field", field = "data.room").into_owned(),
        ));
    }

    if room.len() > MAX_ROOM_NAME_LENGTH {
        return Err(WsValidationError::new(
            "FIELD_TOO_LONG",
            t!(
                "ws.field_too_long",
                field = "data.room",
                limit = MAX_ROOM_NAME_LENGTH
            )
            .into_owned(),
        ));
    }

    Ok(WsClientAction::JoinRoom { room })
}

fn validate_chat(data: serde_json::Value) -> Result<WsClientAction, WsValidationError> {
    let content = data
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            WsValidationError::new(
                "MISSING_FIELD",
                t!("ws.missing_field", field = "data.content").into_owned(),
            )
        })?;

    if content.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_FIELD",
            t!("ws.empty_field", field = "data.content").into_owned(),
        ));
    }

    let content = sanitize_input(&strip_html(content));

    if content.is_empty() {
        return Err(WsValidationError::new(
            "EMPTY_FIELD",
            t!("ws.empty_field", field = "data.content").into_owned(),
        ));
    }

    if content.len() > MAX_CHAT_CONTENT_LENGTH {
        return Err(WsValidationError::new(
            "FIELD_TOO_LONG",
            t!(
                "ws.field_too_long",
                field = "data.content",
                limit = MAX_CHAT_CONTENT_LENGTH
            )
            .into_owned(),
        ));
    }

    Ok(WsClientAction::Chat { content })
}

/// Validate that `data` is a JSON object (used for actions that accept optional fields).
fn validate_data_is_object(data: &serde_json::Value) -> Result<(), WsValidationError> {
    if !data.is_object() {
        return Err(WsValidationError::new(
            "INVALID_DATA_TYPE",
            t!("ws.invalid_data_type").into_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_parse_failure_does_not_describe_the_internals_to_the_client() {
        // The serde reason names the expected schema of the internal request
        // struct: free reconnaissance for a client probing the parser.
        let error = validate_client_message(r#"{"action":42}"#).unwrap_err();
        assert_eq!(error.error.code, "INVALID_JSON");
        for leak in ["expected", "invalid type", "at line", "column", "struct"] {
            assert!(
                !error.error.message.contains(leak),
                "the message must not carry the parse reason ({leak}): {}",
                error.error.message
            );
        }
    }

    #[test]
    fn an_unknown_action_is_not_echoed_back() {
        // The unknown action is the client's own probe string: echoing it
        // confirms the endpoint reached the action dispatcher.
        let error = validate_client_message(r#"{"action":"probe_action","data":{}}"#).unwrap_err();
        assert_eq!(error.error.code, "UNKNOWN_ACTION");
        assert!(
            !error.error.message.contains("probe_action"),
            "the message must not echo the unknown action: {}",
            error.error.message
        );
    }

    #[test]
    fn the_parse_and_action_messages_carry_no_interpolated_detail() {
        // The unit tests run without the i18n catalog loaded, so the message
        // assertions above cannot see the rendered strings. This reads the
        // catalog itself: a placeholder in the template is the only way a
        // detail (the serde reason, the probed action) can reach a client.
        for locale in ["en", "es", "pt-BR"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("locales")
                .join(format!("{locale}.json"));
            let catalog: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&path).expect("the locale file exists"),
            )
            .expect("the locale file is JSON");
            for key in ["ws.invalid_json", "ws.unknown_action"] {
                let message = catalog
                    .get(key)
                    .unwrap_or_else(|| panic!("{key} must exist in {locale}"))
                    .as_str()
                    .expect("the message is a string");
                assert!(
                    !message.contains("{}"),
                    "{locale}: {key} must not interpolate a detail: {message}"
                );
            }
        }
    }

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
        let err = validate_client_message(r#"{"action":"fly_to_moon","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "UNKNOWN_ACTION");
    }

    #[test]
    fn join_room_missing_room_rejected() {
        let err = validate_client_message(r#"{"action":"join_room","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn join_room_empty_room_rejected() {
        let err =
            validate_client_message(r#"{"action":"join_room","data":{"room":""}}"#).unwrap_err();
        assert_eq!(err.error.code, "EMPTY_FIELD");
    }

    #[test]
    fn join_room_long_room_rejected() {
        let long_room = "a".repeat(MAX_ROOM_NAME_LENGTH + 1);
        let msg = format!(
            r#"{{"action":"join_room","data":{{"room":"{}"}}}}"#,
            long_room
        );
        let err = validate_client_message(&msg).unwrap_err();
        assert_eq!(err.error.code, "FIELD_TOO_LONG");
    }

    #[test]
    fn join_room_room_must_be_string() {
        let err =
            validate_client_message(r#"{"action":"join_room","data":{"room":123}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn chat_missing_content_rejected() {
        let err = validate_client_message(r#"{"action":"chat","data":{}}"#).unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn chat_empty_content_rejected() {
        let err =
            validate_client_message(r#"{"action":"chat","data":{"content":""}}"#).unwrap_err();
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
        let err =
            validate_client_message(r#"{"action":"chat","data":{"content":42}}"#).unwrap_err();
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
        let err = validate_client_message(r#"{"action":"join_room","data":"not-an-object"}"#)
            .unwrap_err();
        assert_eq!(err.error.code, "MISSING_FIELD");
    }

    #[test]
    fn chat_strips_html_from_content() {
        let msg = r#"{"action":"chat","data":{"content":"<script>alert(1)</script>hello"}}"#;
        let result = validate_client_message(msg).unwrap();
        match result {
            WsClientAction::Chat { content } => {
                assert!(!content.contains("<script>"));
                assert!(content.contains("hello"));
            },
            _ => panic!("expected Chat"),
        }
    }

    #[test]
    fn chat_rejects_html_only_content() {
        let msg = r#"{"action":"chat","data":{"content":"<img src=x onerror=alert(1)>"}}"#;
        let err = validate_client_message(msg).unwrap_err();
        assert_eq!(err.error.code, "EMPTY_FIELD");
    }

    #[test]
    fn join_room_strips_html_from_room_name() {
        let msg = r#"{"action":"join_room","data":{"room":"<b>general</b>"}}"#;
        let result = validate_client_message(msg).unwrap();
        match result {
            WsClientAction::JoinRoom { room } => {
                assert!(!room.contains("<b>"));
                assert!(room.contains("general"));
            },
            _ => panic!("expected JoinRoom"),
        }
    }

    #[test]
    fn a_subject_joins_the_room_that_names_it() {
        let subject = uuid::Uuid::new_v4();
        assert!(room_is_joinable(&format!("user:{subject}"), &subject));
    }

    #[test]
    fn a_subject_cannot_join_the_room_of_another_subject() {
        // The room broadcast reaches every member: guessing a name must not
        // open another subject's chat.
        let subject = uuid::Uuid::new_v4();
        let other = uuid::Uuid::new_v4();
        assert!(!room_is_joinable(&format!("user:{other}"), &subject));
    }

    #[test]
    fn a_project_room_stays_public_and_a_bare_prefix_is_nobodys() {
        let subject = uuid::Uuid::new_v4();
        assert!(room_is_joinable("general", &subject));
        assert!(!room_is_joinable("user:", &subject));
    }
}
