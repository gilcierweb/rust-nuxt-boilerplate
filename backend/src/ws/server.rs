#![allow(dead_code)]

use actix::prelude::*;
use actix_web::{HttpRequest, HttpResponse, http::header::SEC_WEBSOCKET_PROTOCOL, web};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::AppState;
use crate::errors::{AppError, AppResult};

const WS_PROTOCOL: &str = "justfans-ws";
const WS_AUTH_PROTOCOL_PREFIX: &str = "auth.";

/// Shared state for WebSocket connections
pub type WsConnections = Arc<TokioMutex<HashMap<String, ConnectionInfo>>>;
pub type WsPresence = Arc<TokioMutex<HashMap<Uuid, PresenceInfo>>>;

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub profile_id: Uuid,
    pub username: String,
    pub room: Option<String>,
    pub addr: Recipient<WsMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub active_connections: usize,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// WebSocket state container
#[derive(Clone)]
pub struct WsState {
    pub connections: WsConnections,
    pub presence: WsPresence,
}

impl WsState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(TokioMutex::new(HashMap::new())),
            presence: Arc::new(TokioMutex::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, conn_id: String, info: ConnectionInfo) {
        let should_broadcast_online = {
            let mut conns = self.connections.lock().await;
            let is_new_connection = conns.insert(conn_id, info.clone()).is_none();
            drop(conns);

            if !is_new_connection {
                false
            } else {
                let mut presence = self.presence.lock().await;
                let entry = presence.entry(info.profile_id).or_insert(PresenceInfo {
                    active_connections: 0,
                    last_seen_at: None,
                });
                entry.active_connections += 1;
                entry.last_seen_at = None;
                entry.active_connections == 1
            }
        };

        if should_broadcast_online {
            self.broadcast_presence_change(info.profile_id, true, None).await;
        }
    }

    pub async fn remove_connection(&self, conn_id: &str) {
        let removed_profile_id = {
            let mut conns = self.connections.lock().await;
            conns.remove(conn_id).map(|info| info.profile_id)
        };

        let Some(profile_id) = removed_profile_id else {
            return;
        };

        let last_seen_at = {
            let mut presence = self.presence.lock().await;
            let Some(entry) = presence.get_mut(&profile_id) else {
                return;
            };

            if entry.active_connections > 0 {
                entry.active_connections -= 1;
            }

            if entry.active_connections == 0 {
                let seen_at = Utc::now();
                entry.last_seen_at = Some(seen_at);
                Some(seen_at)
            } else {
                None
            }
        };

        if let Some(seen_at) = last_seen_at {
            self.broadcast_presence_change(profile_id, false, Some(seen_at)).await;
        }
    }

    pub async fn broadcast_to_room(&self, room: &str, message: WsMessage) {
        let conns = self.connections.lock().await;
        for (_, info) in conns.iter() {
            if info.room.as_ref() == Some(&room.to_string()) {
                let _ = info.addr.try_send(message.clone());
            }
        }
    }

    pub async fn broadcast_to_room_except(
        &self,
        room: &str,
        excluded_profile_id: Uuid,
        message: WsMessage,
    ) {
        let conns = self.connections.lock().await;
        for (_, info) in conns.iter() {
            if info.room.as_ref() == Some(&room.to_string())
                && info.profile_id != excluded_profile_id
            {
                let _ = info.addr.try_send(message.clone());
            }
        }
    }

    pub async fn send_to_user(&self, profile_id: Uuid, message: WsMessage) {
        tracing::debug!(target_profile_id = %profile_id, "Dispatching websocket message");
        let conns = self.connections.lock().await;
        for (conn_id, info) in conns.iter() {
            if info.profile_id == profile_id {
                tracing::debug!(connection_id = %conn_id, "Websocket target connection found");
                let _ = info.addr.try_send(message.clone());
            }
        }
    }

    pub async fn get_presence(&self, profile_id: Uuid) -> PresenceInfo {
        let presence = self.presence.lock().await;
        presence.get(&profile_id).cloned().unwrap_or(PresenceInfo {
            active_connections: 0,
            last_seen_at: None,
        })
    }

    pub async fn is_user_online(&self, profile_id: Uuid) -> bool {
        self.get_presence(profile_id).await.active_connections > 0
    }

    async fn broadcast_presence_change(
        &self,
        profile_id: Uuid,
        is_online: bool,
        last_seen_at: Option<DateTime<Utc>>,
    ) {
        let message = WsMessage::new(
            "presence_changed",
            serde_json::json!({
                "profile_id": profile_id,
                "is_online": is_online,
                "last_seen_at": last_seen_at,
            }),
        );
        self.broadcast_to_all(message).await;
    }

    async fn broadcast_to_all(&self, message: WsMessage) {
        let conns = self.connections.lock().await;
        for (_, info) in conns.iter() {
            let _ = info.addr.try_send(message.clone());
        }
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket message types
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct WsMessage {
    pub msg_type: String,
    pub payload: serde_json::Value,
    pub sender: Option<String>,
    pub timestamp: i64,
}

impl WsMessage {
    pub fn new(msg_type: &str, payload: serde_json::Value) -> Self {
        Self {
            msg_type: msg_type.to_string(),
            payload,
            sender: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn chat(content: &str, sender: &str) -> Self {
        let payload = serde_json::json!({
            "content": content,
            "sender": sender,
        });
        let mut msg = Self::new("chat", payload);
        msg.sender = Some(sender.to_string());
        msg
    }

    pub fn live_status(stream_id: &str, is_live: bool, viewer_count: i32) -> Self {
        let payload = serde_json::json!({
            "stream_id": stream_id,
            "is_live": is_live,
            "viewer_count": viewer_count,
        });
        Self::new("live_status", payload)
    }

    pub fn notification(title: &str, message: &str) -> Self {
        let payload = serde_json::json!({
            "title": title,
            "message": message,
        });
        Self::new("notification", payload)
    }
}

/// WebSocket actor for handling connections
pub struct WebSocketActor {
    pub conn_id: String,
    pub profile_id: Uuid,
    pub username: String,
    pub room: Option<String>,
    pub ws_state: web::Data<WsState>,
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let conn_info = ConnectionInfo {
            profile_id: self.profile_id,
            username: self.username.clone(),
            room: self.room.clone(),
            addr: ctx.address().recipient(),
        };
        let ws_state = self.ws_state.clone();
        let conn_id = self.conn_id.clone();
        let fut = async move {
            ws_state.add_connection(conn_id, conn_info).await;
        };
        ctx.spawn(actix::fut::wrap_future(fut));

        tracing::info!(
            "WebSocket connected: {} (profile: {})",
            self.conn_id,
            self.profile_id
        );
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let ws_state = self.ws_state.clone();
        let conn_id = self.conn_id.clone();
        let fut = async move {
            ws_state.remove_connection(&conn_id).await;
        };
        ctx.spawn(actix::fut::wrap_future(fut));
        tracing::info!("WebSocket disconnected: {}", self.conn_id);
    }
}

/// Handle messages from other actors
impl Handler<WsMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        let text = serde_json::to_string(&msg).unwrap_or_default();
        ctx.text(text);
    }
}

/// Handle WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => self.handle_message(&text, ctx),
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WebSocketActor {
    fn handle_message(&mut self, text: &str, ctx: &mut ws::WebsocketContext<Self>) {
        match serde_json::from_str::<ClientMessage>(text) {
            Ok(msg) => match msg.action.as_str() {
                "join_room" => {
                    if let Some(room) = msg.data.get("room").and_then(|v| v.as_str()) {
                        self.room = Some(room.to_string());
                        // Update connection info
                        let conn_info = ConnectionInfo {
                            profile_id: self.profile_id,
                            username: self.username.clone(),
                            room: self.room.clone(),
                            addr: ctx.address().recipient(),
                        };
                        let ws_state = self.ws_state.clone();
                        let conn_id = self.conn_id.clone();
                        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                            ws_state.add_connection(conn_id, conn_info).await;
                        }));

                        let response = WsMessage::new(
                            "joined_room",
                            serde_json::json!({
                                "room": room,
                                "user": self.username,
                            }),
                        );
                        ctx.text(serde_json::to_string(&response).unwrap_or_default());
                    }
                }
                "leave_room" => {
                    self.room = None;
                    let conn_info = ConnectionInfo {
                        profile_id: self.profile_id,
                        username: self.username.clone(),
                        room: None,
                        addr: ctx.address().recipient(),
                    };
                    let ws_state = self.ws_state.clone();
                    let conn_id = self.conn_id.clone();
                    ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                        ws_state.add_connection(conn_id, conn_info).await;
                    }));
                }
                "chat" => {
                    if let (Some(room), Some(content)) =
                        (&self.room, msg.data.get("content").and_then(|v| v.as_str()))
                    {
                        let chat_msg = WsMessage::chat(content, &self.username);
                        let ws_state = self.ws_state.clone();
                        let room = room.to_string();
                        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                            ws_state.broadcast_to_room(&room, chat_msg).await;
                        }));
                    }
                }
                "typing" => {
                    if let Some(room) = &self.room {
                        let typing_msg = WsMessage::new(
                            "typing",
                            serde_json::json!({
                                "user": self.username,
                                "room": room,
                            }),
                        );
                        let ws_state = self.ws_state.clone();
                        let room = room.clone();
                        let profile_id = self.profile_id;
                        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                            ws_state.broadcast_to_room_except(&room, profile_id, typing_msg).await;
                        }));
                    }
                }
                "stop_typing" => {
                    if let Some(room) = &self.room {
                        let stop_typing_msg = WsMessage::new(
                            "stop_typing",
                            serde_json::json!({
                                "user": self.username,
                                "room": room,
                            }),
                        );
                        let ws_state = self.ws_state.clone();
                        let room = room.clone();
                        let profile_id = self.profile_id;
                        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                            ws_state.broadcast_to_room_except(&room, profile_id, stop_typing_msg).await;
                        }));
                    }
                }
                "ping" => {
                    let pong = WsMessage::new(
                        "pong",
                        serde_json::json!({
                            "timestamp": chrono::Utc::now().timestamp(),
                        }),
                    );
                    ctx.text(serde_json::to_string(&pong).unwrap_or_default());
                }
                _ => {
                    tracing::warn!(
                        connection_id = %self.conn_id,
                        action = %msg.action,
                        "Unknown WebSocket action"
                    );
                }
            },
            Err(e) => {
                tracing::error!(
                    connection_id = %self.conn_id,
                    error = %e,
                    "Failed to parse WebSocket message"
                );
            }
        }
    }
}

/// Client message structure
#[derive(Debug, Deserialize)]
struct ClientMessage {
    action: String,
    data: serde_json::Value,
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    ws_state: web::Data<WsState>,
) -> AppResult<HttpResponse> {
    let conn_id = Uuid::new_v4().to_string();

    let token = extract_ws_protocol_token(&req);

    let profile_id = match token {
        Some(t) => {
            let state = req.app_data::<web::Data<AppState>>();
            let secret = state
                .as_ref()
                .map(|s| s.config.jwt_secret.clone())
                .unwrap_or_default();

            match crate::middleware::auth::verify_ws_token(&t, &secret) {
                Ok(claims) => claims.profile_id,
                Err(_) => {
                    tracing::warn!("WebSocket auth failed");
                    return Err(AppError::Unauthorized("Invalid token".to_string()));
                }
            }
        }
        None => {
            return Err(AppError::Unauthorized("Missing token".to_string()));
        }
    };

    tracing::info!(
        "WebSocket connection: {} (profile: {})",
        conn_id,
        profile_id
    );

    let ws_actor = WebSocketActor {
        conn_id,
        profile_id,
        username: profile_id.to_string(),
        room: None,
        ws_state,
    };

    ws::WsResponseBuilder::new(ws_actor, &req, stream)
        .protocols(&[WS_PROTOCOL])
        .start()
        .map_err(|e| AppError::Internal(format!("WebSocket error: {}", e)))
}

fn extract_ws_protocol_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(SEC_WEBSOCKET_PROTOCOL)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            header
                .split(',')
                .map(str::trim)
                .find_map(|value| value.strip_prefix(WS_AUTH_PROTOCOL_PREFIX))
        })
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::{WS_PROTOCOL, extract_ws_protocol_token};
    use actix_web::{http::header::SEC_WEBSOCKET_PROTOCOL, test::TestRequest};

    #[test]
    fn extracts_websocket_token_from_subprotocol_header() {
        let req = TestRequest::default()
            .insert_header((
                SEC_WEBSOCKET_PROTOCOL,
                format!("{}, auth.test-token", WS_PROTOCOL),
            ))
            .to_http_request();

        assert_eq!(
            extract_ws_protocol_token(&req).as_deref(),
            Some("test-token")
        );
    }

    #[test]
    fn ignores_missing_auth_subprotocol() {
        let req = TestRequest::default()
            .insert_header((SEC_WEBSOCKET_PROTOCOL, WS_PROTOCOL))
            .to_http_request();

        assert!(extract_ws_protocol_token(&req).is_none());
    }

    #[cfg(test)]
    mod ws_state_tests {
        use super::super::*;

        #[actix_web::test]
        async fn ws_state_new_creates_empty_state() {
            let state = WsState::new();
            let conns = state.connections.lock().await;
            assert!(conns.is_empty());
        }

        #[actix_web::test]
        async fn remove_nonexistent_connection_is_noop() {
            let state = WsState::new();
            state.remove_connection("nonexistent").await;
            assert!(state.connections.lock().await.is_empty());
        }

        #[actix_web::test]
        async fn user_with_no_connections_is_not_online() {
            let state = WsState::new();
            let profile_id = Uuid::new_v4();
            assert!(!state.is_user_online(profile_id).await);
            let presence = state.get_presence(profile_id).await;
            assert_eq!(presence.active_connections, 0);
            assert!(presence.last_seen_at.is_none());
        }

        #[test]
        fn ws_message_chat_factory() {
            let msg = WsMessage::chat("hello", "user1");
            assert_eq!(msg.msg_type, "chat");
            assert_eq!(msg.sender.as_deref(), Some("user1"));
            assert_eq!(msg.payload["content"], "hello");
        }

        #[test]
        fn ws_message_notification_factory() {
            let msg = WsMessage::notification("Title", "Body");
            assert_eq!(msg.msg_type, "notification");
            assert_eq!(msg.payload["title"], "Title");
            assert_eq!(msg.payload["message"], "Body");
        }

        #[test]
        fn ws_message_live_status_factory() {
            let msg = WsMessage::live_status("stream-1", true, 42);
            assert_eq!(msg.msg_type, "live_status");
            assert_eq!(msg.payload["stream_id"], "stream-1");
            assert_eq!(msg.payload["is_live"], true);
            assert_eq!(msg.payload["viewer_count"], 42);
        }

        #[test]
        fn ws_message_new_sets_timestamp() {
            let msg = WsMessage::new("test", serde_json::json!({}));
            assert!(msg.timestamp > 0);
            assert_eq!(msg.msg_type, "test");
            assert!(msg.sender.is_none());
        }
    }

    #[cfg(test)]
    mod ws_state_actor_tests {
        use super::super::*;
        use actix::Actor;
        use actix::Addr;
        use actix::Context;

        struct DummyActor;

        impl Actor for DummyActor {
            type Context = Context<Self>;
        }

        impl actix::Handler<WsMessage> for DummyActor {
            type Result = ();
            fn handle(&mut self, _msg: WsMessage, _ctx: &mut Self::Context) {}
        }

        fn dummy_recipient() -> actix::Recipient<WsMessage> {
            let addr: Addr<DummyActor> = DummyActor.start();
            addr.recipient()
        }

        fn make_conn_info(profile_id: Uuid, room: Option<&str>) -> ConnectionInfo {
            ConnectionInfo {
                profile_id,
                username: "test_user".to_string(),
                room: room.map(|r| r.to_string()),
                addr: dummy_recipient(),
            }
        }

        #[actix_web::test]
        async fn add_and_remove_connection() {
            let state = WsState::new();
            let profile_id = Uuid::new_v4();
            let conn_id = "conn-1".to_string();

            state.add_connection(conn_id.clone(), make_conn_info(profile_id, None)).await;
            {
                let conns = state.connections.lock().await;
                assert!(conns.contains_key(&conn_id));
            }

            state.remove_connection(&conn_id).await;
            {
                let conns = state.connections.lock().await;
                assert!(!conns.contains_key(&conn_id));
            }
        }

        #[actix_web::test]
        async fn presence_tracks_active_connections() {
            let state = WsState::new();
            let profile_id = Uuid::new_v4();

            state.add_connection("c1".to_string(), make_conn_info(profile_id, None)).await;
            state.add_connection("c2".to_string(), make_conn_info(profile_id, None)).await;

            let presence = state.get_presence(profile_id).await;
            assert_eq!(presence.active_connections, 2);
            assert!(state.is_user_online(profile_id).await);
        }

        #[actix_web::test]
        async fn presence_decrements_on_remove() {
            let state = WsState::new();
            let profile_id = Uuid::new_v4();

            state.add_connection("c1".to_string(), make_conn_info(profile_id, None)).await;
            state.add_connection("c2".to_string(), make_conn_info(profile_id, None)).await;
            state.remove_connection("c1").await;

            let presence = state.get_presence(profile_id).await;
            assert_eq!(presence.active_connections, 1);
            assert!(state.is_user_online(profile_id).await);
        }

        #[actix_web::test]
        async fn last_connection_sets_last_seen() {
            let state = WsState::new();
            let profile_id = Uuid::new_v4();

            state.add_connection("c1".to_string(), make_conn_info(profile_id, None)).await;
            state.remove_connection("c1").await;

            let presence = state.get_presence(profile_id).await;
            assert_eq!(presence.active_connections, 0);
            assert!(presence.last_seen_at.is_some());
            assert!(!state.is_user_online(profile_id).await);
        }
    }
}
