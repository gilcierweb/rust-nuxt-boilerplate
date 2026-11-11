//! Integration tests for the rbac grants data access (`get_user_grants` and
//! `user_ids_holding_all_codes`).
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST (default below)
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The canonical catalog is seeded through the lib service; users and role
//! assignments are created with direct SQL so the assertions stay independent
//! from the seed binary.

use deadpool::managed::Pool;
use diesel::prelude::*;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use backend::repositories::traits::users_trait::IUserRepository;
use backend::repositories::users_repository::UsersRepository;
use backend::services::rbac_catalog_service;

/// Serializes the shared catalog seed across parallel tests.
static CATALOG_SEED_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct TestDb {
    pool: Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl TestDb {
    async fn new() -> Self {
        let database_url = TestDb::database_url();
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

/// Seeds the canonical catalog (idempotent, serialized against other tests).
fn ensure_catalog_seeded() {
    let _guard = CATALOG_SEED_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut conn = diesel::PgConnection::establish(&TestDb::database_url())
        .expect("Failed to connect for the catalog seed");
    rbac_catalog_service::ensure_canonical_catalog(&mut conn).expect("catalog seed run failed");
}

/// Returns the id of a canonical role, serialized against the shared seed.
fn canonical_role_id(role_name: &str) -> Uuid {
    let _guard = CATALOG_SEED_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut conn = diesel::PgConnection::establish(&TestDb::database_url())
        .expect("Failed to connect for the role lookup");
    rbac_catalog_service::ensure_role(&mut conn, role_name, None, None)
        .expect("canonical role lookup failed")
}

/// Inserts a bare user row and binds the given role id when provided.
async fn create_user(db: &TestDb, role_id: Option<Uuid>) -> Uuid {
    let mut conn = db.pool().get().await.expect("Failed to get connection");

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let unique = Uuid::new_v4().to_string();
    let user = diesel::sql_query(
        "INSERT INTO users (email_blind_index, email_encrypted) \
         VALUES (decode(md5($1), 'hex'), decode(md5($2), 'hex')) RETURNING id",
    )
    .bind::<diesel::sql_types::Text, _>(&unique)
    .bind::<diesel::sql_types::Text, _>(&format!("{unique}-encrypted"))
    .get_result::<IdRow>(&mut *conn)
    .await
    .expect("user insert failed");
    let user_id = user.id;

    if let Some(role_id) = role_id {
        diesel::sql_query("INSERT INTO users_roles (user_id, role_id) VALUES ($1, $2)")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(role_id)
            .execute(&mut *conn)
            .await
            .expect("user role insert failed");
    }

    user_id
}

/// The manager role must yield exactly its canonical five allow grants.
#[actix_web::test]
async fn manager_user_gets_five_allow_grants() {
    let db = TestDb::new().await;
    ensure_catalog_seeded();

    let manager_role_id = canonical_role_id("manager");
    let user_id = create_user(&db, Some(manager_role_id)).await;

    let repo = UsersRepository::new(db.pool().clone());
    let grants = repo
        .get_user_grants(&user_id)
        .await
        .expect("get_user_grants failed");

    assert_eq!(
        grants.len(),
        5,
        "manager holds exactly five canonical links"
    );
    for grant in grants {
        assert_eq!(grant.role_name, "manager");
        assert_eq!(grant.resource_type, None, "manager is unscoped");
        assert_eq!(grant.resource_id, None);
        assert_eq!(grant.effect, Some(0), "seeded links are allow");
        assert!(grant.permission_code.is_some());
    }
}

/// A user with no role assignment holds no grants at all.
#[actix_web::test]
async fn user_without_roles_gets_no_grants() {
    let db = TestDb::new().await;
    ensure_catalog_seeded();

    let user_id = create_user(&db, None).await;

    let repo = UsersRepository::new(db.pool().clone());
    let grants = repo
        .get_user_grants(&user_id)
        .await
        .expect("get_user_grants failed");

    assert!(grants.is_empty(), "user without roles must hold no grants");
}

/// Only users holding both administrator codes appear in the holders query.
#[actix_web::test]
async fn holders_of_administrator_codes_include_admin_user() {
    let db = TestDb::new().await;
    ensure_catalog_seeded();

    let admin_role_id = canonical_role_id("admin");
    let user_id = create_user(&db, Some(admin_role_id)).await;

    let repo = UsersRepository::new(db.pool().clone());
    let admin_codes = ["roles:read".to_string(), "roles:update".to_string()];
    let holders = repo
        .user_ids_holding_all_codes(&admin_codes)
        .await
        .expect("user_ids_holding_all_codes failed");

    assert!(
        holders.contains(&user_id),
        "admin role holder must satisfy both administrator codes"
    );
}
