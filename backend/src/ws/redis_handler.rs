#![allow(dead_code)]

use actix::prelude::*;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppState;
use crate::errors::{AppError, AppResult};
use crate::ws::redis_state::WsRedisState;
use crate::ws::server::{ConnectionInfo, WsMessage};

const WS_PROTOCOL: &str = "boilerplate-ws";
const WS_AUTH_PROTOCOL_PREFIX: &str = "auth.";

/// WebSocket actor for handling connections with Redis-backed state
pub struct WebSocketActor {
    pub conn_id: String,
    pub profile_id: Uuid,
    pub username: String,
    pub room: Option<String>,
    pub ws_state: web::Data<WsRedisState>,
    pub last_pong: OffsetDateTime,
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
        let addr = ctx.address().recipient();
        let fut = async move {
            let _ = ws_state.add_connection(conn_id, conn_info, addr).await;
        };
        ctx.spawn(actix::fut::wrap_future(fut));

        // Start heartbeat
        let interval = self.ws_state.limits.heartbeat_interval_secs;
        ctx.run_interval(std::time::Duration::from_secs(interval), |act, ctx| {
            let timeout = act.ws_state.limits.client_timeout_secs;
            let elapsed = (OffsetDateTime::now_utc() - act.last_pong).whole_seconds() as u64;

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
        });

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
            let _ = ws_state.remove_connection(&conn_id).await;
        };
        actix::spawn(fut);
        tracing::info!("WebSocket disconnected: {}", self.conn_id);
    }
}

/// Handle messages from other actors (e.g., Pub/Sub delivery)
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
                self.last_pong = OffsetDateTime::now_utc();
            },
            Ok(ws::Message::Text(text)) => self.handle_message(&text, ctx),
            Ok(ws::Message::Binary(_)) => {},
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            },
            _ => {},
        }
    }
}

impl WebSocketActor {
    fn handle_message(&mut self, text: &str, ctx: &mut ws::WebsocketContext<Self>) {
        use crate::ws::validation::validate_client_message;

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
            },
        };

        match action {
            crate::ws::validation::WsClientAction::JoinRoom { room } => {
                // The broadcast reaches every member of a room, so the room
                // is an access-controlled resource: a subject joins the room
                // that names it or a public project room, never the room of
                // another subject.
                if !crate::ws::validation::room_is_joinable(&room, &self.profile_id) {
                    let refusal = crate::ws::validation::WsValidationError::new(
                        "ROOM_FORBIDDEN",
                        t!("ws.room_forbidden").into_owned(),
                    );
                    ctx.text(refusal.to_json());
                    return;
                }

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
                let addr = ctx.address().recipient();
                ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                    let _ = ws_state.add_connection(conn_id, conn_info, addr).await;
                }));

                // Update local room tracking
                let ws_state = self.ws_state.clone();
                let conn_id = self.conn_id.clone();
                let room_clone = self.room.clone();
                ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                    ws_state.update_local_room(&conn_id, room_clone).await;
                }));

                let response = WsMessage::new(
                    "joined_room",
                    serde_json::json!({
                        "room": room,
                        "user": self.username,
                    }),
                );
                ctx.text(serde_json::to_string(&response).unwrap_or_default());
            },
            crate::ws::validation::WsClientAction::LeaveRoom => {
                self.room = None;
                let ws_state = self.ws_state.clone();
                let conn_id = self.conn_id.clone();
                ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                    ws_state.update_local_room(&conn_id, None).await;
                }));
            },
            crate::ws::validation::WsClientAction::Chat { content } => {
                if let Some(room) = &self.room {
                    let chat_msg = WsMessage::chat(&content, &self.username);
                    let ws_state = self.ws_state.clone();
                    let room = room.to_string();
                    ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
                        let _ = ws_state.broadcast_to_room(&room, &chat_msg).await;
                    }));
                }
            },
            crate::ws::validation::WsClientAction::Typing => {
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
                        let _ = ws_state
                            .broadcast_to_room_except(&room, profile_id, &typing_msg)
                            .await;
                    }));
                }
            },
            crate::ws::validation::WsClientAction::StopTyping => {
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
                        let _ = ws_state
                            .broadcast_to_room_except(&room, profile_id, &stop_typing_msg)
                            .await;
                    }));
                }
            },
            crate::ws::validation::WsClientAction::Ping => {
                let pong = WsMessage::new(
                    "pong",
                    serde_json::json!({
                        "timestamp": OffsetDateTime::now_utc().unix_timestamp(),
                    }),
                );
                ctx.text(serde_json::to_string(&pong).unwrap_or_default());
            },
        }
    }
}

/// WebSocket upgrade handler with Redis-backed state
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    ws_state: web::Data<crate::ws::redis_state::WsRedisState>,
) -> AppResult<HttpResponse> {
    let conn_id = Uuid::new_v4().to_string();

    let token = crate::ws::server::extract_ws_protocol_token(&req);

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
                &state
                    .as_ref()
                    .map(|s| s.config.jwt_audience.clone())
                    .unwrap_or_else(|| crate::middleware::auth::DEFAULT_TOKEN_AUDIENCE.to_string()),
            ) {
                Ok(result) => {
                    if let Some(m) = state.as_ref().map(|s| s.metrics.clone()) {
                        match result.outcome {
                            crate::middleware::auth::JwtVerifyOutcome::DirectMatch => {
                                m.record_jwt_direct_match()
                            },
                            crate::middleware::auth::JwtVerifyOutcome::FallbackMatch => {
                                m.record_jwt_fallback_match()
                            },
                            crate::middleware::auth::JwtVerifyOutcome::Rejected => {
                                m.record_jwt_rejected()
                            },
                        }
                    }

                    // A valid signature is not a live session: the handshake
                    // runs the same revocation rule the HTTP path enforces, so
                    // a token revoked by logout, or one predating a password
                    // change, cannot keep a socket open until it expires.
                    let answers = match state.as_ref() {
                        Some(app_state) => {
                            let blacklist = crate::repositories::access_token_blacklist::AccessTokenBlacklist::new(
                                app_state.redis.clone(),
                            );
                            crate::middleware::token_revocation::RevocationAnswers {
                                blacklisted: blacklist
                                    .is_blacklisted(
                                        &crate::repositories::access_token_blacklist::hash_token_for_blacklist(
                                            &t,
                                        ),
                                    )
                                    .await,
                                epoch: blacklist.credential_epoch(&result.claims.sub).await,
                            }
                        },
                        None => crate::middleware::token_revocation::RevocationAnswers::unavailable(),
                    };

                    match crate::middleware::token_revocation::decide_revocation(
                        &answers,
                        result.claims.iat,
                    ) {
                        crate::middleware::token_revocation::RevocationOutcome::Valid => {
                            result.claims.profile_id
                        },
                        crate::middleware::token_revocation::RevocationOutcome::Revoked => {
                            tracing::info!(
                                event = "auth.ws_token_revoked",
                                user_id = %result.claims.sub,
                                "WebSocket token revoked; refusing the handshake"
                            );
                            return Err(AppError::Unauthorized(
                                t!("middleware.token_revoked").into_owned(),
                            ));
                        },
                        crate::middleware::token_revocation::RevocationOutcome::Indeterminate => {
                            if let Some((event, error)) = answers.indeterminacy_cause() {
                                tracing::error!(
                                    event,
                                    error = %error,
                                    "WebSocket revocation status unavailable (Redis); failing closed"
                                );
                            }
                            return Err(AppError::ServiceUnavailable(
                                t!("middleware.token_status_unavailable").into_owned(),
                            ));
                        },
                    }
                },
                Err(_) => {
                    tracing::warn!("WebSocket auth failed");
                    return Err(AppError::Unauthorized(
                        t!("middleware.invalid_token").into_owned(),
                    ));
                },
            }
        },
        None => {
            return Err(AppError::Unauthorized(
                t!("auth.missing_token").into_owned(),
            ));
        },
    };

    let ip = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Err(e) = ws_state.check_connection_limits(&ip).await {
        tracing::warn!(
            connection_id = %conn_id,
            ip = %ip,
            error = %e,
            "WebSocket connection rejected"
        );
        return Err(e);
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
        last_pong: OffsetDateTime::now_utc(),
    };

    ws::WsResponseBuilder::new(ws_actor, &req, stream)
        .protocols(&[WS_PROTOCOL])
        .start()
        .map_err(|e| AppError::Internal(format!("WebSocket error: {}", e)))
}
