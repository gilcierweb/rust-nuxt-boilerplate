//! Integration tests of the server side search and of the sort allowlists of
//! the admin lists.
//!
//! The stack is the production one: `JwtAuth` loads the grants of the subject
//! and `RequireAdminDomain` gates the scope, so the routes are exercised the
//! way the frontend calls them.
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
use backend::controllers::{audit_logs_controller, roles_controller, users_controller};
use backend::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
use backend::middleware::require_admin_middleware::RequireAdminDomain;
use backend::models::audit_log::NewAuditLog;
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

/// A user with a decodable email, holding the given canonical role.
async fn user_with_profile(db: &TestDb, role_name: &str, full_name: &str) -> (Uuid, String) {
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

    diesel::insert_into(backend::db::schema::profiles::table)
        .values(backend::models::profile::NewProfile {
            user_id: user.id,
            full_name: Some(full_name.to_string()),
            ..backend::models::profile::NewProfile::for_user(user.id, protected_email.key_version)
        })
        .execute(&mut *conn)
        .await
        .expect("profile insert failed");

    diesel::sql_query(
        "INSERT INTO users_roles (user_id, role_id) \
         SELECT $1, id FROM roles WHERE name = $2 ORDER BY created_at LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user.id)
    .bind::<diesel::sql_types::Text, _>(role_name)
    .execute(&mut *conn)
    .await
    .expect("user role insert failed");

    (user.id, email)
}

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
                    .wrap(RequireAdminDomain::new())
                    .wrap(JwtAuth::new(JwtAuthConfig::new(vec![])))
                    .configure(users_controller::config)
                    .configure(roles_controller::config)
                    .configure(audit_logs_controller::config),
            ),
        )
        .await
    }};
}

fn bearer(config: &backend::config::AppConfig, subject: Uuid) -> String {
    create_token(subject, Uuid::new_v4(), 1, &config.jwt_secret, 3600)
        .expect("failed to mint the test token")
}

fn authorized(request: actix_web::test::TestRequest, token: &str) -> actix_web::test::TestRequest {
    request.insert_header((
        actix_web::http::header::AUTHORIZATION,
        format!("Bearer {token}"),
    ))
}

/// The status and the body of a response.
async fn response_json(response: actix_web::dev::ServiceResponse) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    let json = serde_json::from_slice::<Value>(&bytes).expect("valid json body");
    (status, json)
}

#[actix_web::test]
async fn the_user_list_searches_by_name_fragment() {
    seed_catalog();
    let db = TestDb::new().await;
    // The name carries a unique token: the test database is shared with the
    // other suites, so a plain fragment like "beeble" would match leftovers.
    let tag = &Uuid::new_v4().to_string()[..8];
    let full_name = format!("Zaphod {tag} Beeblebrox");
    let editor = user_with_profile(&db, "admin", &full_name).await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor.0);

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/users?q=zaphod+{}", tag).as_str()),
        &token,
    )
    .to_request();
    let (status, json) = response_json(test::call_service(&app, req).await).await;
    assert_eq!(status, StatusCode::OK);

    let items = json["data"].as_array().expect("the data array");
    assert_eq!(items.len(), 1, "the fragment matches one display name");
    assert_eq!(items[0]["id"], editor.0.to_string());
    assert_eq!(items[0]["full_name"], full_name);

    // The total is the metadata of the same query the page came from, so it
    // counts the filtered set. A count of the whole table would tell the
    // pagination that many pages of results exist where none are left.
    assert_eq!(
        json["pagination"]["total"], 1,
        "the total counts the rows the search matched, not the whole table"
    );
}

// A query string such as `?sort_by` (a key with no value) reads as an empty
// string, and a client that omits the value asks for the same default order as
// a client that omits the key. Refusing it would break every list whose table
// sends the two keys on the first render, before a column is chosen.
#[actix_web::test]
async fn the_lists_answer_a_valueless_sort_key_with_the_default_order() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = user_with_profile(&db, "admin", "Zaphod Beeblebrox").await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor.0);

    // The permissions list is not part of this suite's stack: its app configures
    // the users, roles and audit logs scopes only.
    for url in [
        "/admin/users?sort_by&sort_dir",
        "/admin/roles?sort_by&sort_dir",
        "/admin/audit-logs?sort_by&sort_dir",
    ] {
        let req = authorized(test::TestRequest::get().uri(url), &token).to_request();
        let (status, json) = response_json(test::call_service(&app, req).await).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{url} must answer the default order, not a validation error"
        );
        assert!(
            json["data"].as_array().is_some(),
            "{url} must answer a data array"
        );
    }
}

#[actix_web::test]
async fn the_user_list_rejects_a_sort_field_outside_the_allowlist() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = user_with_profile(&db, "admin", "Zaphod Beeblebrox").await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor.0);

    // The email is encrypted, so its blind index order is not the alphabetical
    // order of the address: the field is not part of the allowlist.
    let req = authorized(
        test::TestRequest::get().uri("/admin/users?sort_by=email"),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let req = authorized(
        test::TestRequest::get().uri("/admin/users?sort_by=display_name&sort_dir=desc"),
        &token,
    )
    .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
}

#[actix_web::test]
async fn the_user_list_matches_an_exact_email_and_no_fragment() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = user_with_profile(&db, "admin", "Zaphod Beeblebrox").await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor.0);

    // A fragment of the address cannot match: only the HMAC index exists.
    let fragment = editor.1.split('@').next().unwrap().to_string();
    let local = &fragment[..8];
    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/users?q={local}").as_str()),
        &token,
    )
    .to_request();
    let (status, json) = response_json(test::call_service(&app, req).await).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        json["data"]
            .as_array()
            .is_some_and(|items| items.is_empty()),
        "a fragment of an encrypted email matches nothing"
    );

    // The exact address matches through the blind index.
    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/users?q={}", editor.1).as_str()),
        &token,
    )
    .to_request();
    let (status, json) = response_json(test::call_service(&app, req).await).await;
    assert_eq!(status, StatusCode::OK);
    let items = json["data"].as_array().expect("the data array");
    assert_eq!(items.len(), 1, "the exact address matches one row");
    assert_eq!(items[0]["id"], editor.0.to_string());
}

#[actix_web::test]
async fn the_role_list_searches_by_name_fragment() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = user_with_profile(&db, "admin", "Zaphod Beeblebrox").await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor.0);

    let req = authorized(test::TestRequest::get().uri("/admin/roles?q=mana"), &token).to_request();
    let (status, json) = response_json(test::call_service(&app, req).await).await;
    assert_eq!(status, StatusCode::OK);
    let items = json["data"].as_array().expect("the data array");
    assert!(
        items.iter().any(|role| role["name"] == "manager"),
        "the fragment matches the role name"
    );

    // A sort field outside the allowlist is refused instead of silently falling
    // back to the default order.
    let req = authorized(
        test::TestRequest::get().uri("/admin/roles?sort_by=resource_id"),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[actix_web::test]
async fn the_audit_log_list_searches_by_action() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = user_with_profile(&db, "admin", "Zaphod Beeblebrox").await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor.0);

    // Seed one entry with a distinctive action.
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    diesel::insert_into(backend::db::schema::audit_logs::table)
        .values(NewAuditLog {
            actor_user_id: Some(editor.0),
            actor_role_snapshot: Some("admin".to_string()),
            action: "probe.action.create".to_string(),
            resource_type: "User".to_string(),
            resource_id: Some(Uuid::new_v4()),
            ip_address: None,
            user_agent: None,
            request_id: None,
            changes: serde_json::json!({}),
            metadata: serde_json::json!({}),
            prev_hash: None,
            hash: "a".repeat(64),
        })
        .execute(&mut *conn)
        .await
        .expect("audit log insert failed");
    drop(conn);

    let req = authorized(
        test::TestRequest::get().uri("/admin/audit-logs?q=probe.action"),
        &token,
    )
    .to_request();
    let (status, json) = response_json(test::call_service(&app, req).await).await;
    assert_eq!(status, StatusCode::OK);
    let items = json["data"].as_array().expect("the data array");
    assert!(
        items.iter().all(|entry| entry["action"]
            .as_str()
            .is_some_and(|action| action.starts_with("probe.action"))),
        "the search filters the log by action"
    );

    let req = authorized(
        test::TestRequest::get().uri("/admin/audit-logs?sort_by=changes"),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
