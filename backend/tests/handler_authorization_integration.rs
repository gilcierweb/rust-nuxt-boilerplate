//! Integration tests of the handler authorization over the grants driven
//! ability.
//!
//! The stack is the production one: the `JwtAuth` middleware loads the grants
//! of the subject from the database, builds the ability from them and
//! attaches it to the request; the handlers authorize against that ability.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Redis running at REDIS_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::{App, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel::Connection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use backend::config::test_config::test_config;
use backend::controllers::{
    audit_logs_controller, roles_controller, upload_controller, users_controller,
};
use backend::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
use backend::repositories::AppContainer;
use backend::services::rbac_catalog_service;

/// Serializes the shared catalog seed across the parallel tests.
static CATALOG_SEED_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct TestDb {
    pool: Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl TestDb {
    async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
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

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

/// Seeds the canonical catalog (idempotent, serialized against other tests).
fn seed_catalog() {
    let _guard = CATALOG_SEED_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let database_url = std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());
    let mut conn = diesel::PgConnection::establish(&database_url)
        .expect("Failed to connect for the catalog seed");
    rbac_catalog_service::ensure_canonical_catalog(&mut conn).expect("catalog seed run failed");
}

/// Creates a subject and assigns the given canonical roles to it.
async fn subject_with_roles(db: &TestDb, roles: &[&str]) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    // The email is protected with the real security service: the admin list
    // decrypts it, so a fake payload would fail the read.
    let config = test_config();
    let security = backend::security::SecurityService::from_config(&config)
        .expect("failed to build the security service");
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

/// Creates a dedicated role holding a deny link, so the deny wins over the
/// allow links of the other roles of the subject without touching the
/// canonical roles shared with the other tests.
async fn role_with_deny_link(db: &TestDb, code: &str) -> String {
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let role_name = format!("deny-probe-{}", Uuid::new_v4());

    diesel::sql_query(
        "INSERT INTO roles (id, name, resource_type, resource_id, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, NULL, NULL, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Text, _>(&role_name)
    .execute(&mut *conn)
    .await
    .expect("role insert failed");

    diesel::sql_query(
        "INSERT INTO roles_permissions (role_id, permission_id, effect, created_at) \
         SELECT r.id, p.id, 1, NOW() FROM roles r JOIN permissions p ON p.code = $2 \
         WHERE r.name = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&role_name)
    .bind::<diesel::sql_types::Text, _>(code)
    .execute(&mut *conn)
    .await
    .expect("deny link insert failed");

    role_name
}

/// The production admin stack, backed by the test database.
macro_rules! admin_app {
    ($db:expr) => {{
        let config = Arc::new(test_config());
        let redis = redis_pool();

        let container = web::Data::new(AppContainer::new(
            $db.pool().clone(),
            redis,
            (*config).clone(),
        ));

        test::init_service(
            App::new().app_data(container).service(
                web::scope("/admin")
                    .wrap(JwtAuth::new(JwtAuthConfig::new(vec![])))
                    .configure(users_controller::config)
                    .configure(roles_controller::config)
                    .configure(audit_logs_controller::config)
                    .configure(upload_controller::config),
            ),
        )
        .await
    }};
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

#[actix_web::test]
async fn admin_with_read_grants_lists_users() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = test::TestRequest::get()
        .uri("/admin/users")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn a_deny_link_refuses_the_create_probe() {
    seed_catalog();
    let db = TestDb::new().await;
    let deny_role = role_with_deny_link(&db, "roles:create").await;
    let subject = subject_with_roles(&db, &["admin", &deny_role]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = test::TestRequest::post()
        .uri("/admin/roles")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .set_json(serde_json::json!({
            "name": "new-role",
            "resource_type": null,
            "resource_id": null
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn viewer_lists_users_but_cannot_delete_roles() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["viewer"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = test::TestRequest::get()
        .uri("/admin/users")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::delete()
        .uri(format!("/admin/roles/{}", Uuid::new_v4()).as_str())
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn manager_cannot_delete_roles() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["manager"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = test::TestRequest::delete()
        .uri(format!("/admin/roles/{}", Uuid::new_v4()).as_str())
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn upload_without_the_create_grant_is_refused() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["viewer"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = test::TestRequest::post()
        .uri("/admin/upload")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn audit_logs_without_the_read_grant_are_refused() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["viewer"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = test::TestRequest::get()
        .uri("/admin/audit-logs")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", bearer(&config, subject)),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
