//! Integration tests of the role permissions endpoints and of the delete
//! guards of the role catalog.
//!
//! The stack is the production one: `JwtAuth` loads the grants of the subject
//! from the database on every request, so a link change is visible on the very
//! next request, which is what the "next login" assertions rely on.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The canonical catalog is seeded from the lib service. The manager role is
//! the fixture of the replace tests: every test rewrites it, so they run one at
//! a time (`--test-threads=1`) and the next seed run restores the canonical
//! links.

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
use backend::controllers::{audit_logs_controller, roles_controller};
use backend::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
use backend::middleware::require_admin_middleware::RequireAdminDomain;
use backend::repositories::AppContainer;
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

/// Creates a subject and assigns the given role ids to it.
///
/// The email is protected with the shared test keys: the member list decrypts
/// every email it returns, so a fake payload would break the read.
async fn subject_with_role_ids(db: &TestDb, role_ids: &[Uuid]) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

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

/// Creates a subject holding one role, by id.
async fn subject_with_role_id(db: &TestDb, role_id: Uuid) -> Uuid {
    subject_with_role_ids(db, &[role_id]).await
}

/// Creates a subject and assigns the given canonical roles to it.
async fn subject_with_roles(db: &TestDb, roles: &[&str]) -> Uuid {
    let mut role_ids = Vec::with_capacity(roles.len());
    for role_name in roles {
        role_ids.push(role_id(db, role_name).await);
    }
    subject_with_role_ids(db, &role_ids).await
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
                    .configure(roles_controller::config)
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

/// A role created by the test itself, free to be deleted.
async fn scratch_role(db: &TestDb) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    diesel::sql_query(
        "INSERT INTO roles (id, name, resource_type, resource_id, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, NULL, NULL, NOW(), NOW()) RETURNING id",
    )
    .bind::<diesel::sql_types::Text, _>(format!("scratch-{}", Uuid::new_v4()))
    .get_result::<IdRow>(&mut *conn)
    .await
    .expect("scratch role insert failed")
    .id
}

async fn body_json(response: actix_web::dev::ServiceResponse) -> Value {
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    serde_json::from_slice(&bytes).expect("valid json body")
}

fn codes(payload: &Value) -> Vec<String> {
    payload["permissions"]
        .as_array()
        .expect("the permissions array")
        .iter()
        .map(|item| item["code"].as_str().expect("a code").to_string())
        .collect()
}

/// Converges the links of a canonical role to the seeded set: a link outside the
/// canonical set is removed, so the assertions of this file do not depend on
/// what an earlier run left behind.
async fn converge_canonical_links(db: &TestDb, role_name: &str, codes: &[&str]) {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    diesel::sql_query(
        "DELETE FROM roles_permissions          WHERE role_id = (SELECT id FROM roles WHERE name = $1)            AND permission_id NOT IN (SELECT id FROM permissions WHERE code = ANY($2))",
    )
    .bind::<diesel::sql_types::Text, _>(role_name)
    .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(
        &codes.iter().map(|code| (*code).to_string()).collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await
    .expect("link convergence failed");

    for code in codes {
        diesel::sql_query(
            "INSERT INTO roles_permissions (role_id, permission_id, effect, created_at) \
             SELECT r.id, p.id, 0, NOW() FROM roles r, permissions p \
             WHERE r.name = $1 AND p.code = $2 \
             ON CONFLICT DO NOTHING",
        )
        .bind::<diesel::sql_types::Text, _>(role_name)
        .bind::<diesel::sql_types::Text, _>(*code)
        .execute(&mut *conn)
        .await
        .expect("link convergence failed");
    }
}

#[actix_web::test]
async fn get_returns_the_seeded_links_of_the_role() {
    seed_catalog();
    let db = TestDb::new().await;
    converge_canonical_links(
        &db,
        "manager",
        &[
            "users:read",
            "users:create",
            "users:update",
            "roles:read",
            "audit_logs:read",
        ],
    )
    .await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let manager = role_id(&db, "manager").await;

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/roles/{manager}/permissions").as_str()),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let payload = body_json(resp).await;
    let mut listed = codes(&payload);
    listed.sort();
    assert_eq!(
        listed,
        vec![
            "audit_logs:read".to_string(),
            "roles:read".to_string(),
            "users:create".to_string(),
            "users:read".to_string(),
            "users:update".to_string(),
        ],
        "the manager holds its five canonical links"
    );
    assert!(
        payload["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["effect"] == 0),
        "seeded links are allow"
    );
}

/// A role created by the test itself, holding the given links, free to be
/// rewritten without touching a canonical role.
async fn probe_role_with_links(db: &TestDb, codes: &[&str]) -> Uuid {
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
async fn put_replaces_every_link_of_the_role() {
    seed_catalog();
    let db = TestDb::new().await;
    // The editor holds the administrator codes: editing the links of a role is
    // a write over the roles resource, allowed to the admin role.
    let editor = subject_with_roles(&db, &["admin"]).await;
    // The probe holds the canonical manager set, so the rewrite is visible on a
    // subject that is not the editor.
    let probe_role = probe_role_with_links(
        &db,
        &[
            "users:read",
            "users:create",
            "users:update",
            "roles:read",
            "audit_logs:read",
        ],
    )
    .await;
    let probe_user = subject_with_role_id(&db, probe_role).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, editor);

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/roles/{probe_role}/permissions").as_str())
            .set_json(serde_json::json!({
                "permissions": [
                    { "code": "users:read", "effect": 0 },
                    { "code": "users:create", "effect": 0 },
                    { "code": "users:update", "effect": 0 },
                    { "code": "roles:read", "effect": 0 },
                    { "code": "profiles:read", "effect": 0 }
                ]
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["updated"], 5);

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/roles/{probe_role}/permissions").as_str()),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let mut listed = codes(&body_json(resp).await);
    listed.sort();
    assert_eq!(
        listed,
        vec![
            "profiles:read".to_string(),
            "roles:read".to_string(),
            "users:create".to_string(),
            "users:read".to_string(),
            "users:update".to_string(),
        ]
    );

    // The ability is rebuilt from the grants on every request, so the probe
    // user loses the audit logs read on the very next call.
    let probe_token = bearer(&config, probe_user);
    let request = authorized(
        test::TestRequest::get().uri("/admin/audit-logs"),
        &probe_token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, request).await.status(),
        StatusCode::FORBIDDEN,
        "the dropped link must refuse the audit logs read"
    );

    let request =
        authorized(test::TestRequest::get().uri("/admin/roles"), &probe_token).to_request();
    assert_eq!(
        test::call_service(&app, request).await.status(),
        StatusCode::OK,
        "the kept roles:read link must still list roles"
    );
}

#[actix_web::test]
async fn put_role_permissions_refuses_last_administrator_source() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let admin_role = role_id(&db, "admin").await;
    let token = bearer(&config, subject);

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/roles/{admin_role}/permissions").as_str())
            .set_json(serde_json::json!({
                "permissions": [
                    { "code": "users:read", "effect": 0 },
                    { "code": "users:delete", "effect": 0 }
                ]
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "the last administrator source must survive"
    );

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/roles/{admin_role}/permissions").as_str()),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        codes(&body_json(resp).await).len(),
        24,
        "a refused write leaves every link untouched"
    );
}

#[actix_web::test]
async fn put_refuses_an_unknown_effect_code_or_role() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let probe_role = probe_role_with_links(&db, &["users:read"]).await;
    let token = bearer(&config, subject);

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/roles/{probe_role}/permissions").as_str())
            .set_json(serde_json::json!({
                "permissions": [{ "code": "users:read", "effect": 7 }]
            })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "an effect outside the model is refused before the database"
    );

    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/roles/{probe_role}/permissions").as_str())
            .set_json(serde_json::json!({
                "permissions": [{ "code": "documents:read", "effect": 0 }]
            })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "a code outside the catalog is refused"
    );

    let unknown = Uuid::new_v4();
    let req = authorized(
        test::TestRequest::put()
            .uri(format!("/admin/roles/{unknown}/permissions").as_str())
            .set_json(serde_json::json!({
                "permissions": [{ "code": "users:read", "effect": 0 }]
            })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::NOT_FOUND,
        "an unknown role id is a 404"
    );
}

#[actix_web::test]
async fn delete_refuses_the_admin_and_the_default_role_and_accepts_a_scratch_one() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, subject);

    let admin_role = role_id(&db, "admin").await;
    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/roles/{admin_role}").as_str()),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "the administrator role is not deletable"
    );

    let default_role = role_id(&db, "customer").await;
    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/roles/{default_role}").as_str()),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "the registration default role is not deletable"
    );

    let scratch = scratch_role(&db).await;
    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/roles/{scratch}").as_str()),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::OK,
        "a role that grants nothing and holds no member is deletable"
    );
}

#[actix_web::test]
async fn get_members_lists_the_decrypted_emails() {
    seed_catalog();
    let db = TestDb::new().await;
    let member = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let admin_role = role_id(&db, "admin").await;

    let req = authorized(
        test::TestRequest::get().uri(format!("/admin/roles/{admin_role}/members").as_str()),
        &bearer(&config, member),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let members = body_json(resp).await;
    let entries = members.as_array().expect("the members array");
    let listed = entries
        .iter()
        .find(|entry| entry["id"] == member.to_string())
        .expect("the member of the admin role");
    let email = listed["email"].as_str().expect("a decrypted email");
    assert!(email.ends_with("@example.com"), "the email is decrypted");
    assert!(
        listed["display_name"].is_null(),
        "the subject has no profile, so the display name is null"
    );
}
