//! Integration tests for the `add_rbac_catalog` migration and the canonical
//! catalog seed.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST (default below matches the auth tests)
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! Migration assertions cover the schema shape (DDL only, migrations never
//! seed). Catalog assertions run the seed service twice and verify the second
//! run changes nothing.

use deadpool::managed::Pool;
use diesel::Connection;
use diesel::QueryableByName;
use diesel::sql_types::BigInt;
use diesel::sql_types::Text;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use backend::services::rbac_catalog_service;

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

    fn database_url() -> String {
        std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string())
    }
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct NameRow {
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName)]
struct EffectRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::SmallInt)]
    effect: i16,
}

/// The 24 canonical catalog codes: 6 resources x 4 CRUD actions.
const CATALOG_CODES: [&str; 24] = [
    "users:read",
    "users:create",
    "users:update",
    "users:delete",
    "roles:read",
    "roles:create",
    "roles:update",
    "roles:delete",
    "audit_logs:read",
    "audit_logs:create",
    "audit_logs:update",
    "audit_logs:delete",
    "profiles:read",
    "profiles:create",
    "profiles:update",
    "profiles:delete",
    "upload:read",
    "upload:create",
    "upload:update",
    "upload:delete",
    "permissions:read",
    "permissions:create",
    "permissions:update",
    "permissions:delete",
];

async fn count(db: &TestDb, sql: &str) -> i64 {
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    diesel::sql_query(sql)
        .load::<CountRow>(&mut *conn)
        .await
        .expect("count query failed")
        .remove(0)
        .count
}

/// Serializes the shared catalog seed across parallel tests: concurrent
/// select-then-insert runs race into a unique violation otherwise.
static CATALOG_SEED_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Seeds the canonical catalog through the lib service and asserts that a
/// second run is a no-op (idempotent seed, no revocation).
async fn ensure_catalog_seeded() {
    let database_url = TestDb::database_url();
    let mut conn = diesel::PgConnection::establish(&database_url)
        .expect("Failed to connect for the catalog seed");

    let _guard = CATALOG_SEED_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    rbac_catalog_service::ensure_canonical_catalog(&mut conn)
        .expect("first catalog seed run failed");
    rbac_catalog_service::ensure_canonical_catalog(&mut conn)
        .expect("second catalog seed run must be idempotent");
}

/// `permissions.name` exists, is NOT NULL, and is a varchar.
#[actix_web::test]
async fn permissions_name_column_is_not_null_varchar() {
    let db = TestDb::new().await;
    let found = count(
        &db,
        "SELECT count(*) AS count FROM information_schema.columns \
         WHERE table_name = 'permissions' AND column_name = 'name' \
         AND is_nullable = 'NO' AND data_type = 'character varying'",
    )
    .await;
    assert_eq!(found, 1, "permissions.name must be NOT NULL varchar");
}

/// `roles_permissions.effect` exists, is NOT NULL smallint with default 0.
#[actix_web::test]
async fn roles_permissions_effect_column_has_default_allow() {
    let db = TestDb::new().await;
    let found = count(
        &db,
        "SELECT count(*) AS count FROM information_schema.columns \
         WHERE table_name = 'roles_permissions' AND column_name = 'effect' \
         AND is_nullable = 'NO' AND data_type = 'smallint' AND column_default = '0'",
    )
    .await;
    assert_eq!(
        found, 1,
        "roles_permissions.effect must be NOT NULL smallint default 0"
    );
}

/// The two unique indexes the catalog depends on.
#[actix_web::test]
async fn unique_indexes_exist() {
    let db = TestDb::new().await;
    let found = count(
        &db,
        "SELECT count(*) AS count FROM pg_indexes \
         WHERE indexname IN ('uq_permissions_name', 'uq_roles_scope')",
    )
    .await;
    assert_eq!(
        found, 2,
        "uq_permissions_name and uq_roles_scope must exist"
    );
}

/// After the seed service runs: all 24 codes exist with human readable names.
#[actix_web::test]
async fn catalog_seed_creates_all_24_permissions() {
    let db = TestDb::new().await;
    ensure_catalog_seeded().await;
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let codes: Vec<String> = CATALOG_CODES.iter().map(|c| c.to_string()).collect();
    let rows = diesel::sql_query(
        "SELECT name AS name FROM permissions WHERE code = ANY($1) ORDER BY code",
    )
    .bind::<diesel::sql_types::Array<Text>, _>(&codes)
    .load::<NameRow>(&mut *conn)
    .await
    .expect("catalog query failed");
    assert_eq!(rows.len(), 24, "all 24 catalog codes must exist");
    for row in rows {
        assert!(
            row.name.ends_with(" read")
                || row.name.ends_with(" create")
                || row.name.ends_with(" update")
                || row.name.ends_with(" delete"),
            "permission name must be human readable: {:?}",
            row.name
        );
    }
}

/// After the seed: 5 unscoped canonical roles plus the self scoped customer.
#[actix_web::test]
async fn catalog_seed_creates_canonical_roles() {
    let db = TestDb::new().await;
    ensure_catalog_seeded().await;

    let unscoped: Vec<String> = vec![
        "admin".to_string(),
        "manager".to_string(),
        "editor".to_string(),
        "viewer".to_string(),
        "support".to_string(),
    ];
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let rows = diesel::sql_query(
        "SELECT name AS name FROM roles \
         WHERE name = ANY($1) AND resource_type IS NULL AND resource_id IS NULL \
         ORDER BY name",
    )
    .bind::<diesel::sql_types::Array<Text>, _>(&unscoped)
    .load::<NameRow>(&mut *conn)
    .await
    .expect("unscoped roles query failed");
    assert_eq!(rows.len(), 5, "5 unscoped canonical roles must exist");

    let customer = count(
        &db,
        "SELECT count(*) AS count FROM roles \
         WHERE name = 'customer' AND resource_type = 'user' AND resource_id IS NULL",
    )
    .await;
    assert_eq!(
        customer, 1,
        "customer role must be self scoped (resource_type='user', resource_id NULL)"
    );
}

/// Canonical allow links: admin 24, manager 5, editor 3, viewer 2, support 2,
/// customer 2.
#[actix_web::test]
async fn catalog_seed_creates_canonical_links() {
    let db = TestDb::new().await;
    ensure_catalog_seeded().await;

    let expected: [(&str, Option<&str>, i64); 6] = [
        ("admin", None, 24),
        ("manager", None, 5),
        ("editor", None, 3),
        ("viewer", None, 2),
        ("support", None, 2),
        ("customer", Some("user"), 2),
    ];

    for (role_name, resource_type, expected_count) in expected {
        let sql = format!(
            "SELECT count(*) AS count FROM roles r \
             JOIN roles_permissions rp ON rp.role_id = r.id \
             WHERE r.name = '{}' AND r.resource_type {} AND rp.effect = 0",
            role_name,
            resource_type
                .map(|t| format!("= '{t}'"))
                .unwrap_or_else(|| "IS NULL".to_string()),
        );
        let found = count(&db, &sql).await;
        assert_eq!(
            found, expected_count,
            "role {role_name} must hold {expected_count} allow links"
        );
    }
}

/// Every seeded canonical link has effect = 0 (allow); the deny effect is
/// written by the admin api, never by the seed.
#[actix_web::test]
async fn canonical_links_have_no_deny() {
    let db = TestDb::new().await;
    ensure_catalog_seeded().await;
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let rows = diesel::sql_query(
        "SELECT p.name AS name, rp.effect AS effect \
         FROM roles r \
         JOIN roles_permissions rp ON rp.role_id = r.id \
         JOIN permissions p ON p.id = rp.permission_id \
         WHERE r.name = 'admin' AND r.resource_type IS NULL \
         ORDER BY p.code",
    )
    .load::<EffectRow>(&mut *conn)
    .await
    .expect("admin links query failed");
    assert_eq!(rows.len(), 24);
    for row in rows {
        assert_eq!(row.effect, 0, "link {:?} must be allow", row.name);
    }
}

/// Legacy roles seeded by the seed module are preserved (additive only).
#[actix_web::test]
async fn legacy_roles_are_preserved() {
    let db = TestDb::new().await;
    ensure_catalog_seeded().await;
    let legacy: Vec<String> = vec![
        "moderator".to_string(),
        "creator".to_string(),
        "subscriber".to_string(),
        "agency".to_string(),
    ];
    let mut conn = db.pool().get().await.expect("Failed to get connection");
    let rows =
        diesel::sql_query("SELECT name AS name FROM roles WHERE name = ANY($1) ORDER BY name")
            .bind::<diesel::sql_types::Array<Text>, _>(&legacy)
            .load::<NameRow>(&mut *conn)
            .await
            .expect("legacy roles query failed");
    assert_eq!(rows.len(), 4, "legacy roles must survive the seed");
}
