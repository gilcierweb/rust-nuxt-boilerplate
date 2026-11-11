//! Integration tests of the registration flow.
//!
//! The user, its profile and its default role are created in one
//! transaction: a registration either lands complete or not at all.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Redis running at REDIS_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`

use actix_web::http::StatusCode;
use actix_web::{App, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel::Connection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use backend::config::AppConfig;
use backend::config::test_config::test_config;
use backend::controllers::auth_controller;
use backend::repositories::AppContainer;
use backend::security::SecurityService;
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

/// The auth stack, backed by the test database.
macro_rules! auth_app {
    ($db:expr, $config:expr) => {{
        let container =
            web::Data::new(AppContainer::new($db.pool().clone(), redis_pool(), $config));

        test::init_service(
            App::new()
                .app_data(container)
                .service(web::scope("/auth").configure(auth_controller::config)),
        )
        .await
    }};
}

#[derive(diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct RoleIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    role_id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

/// The blind index of the address, the way the registration stores it.
fn blind_index(email: &str) -> Vec<u8> {
    let config = test_config();
    let security =
        SecurityService::from_config(&config).expect("failed to build the security service");
    security
        .protect_email(email)
        .expect("email protection failed")
        .blind_index
}

/// The number of user rows holding the address.
async fn users_with_email(db: &TestDb, email: &str) -> i64 {
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let index = blind_index(email);

    diesel::sql_query("SELECT COUNT(*) AS count FROM users WHERE email_blind_index = $1")
        .bind::<diesel::sql_types::Bytea, _>(&index)
        .get_result::<CountRow>(&mut *conn)
        .await
        .expect("user count failed")
        .count
}

/// The id of the user holding the address.
async fn user_id_with_email(db: &TestDb, email: &str) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let index = blind_index(email);

    diesel::sql_query("SELECT id FROM users WHERE email_blind_index = $1")
        .bind::<diesel::sql_types::Bytea, _>(&index)
        .get_result::<IdRow>(&mut *conn)
        .await
        .expect("user lookup failed")
        .id
}

/// The role ids linked to the user.
async fn role_ids_of(db: &TestDb, user_id: Uuid) -> Vec<Uuid> {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    diesel::sql_query("SELECT role_id FROM users_roles WHERE user_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<RoleIdRow>(&mut *conn)
        .await
        .expect("role link lookup failed")
        .into_iter()
        .map(|row| row.role_id)
        .collect()
}

/// The id of the canonical customer role.
async fn customer_role_id(db: &TestDb) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    diesel::sql_query("SELECT id FROM roles WHERE name = 'customer' AND resource_id IS NULL")
        .get_result::<IdRow>(&mut *conn)
        .await
        .expect("customer role lookup failed")
        .id
}

/// The number of profile rows of the user.
async fn profile_count(db: &TestDb, user_id: Uuid) -> i64 {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    diesel::sql_query("SELECT COUNT(*) AS count FROM profiles WHERE user_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<CountRow>(&mut *conn)
        .await
        .expect("profile count failed")
        .count
}

/// Registers an account with the given configuration.
macro_rules! register {
    ($app:expr, $email:expr) => {{
        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(serde_json::json!({
                "email": $email,
                "password": "CorrectPassword1!",
                "password_confirmation": "CorrectPassword1!"
            }))
            .to_request();

        test::call_service(&$app, req).await
    }};
}

#[actix_web::test]
async fn register_assigns_the_default_role_to_the_new_user() {
    seed_catalog();
    let db = TestDb::new().await;
    let email = format!("register-{}@example.com", Uuid::new_v4());
    let config: AppConfig = test_config();
    let app = auth_app!(&db, config);

    let resp = register!(app, &email);
    assert_eq!(resp.status(), StatusCode::CREATED);

    let user_id = user_id_with_email(&db, &email).await;
    assert_eq!(
        profile_count(&db, user_id).await,
        1,
        "the registration creates the profile of the user"
    );

    let role_ids = role_ids_of(&db, user_id).await;
    assert_eq!(
        role_ids.len(),
        1,
        "the registration assigns exactly one role, got {role_ids:?}"
    );
    assert_eq!(
        role_ids[0],
        customer_role_id(&db).await,
        "the registration assigns the default role"
    );
}

#[actix_web::test]
async fn register_fails_closed_when_default_role_missing() {
    seed_catalog();
    let db = TestDb::new().await;
    let email = format!("register-missing-{}@example.com", Uuid::new_v4());
    let mut config: AppConfig = test_config();
    config.register_default_role = "ghost".to_string();
    let app = auth_app!(&db, config);

    let resp = register!(app, &email);
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    assert_eq!(
        users_with_email(&db, &email).await,
        0,
        "a registration without a resolvable role creates no user"
    );
}

#[actix_web::test]
async fn register_fails_closed_when_default_role_ambiguous() {
    seed_catalog();
    let db = TestDb::new().await;
    let email = format!("register-ambiguous-{}@example.com", Uuid::new_v4());
    let role_name = format!("amb-{}", Uuid::new_v4());

    // Two roles carry the name and neither of them is bound to a subject:
    // the name is ambiguous and the registration must fail closed.
    {
        let mut conn = db.pool().get().await.expect("Failed to get connection");
        for resource_type in [None, Some("user")] {
            diesel::sql_query(
                "INSERT INTO roles (id, name, resource_type, resource_id, created_at, updated_at) \
                 VALUES (gen_random_uuid(), $1, $2, NULL, NOW(), NOW())",
            )
            .bind::<diesel::sql_types::Text, _>(&role_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(resource_type)
            .execute(&mut *conn)
            .await
            .expect("role insert failed");
        }
    }

    let mut config: AppConfig = test_config();
    config.register_default_role = role_name;
    let app = auth_app!(&db, config);

    let resp = register!(app, &email);
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    assert_eq!(
        users_with_email(&db, &email).await,
        0,
        "a registration with an ambiguous role creates no user"
    );
}
