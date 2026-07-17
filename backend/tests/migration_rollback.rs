// Migration rollback validation tests
//
// These tests verify that all Diesel migration files are present and valid
// without requiring a database connection. They can run in CI without a DB.

use std::fs;
use std::path::Path;

const MIGRATIONS_DIR: &str = "migrations";

fn migrations_dir() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest_dir).join(MIGRATIONS_DIR)
}

#[test]
fn all_migrations_have_up_sql() {
    let dir = migrations_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|_| panic!("Migrations dir not found: {:?}", dir));

    let mut migration_count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".keep" {
            continue;
        }

        let migration_path = entry.path();
        if !migration_path.is_dir() {
            continue;
        }

        let up_sql = migration_path.join("up.sql");
        assert!(up_sql.exists(), "Missing up.sql for migration: {}", name);
        assert!(
            fs::metadata(&up_sql).unwrap().len() > 0,
            "up.sql is empty for migration: {}",
            name
        );

        migration_count += 1;
    }

    assert!(migration_count > 0, "No migrations found");
    println!("Found {} migrations with up.sql", migration_count);
}

#[test]
fn all_migrations_have_down_sql() {
    let dir = migrations_dir();
    let entries = fs::read_dir(&dir).unwrap();

    let mut migration_count = 0;
    for entry in entries {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".keep" {
            continue;
        }

        let migration_path = entry.path();
        if !migration_path.is_dir() {
            continue;
        }

        let down_sql = migration_path.join("down.sql");
        assert!(
            down_sql.exists(),
            "Missing down.sql for migration: {}",
            name
        );

        let content = fs::read_to_string(&down_sql).unwrap();
        assert!(
            !content.trim().is_empty(),
            "down.sql is empty or whitespace-only for migration: {}",
            name
        );

        migration_count += 1;
    }

    assert_eq!(
        migration_count, 10,
        "Expected 10 migrations (found {})",
        migration_count
    );
    println!("All {} migrations have valid down.sql", migration_count);
}

#[test]
fn down_sql_contains_drop_statements() {
    let dir = migrations_dir();
    let entries = fs::read_dir(&dir).unwrap();

    let mut migrations_with_drops = 0;
    let mut migrations_without_drops = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".keep" {
            continue;
        }

        let migration_path = entry.path();
        if !migration_path.is_dir() {
            continue;
        }

        let down_sql = migration_path.join("down.sql");
        let content = fs::read_to_string(&down_sql).unwrap();

        // Check that down.sql is not empty (whitespace only)
        assert!(
            !content.trim().is_empty(),
            "down.sql is empty or whitespace-only for migration: {}",
            name
        );

        // Check for DROP/DELETE statements (excluding comment-only files)
        let has_drop = content.to_uppercase().contains("DROP TABLE")
            || content.to_uppercase().contains("DROP INDEX")
            || content.to_uppercase().contains("DROP TYPE")
            || content.to_uppercase().contains("DROP FUNCTION")
            || content.to_uppercase().contains("DELETE FROM");

        if has_drop {
            migrations_with_drops += 1;
        } else {
            migrations_without_drops.push(name);
        }
    }

    println!(
        "Migrations with DROP/DELETE: {}, without: {} ({:?})",
        migrations_with_drops,
        migrations_without_drops.len(),
        migrations_without_drops
    );

    // At least 7 out of 9 migrations should have DROP/DELETE statements
    // (The initial setup and audit_logs migrations are exceptions)
    assert!(
        migrations_with_drops >= 7,
        "Too few migrations ({} < 7) contain DROP/DELETE statements",
        migrations_with_drops
    );
}

#[test]
fn migration_names_are_chronological() {
    let dir = migrations_dir();
    let entries = fs::read_dir(&dir).unwrap();

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name == ".keep" || !e.path().is_dir() {
                None
            } else {
                Some(name)
            }
        })
        .collect();

    names.sort();

    // Verify chronological ordering (timestamp prefix)
    let mut prev_timestamp: Option<i64> = None;
    for name in &names {
        if let Some(ts_str) = name.split('_').next()
            && let Ok(ts) = ts_str.parse::<i64>()
        {
            if let Some(prev) = prev_timestamp {
                assert!(
                    ts > prev,
                    "Migration '{}' timestamp {} is not greater than previous {}",
                    name,
                    ts,
                    prev
                );
            }
            prev_timestamp = Some(ts);
        }
    }

    println!("Migration order verified: {:?}", names);
}

#[test]
fn migration_count_matches_expected() {
    let dir = migrations_dir();
    let entries = fs::read_dir(&dir).unwrap();

    let count = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| e.file_name().to_string_lossy() != ".keep")
        .count();

    assert_eq!(count, 10, "Expected exactly 10 migrations, found {}", count);
}

#[test]
fn diesel_toml_exists() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let diesel_toml = Path::new(&manifest_dir).join("diesel.toml");
    assert!(
        diesel_toml.exists(),
        "diesel.toml not found in project root"
    );
}

#[test]
fn diesel_toml_has_correct_config() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let diesel_toml = Path::new(&manifest_dir).join("diesel.toml");
    let content = fs::read_to_string(&diesel_toml).unwrap();

    assert!(
        content.contains("print_schema"),
        "diesel.toml missing print_schema config"
    );
    assert!(
        content.contains("migration"),
        "diesel.toml missing migration config"
    );
}

/// Run a live migration rollback cycle against a real PostgreSQL database.
/// Skipped when `DATABASE_URL_TEST` is not set (e.g. in unit-test-only runs).
/// When set (e.g. in CI), validates that up.sql and down.sql execute without
/// errors — a broken down.sql would only be caught during a production rollback.
#[test]
fn live_migration_rollback_cycle() {
    let db_url = match std::env::var("DATABASE_URL_TEST") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping live rollback test: DATABASE_URL_TEST not set");
            return;
        },
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_dir = Path::new(&manifest_dir).join(MIGRATIONS_DIR);

    // Count migrations
    let migration_count = fs::read_dir(&migrations_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy() != ".keep")
        .count();

    assert!(migration_count > 0, "No migrations found");
    println!("Testing rollback cycle for {} migrations", migration_count);

    // Start with a clean database - drop all tables
    drop_all_tables(&db_url);

    // Collect migration directories in chronological order
    let mut migrations: Vec<String> = fs::read_dir(&migrations_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy() != ".keep")
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    migrations.sort();

    // Apply all migrations forward
    for name in &migrations {
        let up_sql = migrations_dir.join(name).join("up.sql");
        let sql = fs::read_to_string(&up_sql)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", up_sql.display(), e));
        println!("  UP   {}", name);
        run_sql_against_db(&db_url, &sql);
    }

    // Revert all migrations in reverse order
    for name in migrations.iter().rev() {
        let down_sql = migrations_dir.join(name).join("down.sql");
        let sql = fs::read_to_string(&down_sql)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", down_sql.display(), e));
        println!("  DOWN {}", name);
        run_sql_against_db(&db_url, &sql);
    }

    // Re-apply all migrations to restore database
    for name in &migrations {
        let up_sql = migrations_dir.join(name).join("up.sql");
        let sql = fs::read_to_string(&up_sql).unwrap();
        println!("  UP   {} (restore)", name);
        run_sql_against_db(&db_url, &sql);
    }

    println!(
        "Migration rollback cycle passed for {} migrations",
        migration_count
    );
}

/// Execute raw SQL against the database via a simple psql command.
fn run_sql_against_db(database_url: &str, sql: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("psql")
        .arg(database_url)
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run psql — is it installed?");

    child
        .stdin
        .take()
        .expect("Failed to open stdin")
        .write_all(sql.as_bytes())
        .expect("Failed to write SQL to stdin");

    let output = child.wait_with_output().expect("Failed to wait for psql");
    assert!(
        output.status.success(),
        "SQL execution failed (exit {}):\n{}\nSQL:\n{}",
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr),
        sql,
    );
}

/// Drop all tables in the public schema to start fresh.
fn drop_all_tables(database_url: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let sql = "DROP SCHEMA public CASCADE; CREATE SCHEMA public;";

    let mut child = Command::new("psql")
        .arg(database_url)
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run psql — is it installed?");

    child
        .stdin
        .take()
        .expect("Failed to open stdin")
        .write_all(sql.as_bytes())
        .expect("Failed to write SQL to stdin");

    let output = child.wait_with_output().expect("Failed to wait for psql");
    assert!(
        output.status.success(),
        "Failed to drop schema: exit {}: {}",
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr),
    );
}
