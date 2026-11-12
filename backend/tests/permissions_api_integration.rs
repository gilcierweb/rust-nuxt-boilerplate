//! Integration tests of the admin permissions API (catalog CRUD).
//!
//! The stack is the production one: `JwtAuth` loads the grants of the subject
//! and `RequireAdminDomain` gates the scope, so the routes are exercised the
//! way the frontend calls them.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The canonical catalog is seeded from the lib service. Because the admin
//! role links every canonical code, the create and delete cycle of one test
//! frees its own code first: the seed never deletes, so the state converges
//! back on the next run.

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
use backend::controllers::permissions_controller;
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

/// Creates a subject and assigns the given canonical roles to it.
///
/// The email is protected with the shared test keys: the admin list decrypts
/// every user of the database, so a fake payload would break the other tests.
async fn subject_with_roles(db: &TestDb, roles: &[&str]) -> Uuid {
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
                    .configure(permissions_controller::config),
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

fn authorized(request: actix_web::test::TestRequest, token: &str) -> actix_web::test::TestRequest {
    request.insert_header((
        actix_web::http::header::AUTHORIZATION,
        format!("Bearer {token}"),
    ))
}

/// The id of a canonical permission code.
async fn permission_id(db: &TestDb, code: &str) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    diesel::sql_query("SELECT id FROM permissions WHERE code = $1")
        .bind::<diesel::sql_types::Text, _>(code)
        .get_result::<IdRow>(&mut *conn)
        .await
        .expect("permission lookup failed")
        .id
}

/// Removes every link of a code, so the row is free to be deleted again.
async fn detach_code(db: &TestDb, code: &str) {
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    diesel::sql_query(
        "DELETE FROM roles_permissions WHERE permission_id IN \
         (SELECT id FROM permissions WHERE code = $1)",
    )
    .bind::<diesel::sql_types::Text, _>(code)
    .execute(&mut *conn)
    .await
    .expect("link removal failed");
}

async fn body_json(response: actix_web::dev::ServiceResponse) -> Value {
    let bytes = to_bytes(response.into_body()).await.expect("body read");
    serde_json::from_slice(&bytes).expect("valid json body")
}

#[actix_web::test]
async fn admin_lists_permissions_with_their_names_and_roles() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?per_page=100"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    let items = json["data"].as_array().expect("the data array");
    let users_read = items
        .iter()
        .find(|item| item["code"] == "users:read")
        .expect("the canonical users:read code");

    assert_eq!(users_read["name"], "Users read");
    assert_eq!(users_read["resource"], "users");
    assert_eq!(users_read["action"], "read");
    let roles = users_read["roles"].as_array().expect("the roles array");
    assert!(
        roles
            .iter()
            .any(|role| role["role_name"] == "admin" && role["effect"] == 0),
        "the admin role must carry the allow link of users:read"
    );
}

#[actix_web::test]
async fn the_list_search_matches_the_code() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?q=upload"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    let items = json["data"].as_array().expect("the data array");
    assert_eq!(items.len(), 4, "the upload resource holds four codes");
    assert!(items.iter().all(|item| {
        item["code"]
            .as_str()
            .is_some_and(|code| code.starts_with("upload:"))
    }));
}

// A query string such as `?sort_by` (a key with no value) reads as an empty
// string, and a client that omits the value asks for the same default order as
// a client that omits the key. This list is the last one of the admin domain
// validating its sort field on its own, so a valueless key was the one place
// the tables still answered 422 on the first render.
#[actix_web::test]
async fn the_list_answers_a_valueless_sort_key_with_the_default_order() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?page=1&per_page=10&sort_by&sort_dir"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "a valueless sort key means no ordering, not a validation error"
    );

    let json = body_json(resp).await;
    assert!(
        json["data"].as_array().is_some(),
        "the list answers a data array"
    );
}

#[actix_web::test]
async fn the_list_refuses_a_sort_field_outside_the_allowlist() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    // The names are translated, so their alphabetical order is not the one the
    // reader expects: a silent fallback would hide the mistake of the client.
    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?sort_by=description"),
        &bearer(&config, subject),
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[actix_web::test]
async fn the_list_search_folds_the_case_of_the_human_name() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    // The canonical names are capitalized ("Users read"), so a term typed in
    // lower case has to match: every other admin list folds the case and the
    // operator should not have to know the exact capitalization.
    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?q=users+read"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    let items = json["data"].as_array().expect("the data array");
    assert!(
        items.iter().any(|item| item["code"] == "users:read"),
        "the lower case name fragment matches the capitalized name"
    );
}

#[actix_web::test]
async fn a_sort_field_outside_the_allowlist_is_refused() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?sort_by=description"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?sort_by=code&sort_dir=desc"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // The allowlist names the fields the response serializes. A field the
    // catalog list does not return is a client mistake too: answering it with
    // a silent fallback to the name order would hide the mistake.
    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions?sort_by=updated_at"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn the_catalog_lists_the_enum_resources_and_actions() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let req = authorized(
        test::TestRequest::get().uri("/admin/permissions/catalog"),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp).await;
    let resources = json["resources"].as_array().expect("the resources array");
    let actions = json["actions"].as_array().expect("the actions array");
    assert_eq!(resources.len(), 6, "the six domain resources");
    assert_eq!(actions.len(), 4, "the four crud actions");
    assert!(resources.iter().any(|resource| resource["code"] == "users"
        && resource["label_key"] == "authorization.resources.users"));
    assert!(
        actions.iter().any(|action| action["code"] == "read"
            && action["label_key"] == "authorization.actions.read")
    );
}

#[actix_web::test]
async fn create_refuses_a_duplicate_code_and_an_unknown_pair() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, subject);

    let req = authorized(
        test::TestRequest::post()
            .uri("/admin/permissions?per_page=100")
            .set_json(serde_json::json!({
                "name": "Users read",
                "resource": "users",
                "action": "read"
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT, "duplicate code");

    let req = authorized(
        test::TestRequest::post()
            .uri("/admin/permissions?per_page=100")
            .set_json(serde_json::json!({
                "name": "Documents read",
                "resource": "documnets",
                "action": "read"
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "unknown resource"
    );

    let req = authorized(
        test::TestRequest::post()
            .uri("/admin/permissions?per_page=100")
            .set_json(serde_json::json!({
                "name": "Users publish",
                "resource": "users",
                "action": "publish"
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "unknown action"
    );
}

#[actix_web::test]
async fn admin_creates_updates_and_deletes_a_permission() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, subject);

    // The admin role links every canonical code, so the delete guard would
    // refuse the row: the test frees its own code first.
    detach_code(&db, "audit_logs:update").await;
    let existing = permission_id(&db, "audit_logs:update").await;

    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/permissions/{existing}").as_str()),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["deleted"], true);

    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/permissions/{existing}").as_str()),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let req = authorized(
        test::TestRequest::post()
            .uri("/admin/permissions?per_page=100")
            .set_json(serde_json::json!({
                "name": "Audit logs update",
                "resource": "audit_logs",
                "action": "update",
                "description": "Updates an audit entry"
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let created_id = created["id"].as_str().expect("the created id").to_string();
    assert_eq!(created["code"], "audit_logs:update");
    assert_eq!(created["resource"], "audit_logs");
    assert_eq!(created["action"], "update");
    assert_eq!(created["name"], "Audit logs update");

    let req = authorized(
        test::TestRequest::patch()
            .uri(format!("/admin/permissions/{created_id}").as_str())
            .set_json(serde_json::json!({
                "name": "Audit logs update (edit)",
                "description": "Updates one audit entry"
            })),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["name"], "Audit logs update (edit)");
    assert_eq!(updated["description"], "Updates one audit entry");
    assert_eq!(
        updated["code"], "audit_logs:update",
        "the code is the identity and never moves"
    );
    assert_eq!(updated["resource"], "audit_logs");
    assert_eq!(updated["action"], "update");

    // The cycle ends by removing the row it created: the catalog seed is
    // additive, so the canonical permission comes back with its canonical
    // name on the next run. Leaving the renamed row behind would break the
    // naming assertion of the catalog migration suite, which shares this
    // database and runs after this one.
    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/permissions/{created_id}").as_str()),
        &token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn delete_refuses_a_permission_still_linked_to_a_role() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["admin"]).await;
    let app = admin_app!(&db);
    let config = test_config();

    let users_read = permission_id(&db, "users:read").await;
    let req = authorized(
        test::TestRequest::delete().uri(format!("/admin/permissions/{users_read}").as_str()),
        &bearer(&config, subject),
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[actix_web::test]
async fn a_viewer_without_the_authority_is_refused_everywhere() {
    seed_catalog();
    let db = TestDb::new().await;
    let subject = subject_with_roles(&db, &["viewer"]).await;
    let app = admin_app!(&db);
    let config = test_config();
    let token = bearer(&config, subject);
    let target = permission_id(&db, "users:read").await;

    let get = authorized(
        test::TestRequest::get().uri("/admin/permissions?per_page=100"),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, get).await.status(),
        StatusCode::FORBIDDEN
    );

    let catalog = authorized(
        test::TestRequest::get().uri("/admin/permissions/catalog"),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, catalog).await.status(),
        StatusCode::FORBIDDEN
    );

    let post = authorized(
        test::TestRequest::post()
            .uri("/admin/permissions?per_page=100")
            .set_json(serde_json::json!({
                "name": "Users read",
                "resource": "users",
                "action": "read"
            })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, post).await.status(),
        StatusCode::FORBIDDEN
    );

    let patch = authorized(
        test::TestRequest::patch()
            .uri(format!("/admin/permissions/{target}").as_str())
            .set_json(serde_json::json!({ "name": "Renamed" })),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, patch).await.status(),
        StatusCode::FORBIDDEN
    );

    let delete = authorized(
        test::TestRequest::delete().uri(format!("/admin/permissions/{target}").as_str()),
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete).await.status(),
        StatusCode::FORBIDDEN
    );
}
