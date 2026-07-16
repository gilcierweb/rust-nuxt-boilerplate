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

/// WebSocket connection limits configuration.
///
/// # Defaults
/// - `max_connections_per_ip`: 10 — prevents a single IP from exhausting file descriptors
/// - `max_global_connections`: 10_000 — hard cap on total concurrent WebSocket connections
/// - `heartbeat_interval_secs`: 30 — ping interval in seconds
/// - `client_timeout_secs`: 60 — max time without a pong before disconnecting
#[derive(Clone, Debug)]
pub struct WsLimits {
    pub max_connections_per_ip: usize,
    pub max_global_connections: usize,
    pub heartbeat_interval_secs: u64,
    pub client_timeout_secs: u64,
}

impl Default for WsLimits {
    fn default() -> Self {
        Self {
            max_connections_per_ip: 10,
            max_global_connections: 10_000,
            heartbeat_interval_secs: 30,
            client_timeout_secs: 60,
        }
    }
}

/// Shared state for WebSocket connections
pub type WsConnections = Arc<TokioMutex<HashMap<String, ConnectionInfo>>>;
pub type WsPresence = Arc<TokioMutex<HashMap<Uuid, PresenceInfo>>>;
pub type WsIpCounters = Arc<TokioMutex<HashMap<String, usize>>>;

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub profile_id: Uuid,
    pub username: String,
    pub room: Option<String>,
    pub addr: Recipient<WsMessage>,
    pub ip: String,
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
    ip_counters: WsIpCounters,
    pub limits: WsLimits,
}

impl WsState {
    pub fn new() -> Self {
        Self::with_limits(WsLimits::default())
    }

    pub fn with_limits(limits: WsLimits) -> Self {
        Self {
            connections: Arc::new(TokioMutex::new(HashMap::new())),
            presence: Arc::new(TokioMutex::new(HashMap::new())),
            ip_counters: Arc::new(TokioMutex::new(HashMap::new())),
            limits,
        }
    }

    /// Check whether a new connection from `ip` is allowed under current limits.
    /// Returns `Ok(())` if allowed, or `Err(message)` if rejected.
    pub async fn check_connection_limits(&self, ip: &str) -> Result<(), &'static str> {
        let conns = self.connections.lock().await;
        let total = conns.len();
        drop(conns);

        if total >= self.limits.max_global_connections {
            return Err("Global WebSocket connection limit reached");
        }

        let counters = self.ip_counters.lock().await;
        let per_ip = counters.get(ip).copied().unwrap_or(0);
        drop(counters);

        if per_ip >= self.limits.max_connections_per_ip {
            return Err("Per-IP WebSocket connection limit reached");
        }

        Ok(())
    }

    pub async fn add_connection(&self, conn_id: String, info: ConnectionInfo) {
        let ip = info.ip.clone();

        let should_broadcast_online = {
            let mut conns = self.connections.lock().await;
            let is_new_connection = conns.insert(conn_id, info.clone()).is_none();
            drop(conns);

            if is_new_connection {
                let mut counters = self.ip_counters.lock().await;
                *counters.entry(ip).or_insert(0) += 1;
            }

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
        let removed = {
            let mut conns = self.connections.lock().await;
            conns.remove(conn_id)
        };

        let Some(info) = removed else {
            return;
        };

        {
            let mut counters = self.ip_counters.lock().await;
            if let Some(count) = counters.get_mut(&info.ip) {
                if *count > 1 {
                    *count -= 1;
                } else {
                    counters.remove(&info.ip);
                }
            }
        }

        let last_seen_at = {
            let mut presence = self.presence.lock().await;
            let Some(entry) = presence.get_mut(&info.profile_id) else {
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
            self.broadcast_presence_change(info.profile_id, false, Some(seen_at)).await;
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

    pub async fn total_connections(&self) -> usize {
        self.connections.lock().await.len()
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
    pub last_pong: chrono::DateTime<chrono::Utc>,
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let conn_info = ConnectionInfo {
            profile_id: self.profile_id,
            username: self.username.clone(),
            room: self.room.clone(),
            addr: ctx.address().recipient(),
            ip: "unknown".to_string(),
        };
        let ws_state = self.ws_state.clone();
        let conn_id = self.conn_id.clone();
        let fut = async move {
            ws_state.add_connection(conn_id, conn_info).await;
        };
        ctx.spawn(actix::fut::wrap_future(fut));

        // Start heartbeat
        let interval = self.ws_state.limits.heartbeat_interval_secs;
        ctx.run_interval(
            std::time::Duration::from_secs(interval),
            |act, ctx| {
                let timeout = act.ws_state.limits.client_timeout_secs;
                let elapsed = Utc::now()
                    .signed_duration_since(act.last_pong)
                    .num_seconds() as u64;

                if elapsed > timeout {
                    tracing::warn!(
                        connection_id = %act.conn_id,
                        "WebSocket heartbeat timeout (no pong for {}s)",
                        elapsed
                    );
                    ctx.close(None);
                    ctx.stop();
                    return;
                }

                ctx.ping(b"");
            },
        );

        tracing::info!(
            "WebSocket connected: {} (profile: {})",
            self.conn_id,
            self.profile_id
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let ws_state = self.ws_state.clone();
        let conn_id = self.conn_id.clone();
        let fut = async move {
            ws_state.remove_connection(&conn_id).await;
        };
        actix::spawn(fut);
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
            Ok(ws::Message::Pong(_)) => {
                self.last_pong = Utc::now();
            }
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
        use crate::ws::validation::{validate_client_message, WsClientAction};

        let action = match validate_client_message(text) {
            Ok(a) => a,
            Err(err) => {
                tracing::warn!(
                    connection_id = %self.conn_id,
                    error_code = %err.error.code,
                    "Invalid WebSocket message"
                );
                ctx.text(err.to_json());
                return;
            }
        };

        match action {
            WsClientAction::JoinRoom { room } => {
                self.room = Some(room.clone());
                let conn_info = ConnectionInfo {
                    profile_id: self.profile_id,
                    username: self.username.clone(),
                    room: self.room.clone(),
                    addr: ctx.address().recipient(),
                    ip: "unknown".to_string(),
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
            WsClientAction::LeaveRoom => {
                self.room = None;
                let conn_info = ConnectionInfo {
                    profile_id: self.profile_id,
                    username: self.username.clone(),
                    room: None,
                    addr: ctx.address().recipient(),
                    ip: "unknown".to_string(),
                };
                let ws_state = self.ws_state.clone();
                let conn_id = self.conn_id.clone();
                ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                    ws_state.add_connection(conn_id, conn_info).await;
                }));
            }
            WsClientAction::Chat { content } => {
                if let Some(room) = &self.room {
                    let chat_msg = WsMessage::chat(&content, &self.username);
                    let ws_state = self.ws_state.clone();
                    let room = room.to_string();
                    ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                        ws_state.broadcast_to_room(&room, chat_msg).await;
                    }));
                }
            }
            WsClientAction::Typing => {
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
            WsClientAction::StopTyping => {
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
            WsClientAction::Ping => {
                let pong = WsMessage::new(
                    "pong",
                    serde_json::json!({
                        "timestamp": chrono::Utc::now().timestamp(),
                    }),
                );
                ctx.text(serde_json::to_string(&pong).unwrap_or_default());
            }
        }
    }
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
            let secrets = state
                .as_ref()
                .map(|s| s.config.jwt_secrets.clone())
                .unwrap_or_default();

            match crate::middleware::auth::verify_token_with_secrets(
                &t,
                &secrets,
                crate::middleware::auth::WEBSOCKET_TOKEN_USE,
            ) {
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

    let ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Err(reason) = ws_state.check_connection_limits(&ip).await {
        tracing::warn!(
            connection_id = %conn_id,
            ip = %ip,
            reason = %reason,
            "WebSocket connection rejected"
        );
        return Err(AppError::Forbidden(reason.to_string()));
    }

    tracing::info!(
        "WebSocket connection: {} (profile: {}, ip: {})",
        conn_id,
        profile_id,
        ip
    );

    let ws_actor = WebSocketActor {
        conn_id,
        profile_id,
        username: profile_id.to_string(),
        room: None,
        ws_state,
        last_pong: Utc::now(),
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
                ip: "127.0.0.1".to_string(),
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

    #[cfg(test)]
    mod connection_limits_tests {
        use super::super::*;

        fn make_info(ip: &str) -> ConnectionInfo {
            ConnectionInfo {
                profile_id: Uuid::new_v4(),
                username: "test".to_string(),
                room: None,
                addr: {
                    struct Dummy;
                    impl actix::Actor for Dummy {
                        type Context = actix::Context<Self>;
                    }
                    impl actix::Handler<WsMessage> for Dummy {
                        type Result = ();
                        fn handle(&mut self, _: WsMessage, _: &mut Self::Context) {}
                    }
                    let addr: actix::Addr<Dummy> = Dummy.start();
                    addr.recipient()
                },
                ip: ip.to_string(),
            }
        }

        #[actix_web::test]
        async fn allows_connection_under_ip_limit() {
            let limits = WsLimits {
                max_connections_per_ip: 2,
                max_global_connections: 100,
                heartbeat_interval_secs: 30,
                client_timeout_secs: 60,
            };
            let state = WsState::with_limits(limits);
            assert!(state.check_connection_limits("10.0.0.1").await.is_ok());
        }

        #[actix_web::test]
        async fn rejects_connection_over_ip_limit() {
            let limits = WsLimits {
                max_connections_per_ip: 2,
                max_global_connections: 100,
                heartbeat_interval_secs: 30,
                client_timeout_secs: 60,
            };
            let state = WsState::with_limits(limits);

            let info1 = make_info("10.0.0.1");
            let info2 = make_info("10.0.0.1");
            state.add_connection("c1".to_string(), info1).await;
            state.add_connection("c2".to_string(), info2).await;

            assert!(state.check_connection_limits("10.0.0.1").await.is_err());
        }

        #[actix_web::test]
        async fn different_ips_have_separate_limits() {
            let limits = WsLimits {
                max_connections_per_ip: 1,
                max_global_connections: 100,
                heartbeat_interval_secs: 30,
                client_timeout_secs: 60,
            };
            let state = WsState::with_limits(limits);

            state.add_connection("c1".to_string(), make_info("10.0.0.1")).await;

            assert!(state.check_connection_limits("10.0.0.2").await.is_ok());
        }

        #[actix_web::test]
        async fn rejects_when_global_limit_reached() {
            let limits = WsLimits {
                max_connections_per_ip: 100,
                max_global_connections: 2,
                heartbeat_interval_secs: 30,
                client_timeout_secs: 60,
            };
            let state = WsState::with_limits(limits);

            state.add_connection("c1".to_string(), make_info("10.0.0.1")).await;
            state.add_connection("c2".to_string(), make_info("10.0.0.2")).await;

            assert!(state.check_connection_limits("10.0.0.3").await.is_err());
        }

        #[actix_web::test]
        async fn remove_decrements_ip_counter() {
            let limits = WsLimits {
                max_connections_per_ip: 2,
                max_global_connections: 100,
                heartbeat_interval_secs: 30,
                client_timeout_secs: 60,
            };
            let state = WsState::with_limits(limits);

            state.add_connection("c1".to_string(), make_info("10.0.0.1")).await;
            state.add_connection("c2".to_string(), make_info("10.0.0.1")).await;
            assert!(state.check_connection_limits("10.0.0.1").await.is_err());

            state.remove_connection("c1").await;
            assert!(state.check_connection_limits("10.0.0.1").await.is_ok());
        }

        #[actix_web::test]
        async fn remove_last_connection_cleans_ip_counter() {
            let limits = WsLimits {
                max_connections_per_ip: 10,
                max_global_connections: 100,
                heartbeat_interval_secs: 30,
                client_timeout_secs: 60,
            };
            let state = WsState::with_limits(limits);

            state.add_connection("c1".to_string(), make_info("10.0.0.1")).await;
            state.remove_connection("c1").await;

            let counters = state.ip_counters.lock().await;
            assert!(!counters.contains_key("10.0.0.1"));
        }

        #[actix_web::test]
        async fn total_connections_tracked() {
            let limits = WsLimits::default();
            let state = WsState::with_limits(limits);

            assert_eq!(state.total_connections().await, 0);
            state.add_connection("c1".to_string(), make_info("10.0.0.1")).await;
            state.add_connection("c2".to_string(), make_info("10.0.0.2")).await;
            assert_eq!(state.total_connections().await, 2);

            state.remove_connection("c1").await;
            assert_eq!(state.total_connections().await, 1);
        }

        #[actix_web::test]
        async fn default_limits_are_sane() {
            let limits = WsLimits::default();
            assert!(limits.max_connections_per_ip > 0);
            assert!(limits.max_global_connections > 0);
            assert!(limits.heartbeat_interval_secs > 0);
            assert!(limits.client_timeout_secs > limits.heartbeat_interval_secs);
        }
    }
}
