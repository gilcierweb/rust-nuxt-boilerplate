//! Integration test of the `/auth/me` enrichment.
//!
//! The endpoint is the source of capabilities the frontend reads: the role
//! names, the effective class level permissions and the presentation role, all
//! derived from the grants stored in the database rather than from the token.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The canonical catalog is seeded from the lib service; the subjects carry the
//! shared test keys so the email can be decrypted.

use std::sync::Arc;
use std::sync::Mutex;

use actix_web::body::to_bytes;
use actix_web::http::StatusCode;
use actix_web::{App, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel::Connection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde_json::Value;
use uuid::Uuid;

use backend::config::test_config::test_config;
use backend::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
use backend::middleware::csrf_protection::CsrfProtection;
use backend::repositories::AppContainer;
use backend::security::SecurityService;
use backend::services::rbac_catalog_service;

static CATALOG_SEED_MUTEX: Mutex<()> = Mutex::new(());

struct TestDb {
    pool: Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl TestDb {
    async fn new() -> Self {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url());
        let pool = Pool::builder(manager)
            .max_size(4)
            .build()
            .expect("Failed to create pool");
        Self { pool }
    }

    fn pool(&self) -> &Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
        &self.pool
    }
}

fn database_url() -> String {
    std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string())
}

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

fn seed_catalog() {
    let _guard = CATALOG_SEED_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut conn = diesel::PgConnection::establish(&database_url())
        .expect("Failed to connect for the catalog seed");
    rbac_catalog_service::ensure_canonical_catalog(&mut conn).expect("catalog seed run failed");
}

/// A subject holding the given canonical role.
async fn subject_with_roles(db: &TestDb, roles: &[&str]) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let config = test_config();
    let security =
        SecurityService::from_config(&config).expect("failed to build the security service");
    let email = format!("{}@example.com", Uuid::new_v4());
    let protected_email = security
        .protect_email(&email)
        .expect("email protection failed");

    let user = diesel::sql_query(
        "INSERT INTO users (email_blind_index, email_encrypted, encryption_key_version) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind::<diesel::sql_types::Bytea, _>(&protected_email.blind_index)
    .bind::<diesel::sql_types::Bytea, _>(&protected_email.encrypted)
    .bind::<diesel::sql_types::Integer, _>(protected_email.key_version)
    .get_result::<IdRow>(&mut *conn)
    .await
    .expect("user insert failed");

    for role_name in roles {
        diesel::sql_query(
            "INSERT INTO users_roles (user_id, role_id) \
             SELECT $1, id FROM roles WHERE name = $2 AND resource_type IS NULL",
        )
        .bind::<diesel::sql_types::Uuid, _>(user.id)
        .bind::<diesel::sql_types::Text, _>(*role_name)
        .execute(&mut *conn)
        .await
        .expect("user role insert failed");
    }

    user.id
}

/// The production auth stack, backed by the test database.
macro_rules! auth_app {
    ($db:expr) => {{
        let config = Arc::new(test_config());
        let redis = redis_pool();
        let container = web::Data::new(AppContainer::new(
            $db.pool().clone(),
            redis.clone(),
            (*config).clone(),
        ));

        test::init_service(
            App::new()
                .app_data(container)
                .app_data(web::Data::new(backend::ws::WsRedisState::new(
                    redis,
                    backend::ws::WsLimits::default(),
                )))
                .service(
                    web::scope("/api/v1/auth")
                        .wrap(CsrfProtection::new(vec![]))
                        .wrap(JwtAuth::new(JwtAuthConfig::bearer_exempt()))
                        .configure(backend::controllers::auth_controller::config),
                ),
        )
        .await
    }};
}

async fn body_json(response: actix_web::dev::ServiceResponse) -> Value {
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    serde_json::from_slice(&bytes).expect("valid json body")
}

#[actix_web::test]
async fn admin_me_reports_the_full_capability_set() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = auth_app!(&db);

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&test_config(), subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert_eq!(json["id"], subject.to_string());
    assert!(
        json["email"]
            .as_str()
            .is_some_and(|email| email.ends_with("@example.com")),
        "the email is decrypted"
    );
    assert_eq!(json["role"], 1, "the admin holds the administrator role");
    let roles = json["roles"].as_array().expect("the roles array");
    assert_eq!(
        roles
            .iter()
            .map(|r| r.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["admin"]
    );
    let permissions = json["permissions"]
        .as_array()
        .expect("the permissions array");
    let codes: Vec<&str> = permissions.iter().map(|p| p.as_str().unwrap()).collect();
    assert!(
        codes.contains(&"users:delete"),
        "the full catalog is granted"
    );
    assert!(codes.contains(&"permissions:read"));
    assert_eq!(permissions.len(), 24, "every catalog code is class level");
}

#[actix_web::test]
async fn viewer_me_reports_the_read_only_capability_set() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["viewer"]).await;
    let app = auth_app!(&db);

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&test_config(), subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert_eq!(json["role"], 3, "a read only subject is the viewer role");
    let codes: Vec<&str> = json["permissions"]
        .as_array()
        .expect("the permissions array")
        .iter()
        .map(|code| code.as_str().unwrap())
        .collect();
    assert!(codes.contains(&"users:read"), "the viewer reads users");
    assert!(
        !codes.contains(&"roles:read"),
        "a class level permission the subject does not hold is absent"
    );
}

#[actix_web::test]
async fn customer_me_reports_only_the_self_scoped_permissions() {
    seed_catalog();
    let db = TestDb::new().await;
    // The customer is self scoped: its grants grant nothing at the class level,
    // so the class level list is empty and the role stays the viewer one.
    let subject = subject_with_roles(&db, &["customer"]).await;
    let app = auth_app!(&db);

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&test_config(), subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    assert_eq!(json["role"], 3);
    assert!(
        json["permissions"].as_array().is_some_and(|p| p.is_empty()),
        "a scoped grant never grants class level authority"
    );
}

/// A bearer token for the subject.
fn bearer(config: &backend::config::AppConfig, subject: Uuid) -> String {
    create_token(
        subject,
        Uuid::new_v4(),
        1,
        &config.jwt_secret,
        3600,
        &config.jwt_audience,
    )
    .expect("failed to mint the test token")
}
