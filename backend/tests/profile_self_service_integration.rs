//! Integration tests of the self service profile endpoints.
//!
//! The stack is the production one: `JwtAuth` loads the grants of the subject
//! and builds the ability, so the instance authorization is exercised the way
//! the frontend calls it.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The subjects are created with direct SQL and carry the shared test keys.

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
use backend::models::profile::NewProfile;
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

/// Creates a user with its profile and assigns the given canonical roles.
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

    let profile = NewProfile::for_user(user.id, protected_email.key_version);
    diesel::insert_into(backend::db::schema::profiles::table)
        .values(&profile)
        .execute(&mut *conn)
        .await
        .expect("profile insert failed");

    for role_name in roles {
        // The canonical customer is self scoped, so the lookup is by name only:
        // every canonical name is unique in the catalog.
        diesel::sql_query(
            "INSERT INTO users_roles (user_id, role_id) \
             SELECT $1, id FROM roles WHERE name = $2 ORDER BY created_at LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(user.id)
        .bind::<diesel::sql_types::Text, _>(*role_name)
        .execute(&mut *conn)
        .await
        .expect("user role insert failed");
    }

    user.id
}

/// The production authenticated stack outside `/admin`.
macro_rules! profile_app {
    ($db:expr) => {{
        let config = Arc::new(test_config());
        let redis = redis_pool();
        let container = web::Data::new(AppContainer::new(
            $db.pool().clone(),
            redis.clone(),
            (*config).clone(),
        ));

        test::init_service(
            App::new().app_data(container).service(
                web::scope("/api/v1")
                    .wrap(CsrfProtection::new(vec![]))
                    .wrap(JwtAuth::new(JwtAuthConfig::new(vec![])))
                    .configure(backend::controllers::profile_controller::config),
            ),
        )
        .await
    }};
}

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

fn authorized(request: actix_web::test::TestRequest, token: &str) -> actix_web::test::TestRequest {
    request.insert_header((
        actix_web::http::header::AUTHORIZATION,
        format!("Bearer {token}"),
    ))
}

async fn body_json(response: actix_web::dev::ServiceResponse) -> Value {
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    serde_json::from_slice(&bytes).expect("valid json body")
}

#[actix_web::test]
async fn a_customer_reads_and_patches_its_own_profile() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["customer"]).await;
    let app = profile_app!(&db);
    let token = bearer(&test_config(), subject);

    let req = authorized(test::TestRequest::get().uri("/api/v1/profile"), &token).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let profile = body_json(resp).await;
    assert_eq!(profile["user_id"], subject.to_string());

    let req = authorized(
        test::TestRequest::patch()
            .uri("/api/v1/profile")
            .set_json(serde_json::json!({
                "first_name": "Ada",
                "last_name": "Lovelace"
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let patched = body_json(resp).await;
    assert_eq!(patched["first_name"], "Ada");
    assert_eq!(patched["last_name"], "Lovelace");
}

#[actix_web::test]
async fn a_viewer_reads_its_profile_but_cannot_patch_it() {
    seed_catalog();
    let db = TestDb::new().await;
    // The viewer holds an unscoped `profiles:read` and no `profiles:update`.
    let subject = subject_with_roles(&db, &["viewer"]).await;
    let app = profile_app!(&db);
    let token = bearer(&test_config(), subject);

    let req = authorized(test::TestRequest::get().uri("/api/v1/profile"), &token).to_request();
    assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);

    let req = authorized(
        test::TestRequest::patch()
            .uri("/api/v1/profile")
            .set_json(serde_json::json!({ "first_name": "Ada" })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::FORBIDDEN,
        "a class level read never grants the write"
    );
}

#[actix_web::test]
async fn an_admin_reads_its_own_profile() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = profile_app!(&db);
    let token = bearer(&test_config(), subject);

    let req = authorized(test::TestRequest::get().uri("/api/v1/profile"), &token).to_request();
    assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
}
