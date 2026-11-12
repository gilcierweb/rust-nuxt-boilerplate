//! Integration tests of the user role assignment endpoints: the escalation
//! guard, the two lockout guards and the enrichments of the user list.
//!
//! The stack is the production one: `JwtAuth` rebuilds the ability of the
//! subject from the database on every request, so an assignment is visible on
//! the very next call.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The canonical catalog is seeded from the lib service. The write tests use
//! subjects the test creates itself, so the canonical assignments of the seed
//! stay untouched.

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
use backend::controllers::{audit_logs_controller, users_controller};
use backend::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
use backend::middleware::require_admin_middleware::RequireAdminDomain;
use backend::repositories::AppContainer;
use backend::security::SecurityService;
use backend::services::rbac_catalog_service;

/// Serializes the shared catalog seed across the parallel tests.
static CATALOG_SEED_MUTEX: Mutex<()> = Mutex::new(());

struct TestDb {
    pool: Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl TestDb {
    async fn new() -> Self {
        let database_url = database_url();
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

/// Seeds the canonical catalog (idempotent, serialized against other tests).
fn seed_catalog() {
    let _guard = CATALOG_SEED_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut conn = diesel::PgConnection::establish(&database_url())
        .expect("Failed to connect for the catalog seed");
    rbac_catalog_service::ensure_canonical_catalog(&mut conn).expect("catalog seed run failed");
}

/// The id of a canonical role.
async fn role_id(db: &TestDb, name: &str) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    diesel::sql_query("SELECT id FROM roles WHERE name = $1 ORDER BY created_at LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(name)
        .get_result::<IdRow>(&mut *conn)
        .await
        .unwrap_or_else(|error| panic!("role lookup failed for {name}: {error}"))
        .id
}

/// The users holding a role, in creation order.
async fn holders_of(db: &TestDb, role_id: Uuid) -> Vec<Uuid> {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let rows: Vec<IdRow> = diesel::sql_query(
        "SELECT u.id FROM users_roles ur JOIN users u ON u.id = ur.user_id \
         WHERE ur.role_id = $1 ORDER BY u.created_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(role_id)
    .load(&mut *conn)
    .await
    .expect("holder lookup failed");

    rows.into_iter().map(|row| row.id).collect()
}

/// Inserts a bare user row and assigns it the given role ids.
///
/// The email is protected with the shared test keys: the admin user list
/// decrypts every user of the page it returns, so a fake payload would break
/// the read of the other tests.
async fn create_user(db: &TestDb, role_ids: &[Uuid]) -> Uuid {
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

    for role_id in role_ids {
        diesel::sql_query("INSERT INTO users_roles (user_id, role_id) VALUES ($1, $2)")
            .bind::<diesel::sql_types::Uuid, _>(user.id)
            .bind::<diesel::sql_types::Uuid, _>(*role_id)
            .execute(&mut *conn)
            .await
            .expect("user role insert failed");
    }

    user.id
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
                    // The production order: JwtAuth is the outermost layer, so
                    // the ability is attached before the domain gate asks for it.
                    .wrap(RequireAdminDomain::new())
                    .wrap(JwtAuth::new(JwtAuthConfig::new(vec![])))
                    .configure(users_controller::config)
                    .configure(audit_logs_controller::config),
            ),
        )
        .await
    }};
}

/// A bearer token for the subject.
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

async fn body_json(response: actix_web::dev::ServiceResponse) -> Value {
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    serde_json::from_slice(&bytes).expect("valid json body")
}

/// The status and the body of a response, for the assertions that need both.
async fn response_json(response: actix_web::dev::ServiceResponse) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    let text = String::from_utf8_lossy(&bytes).to_string();
    let json =
        serde_json::from_str::<Value>(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }));
    (status, json)
}

fn assigned_names(payload: &Value) -> Vec<String> {
    payload["roles"]
        .as_array()
        .expect("the roles array")
        .iter()
        .map(|role| role["name"].as_str().expect("a role name").to_string())
        .collect()
}

#[actix_web::test]
async fn admin_assigns_manager_to_a_fresh_user() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = create_user(&db, &[role_id(&db, "admin").await]).await;
    let fresh = create_user(&db, &[]).await;
    let manager = role_id(&db, "manager").await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, editor);

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/users/{fresh}/roles").as_str())
            .set_json(serde_json::json!({ "role_ids": [manager] })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    let (status, payload) = response_json(resp).await;
    if status != StatusCode::OK {
        println!("PUT BODY: {payload}");
    }
    assert_eq!(status, StatusCode::OK);
    assert_eq!(assigned_names(&payload), vec!["manager"]);

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/users/{fresh}/roles").as_str()),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    let (status, payload) = response_json(resp).await;
    if status != StatusCode::OK {
        println!("PUT BODY: {payload}");
    }
    assert_eq!(status, StatusCode::OK);
    assert_eq!(assigned_names(&payload), vec!["manager"]);

    // The ability is rebuilt from the grants on every request, so the fresh
    // user reads users on the very next call.
    let req = authorized(
        test::TestRequest::get().uri("/admin/users"),
        &bearer(&config, fresh),
    )
    .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);

    // And a role outside the request cannot be granted.
    let unknown = Uuid::new_v4();
    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/users/{fresh}/roles").as_str())
            .set_json(serde_json::json!({ "role_ids": [unknown] })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[actix_web::test]
async fn a_viewer_cannot_grant_a_role_beyond_its_own_authority() {
    seed_catalog();
    let db = TestDb::new().await;
    // The manager holds `users:update`, so it may edit the assignments of a
    // user, but its authority never covers the admin role: granting it would be
    // an escalation.
    let principal = create_user(&db, &[role_id(&db, "manager").await]).await;
    let target = create_user(&db, &[]).await;
    let admin_role = role_id(&db, "admin").await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/users/{target}/roles").as_str())
            .set_json(serde_json::json!({ "role_ids": [admin_role] })),
        &bearer(&config, principal),
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "granting authorities beyond the actor's own is refused"
    );

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/users/{target}/roles").as_str()),
        &bearer(&config, principal),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        assigned_names(&body_json(resp).await).is_empty(),
        "a refused assignment leaves the target without roles"
    );
}

#[actix_web::test]
async fn removing_last_admin_role_assignment_is_refused() {
    seed_catalog();
    let db = TestDb::new().await;
    let admin_role = role_id(&db, "admin").await;
    // The only holder of the literal admin role in this database.
    let sole_holder = create_user(&db, &[admin_role]).await;
    leave_one_administrator_source(&db, sole_holder).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, sole_holder);

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/users/{sole_holder}/roles").as_str())
            .set_json(serde_json::json!({ "role_ids": [] })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "the last assignment of the admin role cannot be removed"
    );

    let holders = holders_of(&db, admin_role).await;
    assert_eq!(
        holders,
        vec![sole_holder],
        "a refused write leaves the assignment untouched"
    );
}

#[actix_web::test]
async fn removing_the_only_administrator_holder_is_refused() {
    seed_catalog();
    let db = TestDb::new().await;
    let admin_role = role_id(&db, "admin").await;
    let holder = create_user(&db, &[admin_role]).await;
    leave_one_administrator_source(&db, holder).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, holder);

    // Replacing the administrator codes by a single read authority would leave
    // no one able to repair the roles.
    let users_read = "users:read".to_string();
    let _ = users_read;
    let scratch_role = scratch_role(&db, &["users:read"]).await;

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/users/{holder}/roles").as_str())
            .set_json(serde_json::json!({ "role_ids": [scratch_role] })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "the last administrator source must survive"
    );

    assert_eq!(
        holders_of(&db, admin_role).await,
        vec![holder],
        "a refused write leaves the holder in place"
    );
}

/// The administrator codes as owned strings, the shape a raw bind reads.
fn administrator_codes() -> Vec<String> {
    ["roles:read", "roles:update"]
        .iter()
        .map(|code| (*code).to_string())
        .collect()
}

/// Leaves exactly one user holding the administrator codes: every other
/// assignment of a role that holds both is removed, so the lockout guards of
/// this file see a single source.
async fn leave_one_administrator_source(db: &TestDb, holder: Uuid) {
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    diesel::sql_query(
        "DELETE FROM users_roles \
         WHERE user_id <> $1 \
           AND role_id IN ( \
                SELECT rp.role_id FROM roles_permissions rp \
                  JOIN permissions p ON p.id = rp.permission_id \
                 WHERE rp.effect = 0 AND p.code = ANY($2) \
                 GROUP BY rp.role_id \
                HAVING count(DISTINCT p.code) = 2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(holder)
    .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(administrator_codes())
    .execute(&mut *conn)
    .await
    .expect("administrator source isolation failed");
}

/// A role created by the test itself, holding the given links.
async fn scratch_role(db: &TestDb, codes: &[&str]) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let name = format!("probe-{}", Uuid::new_v4());
    let role = diesel::sql_query(
        "INSERT INTO roles (id, name, resource_type, resource_id, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, NULL, NULL, NOW(), NOW()) RETURNING id",
    )
    .bind::<diesel::sql_types::Text, _>(&name)
    .get_result::<IdRow>(&mut *conn)
    .await
    .expect("probe role insert failed")
    .id;

    for code in codes {
        diesel::sql_query(
            "INSERT INTO roles_permissions (role_id, permission_id, effect, created_at) \
             SELECT $1, id, 0, NOW() FROM permissions WHERE code = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(role)
        .bind::<diesel::sql_types::Text, _>(*code)
        .execute(&mut *conn)
        .await
        .expect("probe link insert failed");
    }

    role
}

#[actix_web::test]
async fn the_user_list_carries_the_role_names() {
    seed_catalog();
    let db = TestDb::new().await;
    let editor = create_user(&db, &[role_id(&db, "admin").await]).await;
    create_user(&db, &[role_id(&db, "manager").await]).await;
    let app = admin_app!(&db);
    let token = bearer(&test_config(), editor);

    // The test database holds the subjects of every other suite, so the editor
    // is looked up page by page instead of assuming the first page. The
    // newest first order puts the subject of this test on the first page.
    let mut listed = None;
    for page in 1..=5 {
        let req = authorized(
            test::TestRequest::get()
                .uri(format!("/admin/users?per_page=100&page={page}&sort_dir=desc").as_str()),
            &token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let json = body_json(resp).await;
        let items = json["data"].as_array().expect("the data array");
        if let Some(item) = items.iter().find(|item| item["id"] == editor.to_string()) {
            listed = Some(item.clone());
            break;
        }
    }

    let listed = listed.expect("the editor in the list");
    let roles = listed["roles"].as_array().expect("the roles column");
    assert_eq!(
        roles
            .iter()
            .map(|role| role.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["admin"],
        "the list carries the role names in the same query"
    );
}
