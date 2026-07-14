use crate::models::role::{NewRole, Role};
use crate::models::profile::{NewProfile, Profile};
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::models::audit_log::{NewAuditLog, AuditLog};
use crate::repositories::traits::roles_trait::IRoleRepository;
use crate::repositories::traits::profiles_trait::IProfileRepository;
use crate::repositories::traits::refresh_tokens_trait::IRefreshTokenRepository;
use crate::repositories::traits::audit_logs_trait::IAuditLogRepository;
use crate::repositories::traits::user_roles_trait::IUserRoleRepository;
use crate::repositories::container::AppContainer;
use crate::config::app_config::mock_container;
use crate::security::SecurityService;
use async_trait::async_trait;
use mockall::*;
use uuid::Uuid;
use chrono::Utc;

mock! {
    pub IRoleRepository {}
    #[async_trait]
    impl IRoleRepository for IRoleRepository {
        async fn all(&self) -> Result<Vec<Role>, diesel::result::Error>;
        async fn find(&self, id: &Uuid) -> Result<Role, diesel::result::Error>;
        async fn create(&self, item: &NewRole) -> Result<Role, diesel::result::Error>;
        async fn update(&self, id: &Uuid, item: &NewRole) -> Result<Role, diesel::result::Error>;
        async fn destroy(&self, id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn find_by_name(&self, name: &str) -> Result<Option<Role>, diesel::result::Error>;
    }
}

mock! {
    pub IProfileRepository {}
    #[async_trait]
    impl IProfileRepository for IProfileRepository {
        async fn all(&self) -> Result<Vec<Profile>, diesel::result::Error>;
        async fn find(&self, id: &Uuid) -> Result<Profile, diesel::result::Error>;
        async fn create(&self, item: &NewProfile) -> Result<Profile, diesel::result::Error>;
        async fn update(&self, id: &Uuid, item: &NewProfile) -> Result<Profile, diesel::result::Error>;
        async fn destroy(&self, id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Option<Profile>, diesel::result::Error>;
    }
}

mock! {
    pub IRefreshTokenRepository {}
    #[async_trait]
    impl IRefreshTokenRepository for IRefreshTokenRepository {
        async fn all(&self) -> Result<Vec<RefreshToken>, diesel::result::Error>;
        async fn find(&self, id: &Uuid) -> Result<RefreshToken, diesel::result::Error>;
        async fn create(&self, item: &NewRefreshToken) -> Result<RefreshToken, diesel::result::Error>;
        async fn update(&self, id: &Uuid, item: &NewRefreshToken) -> Result<RefreshToken, diesel::result::Error>;
        async fn destroy(&self, id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Option<RefreshToken>, diesel::result::Error>;
        async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>, diesel::result::Error>;
        async fn revoke(&self, id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn revoke_all_for_user(&self, user_id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn find_valid_by_user_id(&self, user_id: &Uuid) -> Result<Option<RefreshToken>, diesel::result::Error>;
    }
}

mock! {
    pub IAuditLogRepository {}
    #[async_trait]
    impl IAuditLogRepository for IAuditLogRepository {
        async fn all(&self) -> Result<Vec<AuditLog>, diesel::result::Error>;
        async fn find(&self, id: &Uuid) -> Result<AuditLog, diesel::result::Error>;
        async fn create(&self, item: &NewAuditLog) -> Result<AuditLog, diesel::result::Error>;
        async fn update(&self, id: &Uuid, item: &NewAuditLog) -> Result<AuditLog, diesel::result::Error>;
        async fn destroy(&self, id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<AuditLog>, diesel::result::Error>;
        async fn find_by_resource(&self, resource_type: &str, resource_id: &Uuid) -> Result<Vec<AuditLog>, diesel::result::Error>;
    }
}

mock! {
    pub IUserRoleRepository {}
    #[async_trait]
    impl IUserRoleRepository for IUserRoleRepository {
        async fn all(&self) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error>;
        async fn find(&self, id: &Uuid) -> Result<(Uuid, Uuid), diesel::result::Error>;
        async fn create(&self, user_id: &Uuid, role_id: &Uuid) -> Result<(), diesel::result::Error>;
        async fn destroy(&self, user_id: &Uuid, role_id: &Uuid) -> Result<usize, diesel::result::Error>;
        async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Vec<Uuid>, diesel::result::Error>;
        async fn find_by_role_id(&self, role_id: &Uuid) -> Result<Vec<Uuid>, diesel::result::Error>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::Role;
    use crate::models::profile::Profile;
    use crate::models::refresh_token::RefreshToken;
    use crate::models::audit_log::AuditLog;
    use chrono::Utc;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn test_role_repository_create() {
        let mut mock_repo = MockIRoleRepository::new();
        let new_role = NewRole {
            id: Uuid::new_v4(),
            name: "test_role".to_string(),
            description: Some("Test role description".to_string()),
        };
        let role = Role {
            id: Uuid::new_v4(),
            name: "test_role".to_string(),
            description: Some("Test role description".to_string()),
        };

        mock_repo
            .expect_create()
            .withf(|item| item.name == "test_role")
            .times(1)
            .returning(|item| Ok(Role {
                id: item.id,
                name: item.name.clone(),
                description: item.description.clone(),
            }));

        let result = mock_repo.create(&new_role).await;
        assert!(result.is_ok());
        let created_role = result.unwrap();
        assert_eq!(created_role.name, "test_role");
    }

    #[actix_rt::test]
    async fn test_profile_repository_find_by_user_id() {
        let mut mock_repo = MockIProfileRepository::new();
        let user_id = Uuid::new_v4();
        let profile = Profile {
            id: Uuid::new_v4(),
            user_id,
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            display_name: Some("John Doe".to_string()),
            bio: None,
            avatar_url: None,
            cover_url: None,
            birthday: None,
            age_verified: false,
            country: None,
            state: None,
            city: None,
            social_network: serde_json::json!({}),
            is_creator: false,
            is_agency: false,
            status: "active".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        mock_repo
            .expect_find_by_user_id()
            .withf(|id| *id == user_id)
            .times(1)
            .returning(move |id| Ok(Some(Profile {
                id: Uuid::new_v4(),
                user_id: *id,
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                display_name: Some("John Doe".to_string()),
                bio: None,
                avatar_url: None,
                cover_url: None,
                birthday: None,
                age_verified: false,
                country: None,
                state: None,
                city: None,
                social_network: serde_json::json!({}),
                is_creator: false,
                is_agency: false,
                status: "active".to_string(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            })));

        let result = mock_repo.find_by_user_id(&user_id).await;
        assert!(result.is_ok());
        let profile_option = result.unwrap();
        assert!(profile_option.is_some());
        let found_profile = profile_option.unwrap();
        assert_eq!(found_profile.user_id, user_id);
        assert_eq!(found_profile.first_name, Some("John".to_string()));
    }

    #[actix_rt::test]
    async fn test_refresh_token_repository_find_by_token_hash() {
        let mut mock_repo = MockIRefreshTokenRepository::new();
        let token_hash = "abc123def456";
        let refresh_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: token_hash.to_string(),
            device_info: Some("test-device".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            expires_at: Utc::now() + chrono::Duration::days(30),
            revoked_at: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        mock_repo
            .expect_find_by_token_hash()
            .withf(|hash| hash == token_hash)
            .times(1)
            .returning(move |hash| Ok(Some(RefreshToken {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                token_hash: hash.clone(),
                device_info: Some("test-device".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                expires_at: Utc::now() + chrono::Duration::days(30),
                revoked_at: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            })));

        let result = mock_repo.find_by_token_hash(token_hash).await;
        assert!(result.is_ok());
        let token_option = result.unwrap();
        assert!(token_option.is_some());
        let found_token = token_option.unwrap();
        assert_eq!(found_token.token_hash, token_hash);
    }

    #[actix_rt::test]
    async fn test_audit_log_repository_create() {
        let mut mock_repo = MockIAuditLogRepository::new();
        let new_audit_log = NewAuditLog {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            resource_type: "user".to_string(),
            resource_id: Uuid::new_v4(),
            action: "create".to_string(),
            changes: Some(serde_json::json!({})),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            created_at: Utc::now().naive_utc(),
        };
        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            resource_type: "user".to_string(),
            resource_id: Uuid::new_v4(),
            action: "create".to_string(),
            changes: Some(serde_json::json!({})),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            created_at: Utc::now().naive_utc(),
        };

        mock_repo
            .expect_create()
            .withf(|item| {
                item.resource_type == "user" && item.action == "create"
            })
            .times(1)
            .returning(|item| Ok(AuditLog {
                id: item.id,
                user_id: item.user_id,
                resource_type: item.resource_type.clone(),
                resource_id: item.resource_id,
                action: item.action.clone(),
                changes: item.changes.clone(),
                ip_address: item.ip_address.clone(),
                user_agent: item.user_agent.clone(),
                created_at: item.created_at,
            }));

        let result = mock_repo.create(&new_audit_log).await;
        assert!(result.is_ok());
        let created_log = result.unwrap();
        assert_eq!(created_log.resource_type, "user");
        assert_eq!(created_log.action, "create");
    }

    #[actix_rt::test]
    async fn test_user_role_repository_create() {
        let mut mock_repo = MockIUserRoleRepository::new();
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();

        mock_repo
            .expect_create()
            .withf(|uid, rid| *uid == user_id && *rid == role_id)
            .times(1)
            .returning(|_, _| Ok(()));

        let result = mock_repo.create(&user_id, &role_id).await;
        assert!(result.is_ok());
    }
}