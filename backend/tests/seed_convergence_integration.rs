//! Integration tests of the seed convergence contract.
//!
//! The seed is a binary, so the test runs the real artifact against the test
//! database and then reads the stored links. The state is dirtied on purpose
//! before the second run (an extra administrator, a missing administrator),
//! because the contract of the design is convergence: the second run repairs
//! the state instead of only adding what is missing.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`
//!
//! The keys come from `test_config`, the same deterministic values the other
//! suites use, so the emails of the seed subjects can be looked up.

use std::collections::HashMap;
use std::process::Command;

use diesel::Connection;
use diesel::prelude::*;
use uuid::Uuid;

use backend::config::test_config::test_config;
use backend::db::schema::{roles, users, users_roles};
use backend::security::SecurityService;

/// The canonical rotation of the non admin seed users, in the order the brief
/// of the design prescribes.
const ROTATION: [&str; 5] = ["manager", "editor", "viewer", "support", "customer"];

const SEED_USERS: usize = 100;

fn database_url() -> String {
    std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string())
}

fn seed_email(user_index: usize) -> String {
    format!("user{user_index:02}@example.com")
}

/// The expected role names of each seed user: only the first one is an admin,
/// the rest rotate over the canonical non admin roles.
fn expected_roles() -> HashMap<String, Vec<String>> {
    let mut expected = HashMap::new();
    for index in 1..=SEED_USERS {
        let role = if index == 1 {
            "admin".to_string()
        } else {
            ROTATION[(index - 2) % ROTATION.len()].to_string()
        };
        expected.insert(seed_email(index), vec![role]);
    }
    expected
}

/// The blind index of every seed email, keyed by the email itself.
fn blind_indexes() -> HashMap<String, Vec<u8>> {
    let security =
        SecurityService::from_config(&test_config()).expect("the test security service builds");
    (1..=SEED_USERS)
        .map(|index| {
            let email = seed_email(index);
            let index_bytes = security
                .email_blind_index(&email)
                .expect("a blind index for the seed email");
            (email, index_bytes)
        })
        .collect()
}

fn connection() -> diesel::PgConnection {
    diesel::PgConnection::establish(&database_url())
        .expect("failed to connect to the test database")
}

/// Runs the seed binary against the test database with the shared test keys.
fn run_seed() {
    let config = test_config();
    let status = Command::new(env!("CARGO_BIN_EXE_seed"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("DATABASE_URL", database_url())
        .env("MASTER_KEY", &config.master_key)
        .env("BLIND_INDEX_KEY", &config.blind_index_key)
        .status()
        .expect("failed to spawn the seed binary");

    assert!(status.success(), "the seed binary exited with a failure");
}

/// The role names the seed users actually hold.
fn stored_roles() -> HashMap<String, Vec<String>> {
    let indexes = blind_indexes();
    let mut stored: HashMap<String, Vec<String>> = indexes
        .keys()
        .map(|email| (email.clone(), Vec::new()))
        .collect();

    let wanted: Vec<Vec<u8>> = indexes.values().cloned().collect();
    let rows: Vec<(Vec<u8>, String)> = users::table
        .inner_join(users_roles::table.on(users_roles::user_id.eq(users::id)))
        .inner_join(roles::table.on(roles::id.eq(users_roles::role_id)))
        .select((users::email_blind_index, roles::name))
        .filter(users::email_blind_index.eq_any(wanted))
        .load(&mut connection())
        .expect("failed to read the seed links");

    for (blind_index, role_name) in rows {
        let email = indexes
            .iter()
            .find(|(_, value)| **value == blind_index)
            .map(|(email, _)| email.clone())
            .expect("a row of a seed user");
        stored
            .get_mut(&email)
            .expect("the entry of the seed user")
            .push(role_name);
    }
    stored
}

fn id_of_user(email: &str) -> Uuid {
    let indexes = blind_indexes();
    users::table
        .filter(users::email_blind_index.eq(indexes[email].clone()))
        .select(users::id)
        .first::<Uuid>(&mut connection())
        .expect("the seed user exists")
}

fn id_of_role(role_name: &str) -> Uuid {
    roles::table
        .filter(roles::name.eq(role_name))
        .select(roles::id)
        .first::<Uuid>(&mut connection())
        .expect("the canonical role exists")
}

/// How many seed users hold the administrator role.
fn administrator_holders() -> i64 {
    let indexes = blind_indexes();
    let admin_role = id_of_role("admin");
    users::table
        .inner_join(users_roles::table.on(users_roles::user_id.eq(users::id)))
        .filter(
            users::email_blind_index
                .eq_any(indexes.values().cloned())
                .and(users_roles::role_id.eq(admin_role)),
        )
        .count()
        .get_result::<i64>(&mut connection())
        .expect("failed to count the administrator holders")
}

#[actix_web::test]
async fn the_seed_converges_every_link() {
    run_seed();

    // Dirt the state on purpose: an extra administrator, and the first user
    // loses its own administrator link.
    let mut conn = connection();
    diesel::delete(
        users_roles::table.filter(
            users_roles::user_id
                .eq(id_of_user(&seed_email(1)))
                .and(users_roles::role_id.eq(id_of_role("admin"))),
        ),
    )
    .execute(&mut conn)
    .expect("the administrator link removal");
    diesel::delete(
        users_roles::table.filter(
            users_roles::user_id
                .eq(id_of_user(&seed_email(2)))
                .and(users_roles::role_id.eq(id_of_role("admin"))),
        ),
    )
    .execute(&mut conn)
    .expect("the stale administrator link removal");
    diesel::insert_into(users_roles::table)
        .values((
            users_roles::user_id.eq(id_of_user(&seed_email(2))),
            users_roles::role_id.eq(id_of_role("admin")),
        ))
        .execute(&mut conn)
        .expect("the extra administrator link insert");
    drop(conn);

    run_seed();

    let expected = expected_roles();
    let stored = stored_roles();
    let mut mismatches = Vec::new();
    for (email, wanted) in &expected {
        let actual = stored.get(email).cloned().unwrap_or_default();
        if &actual != wanted {
            mismatches.push(format!("{email}: expected {wanted:?}, holds {actual:?}"));
        }
    }

    assert!(
        mismatches.is_empty(),
        "every seed user converges to its canonical role, but: {}",
        mismatches.join("; ")
    );
}

#[actix_web::test]
async fn only_the_first_seed_user_holds_the_administrator_role() {
    run_seed();
    assert_eq!(
        administrator_holders(),
        1,
        "exactly one seed subject holds the administrator role"
    );
}
