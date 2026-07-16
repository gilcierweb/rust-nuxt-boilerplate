use actix_web::{get, web, HttpResponse};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Serialize)]
struct JwkKey {
    kty: &'static str,
    #[serde(rename = "use")]
    use_: &'static str,
    kid: String,
    alg: &'static str,
    k: String,
}

#[get("/.well-known/jwks.json")]
pub async fn jwks(state: web::Data<AppState>) -> HttpResponse {
    let keys: Vec<JwkKey> = state
        .config
        .jwt_secrets
        .iter()
        .filter(|k| k.is_active())
        .map(|k| JwkKey {
            kty: "oct",
            use_: "sig",
            kid: k.kid.clone(),
            alg: "HS256",
            k: base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                k.secret.as_bytes(),
            ),
        })
        .collect();

    HttpResponse::Ok().json(JwksResponse { keys })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use deadpool::managed::Pool;
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
    use diesel_async::AsyncPgConnection;
    use std::sync::Arc;

    fn make_test_state() -> web::Data<AppState> {
        let config = crate::security::test_utils::test_config();
        let redis_cfg = deadpool_redis::Config::from_url("redis://127.0.0.1:6379");
        let redis = redis_cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .unwrap();

        let manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new("postgres://localhost/test");
        let db: crate::db::database::DBPool = Pool::builder(manager).build().unwrap();

        web::Data::new(AppState {
            db,
            redis,
            config: Arc::new(config),
            metrics: Arc::new(crate::services::metrics_service::MetricsRegistry::new()),
            ws: crate::ws::WsState::new(),
        })
    }

    #[actix_web::test]
    async fn jwks_returns_keys_from_config() {
        let state = make_test_state();
        let app = test::init_service(
            App::new()
                .app_data(state)
                .service(jwks),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/.well-known/jwks.json")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn jwks_returns_jwk_format() {
        let state = make_test_state();
        let app = test::init_service(
            App::new()
                .app_data(state)
                .service(jwks),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/.well-known/jwks.json")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let keys = body["keys"].as_array().unwrap();
        assert!(!keys.is_empty());
        assert_eq!(keys[0]["kty"], "oct");
        assert_eq!(keys[0]["alg"], "HS256");
        assert_eq!(keys[0]["use"], "sig");
        assert!(keys[0]["kid"].is_string());
    }
}
