use crate::config::app_config::Environment;
use crate::services::auth_service::validate_password_strength;

/// The environment variable an operator sets to choose the seed password.
pub const SEED_PASSWORD_ENV_VAR: &str = "SEED_ADMIN_PASSWORD";

/// The documented development default the e2e suites expect. It deliberately
/// fails the strength policy, which is why it is only reachable outside
/// production-like environments.
const DEVELOPMENT_DEFAULT_PASSWORD: &str = "password123";

/// The password every seed user holds in the current run.
///
/// The seeder is a development tool: it refuses to run at all in a
/// production-like environment, so a publicly known administrator credential
/// can never reach a deployment. Outside those environments an operator may
/// choose the password through `SEED_ADMIN_PASSWORD` (which must satisfy the
/// same strength policy the runtime enforces on registration), and the
/// documented development default stays available for the test suites.
pub fn resolve_seed_admin_password(
    configured: Option<&str>,
    is_production_like: bool,
) -> Result<String, String> {
    if is_production_like {
        return Err(
            "the seeder is a development tool: refuse to seed a production-like \
             environment"
                .to_string(),
        );
    }

    let configured = configured.map(str::trim).filter(|value| !value.is_empty());

    match configured {
        Some(password) => validate_password_strength(password, "en")
            .map(|()| password.to_string())
            .map_err(|error| format!("the {SEED_PASSWORD_ENV_VAR} value is rejected: {error}")),
        None => Ok(DEVELOPMENT_DEFAULT_PASSWORD.to_string()),
    }
}

/// How the seeder learns the environment it is running in.
///
/// The guard must work before `AppConfig` can be built: loading the whole
/// config would demand every production secret before refusing to run, which
/// tells an operator nothing about the mistake. Only `ENVIRONMENT` is read,
/// through the same parser the runtime uses.
pub fn seed_environment_is_production_like() -> bool {
    let configured = std::env::var("ENVIRONMENT").unwrap_or_default();
    Environment::from_env_str(&configured).is_production_like()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_every_password_in_a_production_like_environment() {
        assert!(resolve_seed_admin_password(None, true).is_err());
        assert!(resolve_seed_admin_password(Some("Str0ng-P@ssw0rd!"), true).is_err());
    }

    #[test]
    fn uses_the_configured_password_when_it_satisfies_the_policy() {
        let resolved = resolve_seed_admin_password(Some("Str0ng-P@ssw0rd!"), false);
        assert_eq!(resolved.unwrap(), "Str0ng-P@ssw0rd!");
    }

    #[test]
    fn falls_back_to_the_development_default_when_unset() {
        assert_eq!(
            resolve_seed_admin_password(None, false).unwrap(),
            DEVELOPMENT_DEFAULT_PASSWORD
        );
    }

    #[test]
    fn treats_a_blank_value_as_unset() {
        assert_eq!(
            resolve_seed_admin_password(Some("   "), false).unwrap(),
            DEVELOPMENT_DEFAULT_PASSWORD
        );
    }

    #[test]
    fn rejects_a_configured_password_that_fails_the_policy() {
        let error = resolve_seed_admin_password(Some("short"), false).unwrap_err();
        assert!(error.contains("rejected"), "unexpected error: {error}");
    }
}
