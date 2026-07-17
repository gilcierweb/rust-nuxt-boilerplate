// Test repository infrastructure using mockall's auto-generated mocks
//
// This module provides test fixtures and utilities for working with repository mocks.
// The traits are marked with #[cfg_attr(test, mockall::automock)] which
// automatically generates MockIUserRepository, MockIProfileRepository, etc.
// 
// Usage in tests:
// ```rust
// use crate::repositories::test_utils::mocks::mock_container;
// use crate::repositories::users_repository::MockIUserRepository;
// use crate::models::user::{NewUser, User};
//
// let mut mock_repo = MockIUserRepository::new();
// mock_repo.expect_find().returning(|_| Ok(user_fixture()));
// ```

use crate::models::role::Role;
use crate::models::profile::Profile;
use crate::models::refresh_token::RefreshToken;
use crate::models::audit_log::AuditLog;
use crate::models::user::User;
use crate::models::user_role::UserRole;
use chrono::Utc;
use uuid::Uuid;

/// Create a test user fixture.
pub fn user_fixture() -> User {
    User {
        id: Uuid::new_v4(),
        email_blind_index: vec![1, 2, 3, 4],
        email_encrypted: vec![5, 6, 7, 8],
        encrypted_password: "$argon2id$v=19$m=65536,t=3,p=1$test$test".to_string(),
        reset_password_token_digest: None,
        reset_password_sent_at: None,
        remember_created_at: None,
        sign_in_count: 0,
        current_sign_in_at: None,
        last_sign_in_at: None,
        current_sign_in_ip: None,
        last_sign_in_ip: None,
        confirmation_token_digest: None,
        confirmed_at: Some(Utc::now()),
        confirmation_sent_at: None,
        unconfirmed_email_blind_index: None,
        unconfirmed_email_encrypted: None,
        failed_attempts: 0,
        unlock_token_digest: None,
        locked_at: None,
        otp_secret: None,
        otp_enabled_at: None,
        otp_backup_codes: None,
        encryption_key_version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test user with specific ID.
pub fn user_fixture_with_id(id: Uuid) -> User {
    let mut user = user_fixture();
    user.id = id;
    user
}

/// Create a test role fixture.
pub fn role_fixture() -> Role {
    Role {
        id: Uuid::new_v4(),
        name: "test_role".to_string(),
        resource_type: None,
        resource_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test role with specific name.
pub fn role_fixture_with_name(name: &str) -> Role {
    let mut role = role_fixture();
    role.name = name.to_string();
    role
}

/// Create a test profile fixture.
pub fn profile_fixture() -> Profile {
    Profile {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        full_name: Some("John Doe".to_string()),
        nickname: None,
        bio: None,
        avatar: None,
        birthday: None,
        cpf_encrypted: None,
        cpf_blind_index: None,
        phone_encrypted: None,
        phone_blind_index: None,
        whatsapp_encrypted: None,
        whatsapp_blind_index: None,
        status: true,
        social_network: serde_json::json!({}),
        encryption_key_version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test profile with specific user ID.
pub fn profile_fixture_for_user(user_id: Uuid) -> Profile {
    let mut profile = profile_fixture();
    profile.user_id = user_id;
    profile
}

/// Create a test refresh token fixture.
pub fn refresh_token_fixture() -> RefreshToken {
    RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token_hash: "test_hash".to_string(),
        device_info: Some("test-device".to_string()),
        ip_address: Some("127.0.0.1/32".parse().unwrap()),
        expires_at: Utc::now() + chrono::Duration::days(30),
        revoked_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test audit log fixture.
pub fn audit_log_fixture() -> AuditLog {
    AuditLog {
        id: Uuid::new_v4(),
        actor_user_id: Some(Uuid::new_v4()),
        actor_role_snapshot: None,
        action: "create".to_string(),
        resource_type: "user".to_string(),
        resource_id: Some(Uuid::new_v4()),
        ip_address: Some("127.0.0.1/32".parse().unwrap()),
        user_agent: Some("test-agent".to_string()),
        request_id: None,
        changes: serde_json::json!({}),
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
        prev_hash: None,
        hash: "a".repeat(64),
    }
}

/// Create a test user role fixture.
pub fn user_role_fixture(user_id: Uuid, role_id: Uuid) -> UserRole {
    UserRole {
        user_id,
        role_id,
    }
}

/// Helper to setup common mock expectations for a user repository.
pub fn setup_user_repo_find(
    mock_repo: &mut crate::repositories::users_repository::MockIUserRepository,
    user: User,
) {
    let user_id = user.id;
    let user_clone = user.clone();
    mock_repo
        .expect_find()
        .withf(move |id| *id == user_id)
        .times(1)
        .returning(move |_| Ok(user_clone.clone()));
}

/// Helper to setup common mock expectations for finding user by email blind index.
pub fn setup_user_repo_find_by_email(
    mock_repo: &mut crate::repositories::users_repository::MockIUserRepository,
    user: User,
) {
    let email_blind_index = user.email_blind_index.clone();
    let user_clone = user.clone();
    mock_repo
        .expect_find_by_email()
        .withf(move |blind_index| blind_index == &email_blind_index)
        .times(1)
        .returning(move |_| Ok(Some(user_clone.clone())));
}

/// Helper to setup common mock expectations for a role repository.
pub fn setup_role_repo_find(
    mock_repo: &mut crate::repositories::roles_repository::MockIRoleRepository,
    role: Role,
) {
    let role_id = role.id;
    let role_clone = role.clone();
    mock_repo
        .expect_find()
        .withf(move |id| *id == role_id)
        .times(1)
        .returning(move |_| Ok(role_clone.clone()));
}

/// Helper to setup common mock expectations for a profile repository.
pub fn setup_profile_repo_find_by_user_id(
    mock_repo: &mut crate::repositories::profiles_repository::MockIProfileRepository,
    profile: Profile,
) {
    let user_id = profile.user_id;
    let profile_clone = profile.clone();
    mock_repo
        .expect_find_by_user_id()
        .withf(move |id| *id == user_id)
        .times(1)
        .returning(move |_| Ok(Some(profile_clone.clone())));
}

/// Helper to setup common mock expectations for a refresh token repository.
pub fn setup_refresh_token_repo_find_by_hash(
    mock_repo: &mut crate::repositories::refresh_tokens_repository::MockIRefreshTokenRepository,
    token: RefreshToken,
) {
    let token_hash = token.token_hash.clone();
    let token_clone = token.clone();
    mock_repo
        .expect_find_by_token_hash()
        .withf(move |hash| hash == &token_hash)
        .times(1)
        .returning(move |_| Ok(Some(token_clone.clone())));
}

/// Test data module with commonly used test values.
pub mod test_data {
    use super::*;
    use std::sync::LazyLock;

    pub static TEST_USER_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
    pub static TEST_ROLE_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
    pub static TEST_PROFILE_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);
    pub static TEST_TOKEN_HASH: &str = "test_token_hash_abc123";
    pub static TEST_ACTION_CREATE: &str = "create";
    pub static TEST_ACTION_UPDATE: &str = "update";
    pub static TEST_ACTION_DELETE: &str = "delete";
    pub static TEST_RESOURCE_USER: &str = "user";
    pub static TEST_RESOURCE_PROFILE: &str = "profile";
    pub static TEST_RESOURCE_ROLE: &str = "role";
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_user_fixture() {
        let user = user_fixture();
        assert!(!user.id.is_nil());
        assert_eq!(user.sign_in_count, 0);
        assert!(user.confirmed_at.is_some());
    }

    #[test]
    fn test_role_fixture() {
        let role = role_fixture();
        assert!(!role.id.is_nil());
        assert_eq!(role.name, "test_role");
    }

    #[test]
    fn test_profile_fixture() {
        let profile = profile_fixture();
        assert!(!profile.id.is_nil());
        assert!(!profile.user_id.is_nil());
        assert!(profile.status);
    }

    #[test]
    fn test_refresh_token_fixture() {
        let token = refresh_token_fixture();
        assert!(!token.id.is_nil());
        assert!(!token.user_id.is_nil());
        assert!(!token.token_hash.is_empty());
    }

    #[test]
    fn test_audit_log_fixture() {
        let log = audit_log_fixture();
        assert!(!log.id.is_nil());
        assert_eq!(log.action, "create");
    }

    #[test]
    fn test_user_role_fixture() {
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let ur = user_role_fixture(user_id, role_id);
        assert_eq!(ur.user_id, user_id);
        assert_eq!(ur.role_id, role_id);
    }
}