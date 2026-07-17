use crate::errors::{AppError, AppResult};
use crate::ws::server::{ConnectionInfo, PresenceInfo, WsLimits, WsMessage};
use chrono::{DateTime, Utc};
use deadpool_redis::Pool;
use futures_util::StreamExt;
use redis::AsyncCommands;
use redis::cmd;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use actix::Recipient;

const WS_CONN_PREFIX: &str = "ws:conn:";
const WS_CONN_INDEX: &str = "ws:conn:index";
const WS_USER_CONN_PREFIX: &str = "ws:user:";
const WS_ROOM_PREFIX: &str = "ws:room:";
const WS_IP_PREFIX: &str = "ws:ip:";
const WS_GLOBAL_COUNT: &str = "ws:global:count";
const WS_PRESENCE_PREFIX: &str = "ws:presence:";
const WS_PUBSUB_CHANNEL: &str = "ws:broadcast";

/// A locally-tracked WebSocket connection (this instance only).
struct LocalConn {
    addr: Recipient<WsMessage>,
    profile_id: Uuid,
    room: Option<String>,
}

/// Redis-backed WebSocket state for distributed deployments.
///
/// Architecture:
/// - Redis stores global state: connection metadata, presence, IP limits
/// - Each instance keeps a local `HashMap<conn_id, Recipient>` for message delivery
/// - Redis Pub/Sub broadcasts envelopes; each instance filters and delivers locally
#[derive(Clone)]
pub struct WsRedisState {
    pool: Pool,
    pub limits: WsLimits,
    local_connections: Arc<TokioMutex<HashMap<String, LocalConn>>>,
}

impl WsRedisState {
    pub fn new(pool: Pool, limits: WsLimits) -> Self {
        Self {
            pool,
            limits,
            local_connections: Arc::new(TokioMutex::new(HashMap::new())),
        }
    }

    pub fn with_default_limits(pool: Pool) -> Self {
        Self::new(pool, WsLimits::default())
    }

    // ------------------------------------------------------------------
    // Local connection registry
    // ------------------------------------------------------------------

    /// Register a local actor address so Pub/Sub messages can be delivered to it.
    pub async fn register_local(
        &self,
        conn_id: &str,
        addr: Recipient<WsMessage>,
        profile_id: Uuid,
        room: Option<String>,
    ) {
        let mut map = self.local_connections.lock().await;
        map.insert(
            conn_id.to_string(),
            LocalConn {
                addr,
                profile_id,
                room,
            },
        );
    }

    /// Unregister a local actor address.
    pub async fn unregister_local(&self, conn_id: &str) {
        let mut map = self.local_connections.lock().await;
        map.remove(conn_id);
    }

    /// Update the room for a local connection.
    pub async fn update_local_room(&self, conn_id: &str, room: Option<String>) {
        let mut map = self.local_connections.lock().await;
        if let Some(conn) = map.get_mut(conn_id) {
            conn.room = room;
        }
    }

    // ------------------------------------------------------------------
    // Connection limits (Redis-backed)
    // ------------------------------------------------------------------

    /// Check connection limits using Redis atomic operations
    pub async fn check_connection_limits(&self, ip: &str) -> AppResult<()> {
        let _span = tracing::debug_span!("ws.redis.check_limits", ip = %ip).entered();
        let mut conn = self.get_conn().await?;

        let global_count: usize = conn.get(WS_GLOBAL_COUNT).await.unwrap_or(0);
        if global_count >= self.limits.max_global_connections {
            return Err(AppError::Forbidden(
                "Global WebSocket connection limit reached".to_string(),
            ));
        }

        let ip_key = format!("{}{}", WS_IP_PREFIX, ip);
        let ip_count: usize = conn.get(&ip_key).await.unwrap_or(0);
        if ip_count >= self.limits.max_connections_per_ip {
            return Err(AppError::Forbidden(
                "Per-IP WebSocket connection limit reached".to_string(),
            ));
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Redis connection metadata (global state)
    // ------------------------------------------------------------------

    /// Store connection metadata in Redis and register locally.
    pub async fn add_connection(
        &self,
        conn_id: String,
        info: ConnectionInfo,
        addr: Recipient<WsMessage>,
    ) -> AppResult<bool> {
        let _span = tracing::info_span!("ws.redis.add_connection", conn_id = %conn_id, profile_id = %info.profile_id).entered();
        let ip = info.ip.clone();
        let profile_id = info.profile_id;
        let room = info.room.clone();

        let conn_key = format!("{}{}", WS_CONN_PREFIX, conn_id);
        let user_conn_key = format!("{}{}:conns", WS_USER_CONN_PREFIX, profile_id);
        let room_label = room.clone().unwrap_or_else(|| "default".to_string());
        let room_key = format!("{}{}:conns", WS_ROOM_PREFIX, room_label);
        let ip_key = format!("{}{}", WS_IP_PREFIX, ip);

        let conn_data = serde_json::to_string(&info).map_err(|e| {
            AppError::Internal(format!("Failed to serialize connection info: {}", e))
        })?;

        let mut pipe = redis::pipe();

        pipe.cmd("HSET")
            .arg(&conn_key)
            .arg("data")
            .arg(&conn_data)
            .cmd("HSET")
            .arg(&conn_key)
            .arg("profile_id")
            .arg(profile_id.to_string())
            .cmd("HSET")
            .arg(&conn_key)
            .arg("ip")
            .arg(&ip)
            .cmd("HSET")
            .arg(&conn_key)
            .arg("room")
            .arg(&room_label)
            .cmd("HSET")
            .arg(&conn_key)
            .arg("username")
            .arg(&info.username)
            .cmd("EXPIRE")
            .arg(&conn_key)
            .arg(86400);

        pipe.cmd("SADD")
            .arg(WS_CONN_INDEX)
            .arg(&conn_id)
            .cmd("SADD")
            .arg(&user_conn_key)
            .arg(&conn_id)
            .cmd("SADD")
            .arg(&room_key)
            .arg(&conn_id)
            .cmd("INCR")
            .arg(WS_GLOBAL_COUNT)
            .cmd("INCR")
            .arg(&ip_key)
            .cmd("EXPIRE")
            .arg(&user_conn_key)
            .arg(86400)
            .cmd("EXPIRE")
            .arg(&room_key)
            .arg(86400)
            .cmd("EXPIRE")
            .arg(&ip_key)
            .arg(86400);

        let mut conn = self.get_conn().await?;
        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Redis pipeline failed: {}", e)))?;

        // Register locally
        self.register_local(&conn_id, addr, profile_id, room).await;

        // Update presence
        let is_first = self.increment_user_connections(profile_id).await?;
        if is_first {
            self.publish_presence_change(profile_id, true, None).await?;
        }

        Ok(is_first)
    }

    /// Remove connection metadata from Redis and unregister locally.
    pub async fn remove_connection(&self, conn_id: &str) -> AppResult<()> {
        let _span = tracing::info_span!("ws.redis.remove_connection", conn_id = %conn_id).entered();
        // Unregister locally first
        self.unregister_local(conn_id).await;

        let mut conn = self.get_conn().await?;
        let conn_key = format!("{}{}", WS_CONN_PREFIX, conn_id);

        let info_json: Option<String> = conn.hget(&conn_key, "data").await.ok().flatten();
        if info_json.is_none() {
            return Ok(());
        }

        let info: ConnectionInfo = serde_json::from_str(&info_json.unwrap())
            .map_err(|e| AppError::Internal(format!("Failed to deserialize connection: {}", e)))?;

        let profile_id = info.profile_id;
        let ip = info.ip.clone();
        let room_label = info.room.clone().unwrap_or_else(|| "default".to_string());
        let user_conn_key = format!("{}{}:conns", WS_USER_CONN_PREFIX, profile_id);
        let room_key = format!("{}{}:conns", WS_ROOM_PREFIX, room_label);
        let ip_key = format!("{}{}", WS_IP_PREFIX, ip);

        let mut pipe = redis::pipe();
        pipe.cmd("DEL")
            .arg(&conn_key)
            .cmd("SREM")
            .arg(WS_CONN_INDEX)
            .arg(conn_id)
            .cmd("SREM")
            .arg(&user_conn_key)
            .arg(conn_id)
            .cmd("SREM")
            .arg(&room_key)
            .arg(conn_id)
            .cmd("DECR")
            .arg(WS_GLOBAL_COUNT)
            .cmd("DECR")
            .arg(&ip_key)
            .cmd("EXPIRE")
            .arg(&ip_key)
            .arg(3600);

        let mut conn = self.get_conn().await?;
        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Redis pipeline failed: {}", e)))?;

        let is_last = self.decrement_user_connections(profile_id).await?;
        if is_last {
            self.publish_presence_change(profile_id, false, Some(Utc::now()))
                .await?;
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Presence (Redis-backed)
    // ------------------------------------------------------------------

    async fn increment_user_connections(&self, profile_id: Uuid) -> AppResult<bool> {
        let _span =
            tracing::debug_span!("ws.redis.user_conn_incr", profile_id = %profile_id).entered();
        let mut conn = self.get_conn().await?;
        let presence_key = format!("{}{}", WS_PRESENCE_PREFIX, profile_id);

        let count: usize = conn
            .hincr(&presence_key, "active_connections", 1)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to increment connections: {}", e)))?;

        let _: bool = conn
            .expire::<&str, bool>(&presence_key, 86400)
            .await
            .unwrap_or(false);

        Ok(count == 1)
    }

    async fn decrement_user_connections(&self, profile_id: Uuid) -> AppResult<bool> {
        let _span =
            tracing::debug_span!("ws.redis.user_conn_decr", profile_id = %profile_id).entered();
        let mut conn = self.get_conn().await?;
        let presence_key = format!("{}{}", WS_PRESENCE_PREFIX, profile_id);

        let count: isize = conn
            .hincr(&presence_key, "active_connections", -1)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to decrement connections: {}", e)))?;

        if count <= 0 {
            conn.hset::<&str, &str, i32, usize>(&presence_key, "active_connections", 0)
                .await
                .ok();
            let _: bool = conn
                .expire::<&str, bool>(&presence_key, 3600)
                .await
                .unwrap_or(false);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_presence(&self, profile_id: Uuid) -> AppResult<PresenceInfo> {
        let mut conn = self.get_conn().await?;
        let presence_key = format!("{}{}", WS_PRESENCE_PREFIX, profile_id);

        let active_connections: usize = conn
            .hget(&presence_key, "active_connections")
            .await
            .unwrap_or(0);

        let last_seen_at: Option<DateTime<Utc>> = conn
            .hget(&presence_key, "last_seen_at")
            .await
            .ok()
            .flatten()
            .and_then(|s: String| s.parse().ok());

        Ok(PresenceInfo {
            active_connections,
            last_seen_at,
        })
    }

    pub async fn is_user_online(&self, profile_id: Uuid) -> AppResult<bool> {
        let presence = self.get_presence(profile_id).await?;
        Ok(presence.active_connections > 0)
    }

    // ------------------------------------------------------------------
    // Pub/Sub publishing (all instances receive)
    // ------------------------------------------------------------------

    async fn publish_presence_change(
        &self,
        profile_id: Uuid,
        is_online: bool,
        last_seen_at: Option<DateTime<Utc>>,
    ) -> AppResult<()> {
        let envelope = serde_json::json!({
            "type": "presence_changed",
            "profile_id": profile_id.to_string(),
            "is_online": is_online,
            "last_seen_at": last_seen_at.map(|dt| dt.to_rfc3339()),
        });

        self.publish_envelope(&envelope).await
    }

    pub async fn broadcast_to_all(&self, message: &WsMessage) -> AppResult<()> {
        let envelope = serde_json::json!({
            "type": "broadcast",
            "payload": message,
        });
        self.publish_envelope(&envelope).await
    }

    pub async fn send_to_user(&self, profile_id: Uuid, message: &WsMessage) -> AppResult<()> {
        let envelope = serde_json::json!({
            "type": "direct",
            "target_user": profile_id.to_string(),
            "payload": message,
        });
        self.publish_envelope(&envelope).await
    }

    pub async fn broadcast_to_room(&self, room: &str, message: &WsMessage) -> AppResult<()> {
        let envelope = serde_json::json!({
            "type": "room",
            "target_room": room,
            "payload": message,
        });
        self.publish_envelope(&envelope).await
    }

    pub async fn broadcast_to_room_except(
        &self,
        room: &str,
        excluded_profile_id: Uuid,
        message: &WsMessage,
    ) -> AppResult<()> {
        let envelope = serde_json::json!({
            "type": "room_except",
            "target_room": room,
            "excluded_user": excluded_profile_id.to_string(),
            "payload": message,
        });
        self.publish_envelope(&envelope).await
    }

    async fn publish_envelope(&self, envelope: &serde_json::Value) -> AppResult<()> {
        let _span = tracing::debug_span!("ws.redis.publish", channel = WS_PUBSUB_CHANNEL).entered();
        let payload = serde_json::to_string(envelope).map_err(|e| {
            AppError::Internal(format!("Failed to serialize Pub/Sub envelope: {}", e))
        })?;

        let mut conn = self.get_conn().await?;
        let _: usize = conn
            .publish(WS_PUBSUB_CHANNEL, &payload)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to publish to Pub/Sub: {}", e)))?;

        Ok(())
    }

    // ------------------------------------------------------------------
    // Pub/Sub receiving (this instance only)
    // ------------------------------------------------------------------

    /// Handle incoming Pub/Sub message: parse envelope and deliver to local actors.
    pub async fn handle_pubsub_message(&self, payload: &str) -> AppResult<()> {
        let _span = tracing::debug_span!("ws.handle_pubsub").entered();
        let envelope: serde_json::Value = serde_json::from_str(payload)
            .map_err(|e| AppError::Internal(format!("Failed to parse Pub/Sub envelope: {}", e)))?;

        let msg_type = envelope.get("type").and_then(|v| v.as_str()).unwrap_or("");

        let payload_val = match envelope.get("payload") {
            Some(p) => p,
            None => return Ok(()),
        };

        let ws_message: WsMessage = serde_json::from_value(payload_val.clone())
            .map_err(|e| AppError::Internal(format!("Failed to deserialize WsMessage: {}", e)))?;

        match msg_type {
            "presence_changed" => {
                debug!("Received presence change from another instance");
            },
            "broadcast" => {
                let conns = self.local_connections.lock().await;
                for local in conns.values() {
                    let _ = local.addr.try_send(ws_message.clone());
                }
            },
            "direct" => {
                let target_user = envelope
                    .get("target_user")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());
                if let Some(target) = target_user {
                    let conns = self.local_connections.lock().await;
                    for local in conns.values() {
                        if local.profile_id == target {
                            let _ = local.addr.try_send(ws_message.clone());
                        }
                    }
                }
            },
            "room" => {
                let target_room = envelope.get("target_room").and_then(|v| v.as_str());
                if let Some(room) = target_room {
                    let conns = self.local_connections.lock().await;
                    for local in conns.values() {
                        if local.room.as_deref() == Some(room) {
                            let _ = local.addr.try_send(ws_message.clone());
                        }
                    }
                }
            },
            "room_except" => {
                let target_room = envelope.get("target_room").and_then(|v| v.as_str());
                let excluded_user = envelope
                    .get("excluded_user")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());
                if let (Some(room), Some(excluded)) = (target_room, excluded_user) {
                    let conns = self.local_connections.lock().await;
                    for local in conns.values() {
                        if local.room.as_deref() == Some(room) && local.profile_id != excluded {
                            let _ = local.addr.try_send(ws_message.clone());
                        }
                    }
                }
            },
            _ => {
                debug!("Received unknown Pub/Sub message type: {}", msg_type);
            },
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    pub async fn total_connections(&self) -> AppResult<usize> {
        let mut conn = self.get_conn().await?;
        let count: usize = conn.get(WS_GLOBAL_COUNT).await.unwrap_or(0);
        Ok(count)
    }

    pub async fn update_last_seen(&self, profile_id: Uuid) -> AppResult<()> {
        let mut conn = self.get_conn().await?;
        let presence_key = format!("{}{}", WS_PRESENCE_PREFIX, profile_id);
        let now = Utc::now().to_rfc3339();

        cmd("HSET")
            .arg(&presence_key)
            .arg("last_seen_at")
            .arg(&now)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update last seen: {}", e)))?;

        let _: bool = conn
            .expire::<&str, bool>(&presence_key, 86400)
            .await
            .unwrap_or(false);

        Ok(())
    }

    // ------------------------------------------------------------------
    // Internal
    // ------------------------------------------------------------------

    async fn get_conn(&self) -> AppResult<deadpool_redis::Connection> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get Redis connection: {}", e)))
    }
}

/// Background task: subscribe to Redis Pub/Sub and deliver to local actors.
pub async fn run_pubsub_listener(state: Arc<WsRedisState>) -> AppResult<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url)
        .map_err(|e| AppError::Internal(format!("Failed to create Redis client: {}", e)))?;

    let mut pubsub = client
        .get_async_pubsub()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create PubSub: {}", e)))?;

    pubsub
        .subscribe(WS_PUBSUB_CHANNEL)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to subscribe to channel: {}", e)))?;

    info!(
        "WebSocket Pub/Sub listener started on channel {}",
        WS_PUBSUB_CHANNEL
    );

    let mut on_message = pubsub.on_message();
    while let Some(msg) = on_message.next().await {
        if let Ok(payload) = msg.get_payload::<String>() {
            let _span = tracing::debug_span!("ws.redis.pubsub.message").entered();
            if let Err(e) = state.handle_pubsub_message(&payload).await {
                error!("Error handling Pub/Sub message: {}", e);
            }
        }
    }

    warn!("Pub/Sub channel closed, restarting listener");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::Actor;
    use uuid::Uuid;

    #[test]
    fn test_redis_key_prefixes() {
        assert_eq!(WS_CONN_PREFIX, "ws:conn:");
        assert_eq!(WS_PUBSUB_CHANNEL, "ws:broadcast");
    }

    #[actix_web::test]
    async fn test_connection_info_serialization_skips_addr() {
        let info = ConnectionInfo {
            profile_id: Uuid::new_v4(),
            username: "test_user".to_string(),
            room: Some("room1".to_string()),
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
            ip: "127.0.0.1".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test_user"));
        assert!(!json.contains("addr"));

        let deserialized: ConnectionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.username, deserialized.username);
        assert_eq!(info.profile_id, deserialized.profile_id);
        assert_eq!(info.ip, deserialized.ip);
    }

    #[actix_web::test]
    async fn test_register_and_unregister_local() {
        struct Dummy;

        impl actix::Actor for Dummy {
            type Context = actix::Context<Self>;
        }

        impl actix::Handler<WsMessage> for Dummy {
            type Result = ();
            fn handle(&mut self, _: WsMessage, _: &mut actix::Context<Self>) {}
        }

        let pool = deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .unwrap();
        let state = WsRedisState::with_default_limits(pool);
        let conn_id = "test-conn-1".to_string();
        let profile_id = Uuid::new_v4();

        let addr: actix::Addr<Dummy> = Dummy.start();
        let recipient = addr.recipient();

        state
            .register_local(&conn_id, recipient, profile_id, Some("room1".into()))
            .await;

        {
            let map = state.local_connections.lock().await;
            assert!(map.contains_key(&conn_id));
            assert_eq!(map[&conn_id].profile_id, profile_id);
        }

        state.unregister_local(&conn_id).await;

        {
            let map = state.local_connections.lock().await;
            assert!(!map.contains_key(&conn_id));
        }
    }
}
