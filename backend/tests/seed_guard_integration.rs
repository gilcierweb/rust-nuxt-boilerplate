//! The seeder is a development tool: it must refuse a production-like
//! environment and must enforce the strength policy on a configured
//! SEED_ADMIN_PASSWORD, both before any database work happens.
//!
//! The guard cannot be unit tested inside the binary: the standard CI job
//! compiles `--lib --bins` without the `seed` feature, so the binary is only
//! built for this feature-gated suite. That makes it the right home for the
//! end-to-end guard.

use std::process::{Command, Output};

fn run_seed(environment: &str, configured_password: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seed"));
    command.env("ENVIRONMENT", environment);
    // An unreachable database proves the guard fires first: a run that touches
    // Postgres fails with a connection error, never with the guard message.
    command.env(
        "DATABASE_URL",
        "postgres://invalid:invalid@127.0.0.1:1/none",
    );
    match configured_password {
        Some(password) => {
            command.env("SEED_ADMIN_PASSWORD", password);
        },
        None => {
            command.env_remove("SEED_ADMIN_PASSWORD");
        },
    }
    command
        .output()
        .expect("the seed binary must be runnable in the test process")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn refuses_to_seed_a_production_like_environment() {
    for environment in ["production", "staging"] {
        let output = run_seed(environment, None);
        assert!(
            !output.status.success(),
            "the seeder must refuse {environment}"
        );
        assert!(
            stderr_of(&output).contains("development tool"),
            "the refusal must name the reason, got: {}",
            stderr_of(&output)
        );
    }
}

#[test]
fn refuses_a_configured_password_that_fails_the_strength_policy() {
    let output = run_seed("development", Some("short"));
    assert!(!output.status.success());
    assert!(
        stderr_of(&output).contains("SEED_ADMIN_PASSWORD"),
        "the refusal must name the rejected variable, got: {}",
        stderr_of(&output)
    );
}

#[test]
fn a_configured_password_that_satisfies_the_policy_passes_the_guard() {
    // The guard lets the run through: the failure that follows is the
    // unreachable database, which proves the password was accepted.
    let output = run_seed("development", Some("Str0ng-P@ssw0rd!"));
    assert!(!output.status.success());
    let stderr = stderr_of(&output);
    assert!(
        !stderr.contains("development tool") && !stderr.contains("rejected"),
        "a valid configured password must pass the guard, got: {stderr}"
    );
}
