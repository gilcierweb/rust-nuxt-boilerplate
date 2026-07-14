use crate::models::user::{NewUser, User};
use crate::repositories::traits::users_trait::IUserRepository;
use crate::repositories::container::AppContainer;
use crate::config::app_config::mock_container;
use crate::security::SecurityService;
use crate::services::token::generate_random_token;
use crate::services::token_service::hash_token;
use async_trait::async_trait;
use mockall::*;
use uuid::Uuid;

mock! {
    pub IUserRepository {}
    #[async_trait]
    impl IUserRepository for IUserRepository {
        async fn all(&self) -> Result<Vec<User>, diesel::result::Error>;
        async fn find(&self, id: &Uuid) -> Result<User, diesel::result::Error>;
        async fn create(&self, item: &NewUser) -> Result<User, diesel::result::Error>;
        async fn update(&self, id: &Uuid, item: &NewUser) -> Result<User, diesel::result::Error>;
        async fn destroy(&self, id: &Uuid) -> Result<usize, diesel::result::Error>;

        async fn find_by_username_or_email(
            &self,
            username_or_email: &str,
            email_blind_index: &[u8],
        ) -> Result<Option<User>, diesel::result::Error>;
        async fn find_by_email(&self, email_blind_index: &[u8]) -> Result<Option<User>, diesel::result::Error>;
        async fn find_by_reset_token_digest(&self, token_digest: &str) -> Result<Option<User>, diesel::result::Error>;
        async fn update_login_info(
            &self,
            id: &Uuid,
            current_sign_in_at: Option<chrono::NaiveDateTime>,
            last_sign_in_at: Option<chrono::NaiveDateTime>,
            current_sign_in_ip: Option<ipnet::IpNet>,
            last_sign_in_ip: Option<ipnet::IpNet>,
        ) -> Result<User, diesel::result::Error>;
        async fn update_password(&self, id: &Uuid, encrypted_password: &str) -> Result<usize, diesel::result::Error>;
        async fn update_reset_token(
            &self,
            id: &Uuid,
            token_digest: Option<String>,
            sent_at: Option<chrono::NaiveDateTime>,
        ) -> Result<usize, diesel::result::Error>;
        async fn update_pending_email(
            &self,
            user_id: &Uuid,
            blind_index: &[u8],
            encrypted_email: &[u8],
            token_digest: &str,
            sent_at: chrono::NaiveDateTime,
        ) -> Result<usize, diesel::result::Error>;

        async fn confirm_email(&self, token_digest: &str) -> Result<usize, diesel::result::Error>;
        async fn record_failed_login(&self, user_id: &Uuid, max_attempts: i32) -> Result<usize, diesel::result::Error>;
        async fn record_successful_login(
            &self,
            user_id: &Uuid,
            ip: Option<ipnet::IpNet>,
        ) -> Result<usize, diesel::result::Error>;
        async fn get_user_roles(&self, user_id: &Uuid) -> Result<Vec<String>, diesel::result::Error>;
        async fn get_user_permissions(&self, user_id: &Uuid) -> Result<Vec<String>, diesel::result::Error>;
        async fn create_password_reset_token(
            &self,
            user_id: &Uuid,
            token_digest: &str,
            sent_at: chrono::NaiveDateTime,
        ) -> Result<usize, diesel::result::Error>;
        async fn reset_password(&self, token_digest: &str, new_password: &str) -> Result<usize, diesel::result::Error>;
        async fn set_otp_secret(&self, user_id: &Uuid, secret: &str) -> Result<usize, diesel::result::Error>;
        async fn enable_2fa(&self, user_id: &Uuid, backup_codes: &[String]) -> Result<usize, diesel::result::Error>;
        async fn disable_2fa(&self, user_id: &Uuid) -> Result<usize, diesel::result::Error>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::User;
    use crate::models::user::NewUser;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_user() -> NewUser {
        let security = SecurityService::from_env().unwrap();
        let protected_email = security.protect_email("test@example.com").unwrap();

        NewUser {
            id: Uuid::new_v4(),
            email_blind_index: protected_email.blind_index,
            email_encrypted: protected_email.encrypted,
            encrypted_password: "hashed_password".to_string(),
            confirmation_token_digest: Some("token_digest".to_string()),
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: None,
            encryption_key_version: 1,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }

    #[actix_rt::test]
    async fn test_create_user() {
        let mut mock_repo = MockIUserRepository::new();
        let test_user = User {
            id: Uuid::new_v4(),
            email_blind_index: vec![1, 2, 3],
            email_encrypted: vec![4, 5, 6],
            encrypted_password: "hashed_password".to_string(),
            confirmation_token_digest: Some("token_digest".to_string()),
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: None,
            encryption_key_version: 1,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        mock_repo
            .expect_create()
            .withf(|item| {
                item.email == "test@example.com"
                    && item.password == "password123"
            })
            .times(1)
            .returning(|item| Ok(User {
                id: Uuid::new_v4(),
                email_blind_index: item.email_blind_index.clone(),
                email_encrypted: item.email_encrypted.clone(),
                encrypted_password: item.encrypted_password.clone(),
                confirmation_token_digest: item.confirmation_token_digest.clone(),
                unconfirmed_email_blind_index: item.unconfirmed_email_blind_index.clone(),
                unconfirmed_email_encrypted: item.unconfirmed_email_encrypted.clone(),
                encryption_key_version: item.encryption_key_version,
                created_at: item.created_at,
                updated_at: item.updated_at,
            }));

        let new_user = create_test_user();
        let result = mock_repo.create(&new_user).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email_blind_index, test_user.email_blind_index);
    }

    #[actix_rt::test]
    async fn test_find_user_by_id() {
        let mut mock_repo = MockIUserRepository::new();
        let user_id = Uuid::new_v4();
        let test_user = User {
            id: user_id,
            email_blind_index: vec![1, 2, 3],
            email_encrypted: vec![4, 5, 6],
            encrypted_password: "hashed_password".to_string(),
            confirmation_token_digest: Some("token_digest".to_string()),
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: None,
            encryption_key_version: 1,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        mock_repo
            .expect_find()
            .withf(|id| *id == user_id)
            .times(1)
            .returning(move |id| Ok(User {
                id: *id,
                email_blind_index: test_user.email_blind_index.clone(),
                email_encrypted: test_user.email_encrypted.clone(),
                encrypted_password: test_user.encrypted_password.clone(),
                confirmation_token_digest: test_user.confirmation_token_digest.clone(),
                unconfirmed_email_blind_index: test_user.unconfirmed_email_blind_index.clone(),
                unconfirmed_email_encrypted: test_user.unconfirmed_email_encrypted.clone(),
                encryption_key_version: test_user.encryption_key_version,
                created_at: test_user.created_at,
                updated_at: test_user.updated_at,
            }));

        let result = mock_repo.find(&user_id).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
    }

    #[actix_rt::test]
    async fn test_find_user_not_found() {
        let mut mock_repo = MockIUserRepository::new();
        let user_id = Uuid::new_v4();

        mock_repo
            .expect_find()
            .withf(|id| *id == user_id)
            .times(1)
            .returning(move |_| Err(diesel::result::Error::NotFound));

        let result = mock_repo.find(&user_id).await;
        assert!(result.is_err());
    }

    #[actix_rt::test]
    async fn test_find_by_email() {
        let mut mock_repo = MockIUserRepository::new();
        let email_blind_index = vec![1, 2, 3, 4, 5];
        let test_user = User {
            id: Uuid::new_v4(),
            email_blind_index: email_blind_index.clone(),
            email_encrypted: vec![4, 5, 6],
            encrypted_password: "hashed_password".to_string(),
            confirmation_token_digest: Some("token_digest".to_string()),
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: None,
            encryption_key_version: 1,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        mock_repo
            .expect_find_by_email()
            .withf(|blind_index| blind_index == &email_blind_index)
            .times(1)
            .returning(move |blind_index| Ok(Some(User {
                id: Uuid::new_v4(),
                email_blind_index: blind_index.clone(),
                email_encrypted: vec![4, 5, 6],
                encrypted_password: "hashed_password".to_string(),
                confirmation_token_digest: Some("token_digest".to_string()),
                unconfirmed_email_blind_index: None,
                unconfirmed_email_encrypted: None,
                encryption_key_version: 1,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            })));

        let result = mock_repo.find_by_email(&email_blind_index).await;
        assert!(result.is_ok());
        let user_option = result.unwrap();
        assert!(user_option.is_some());
        let user = user_option.unwrap();
        assert_eq!(user.email_blind_index, email_blind_index);
    }

    #[actix_rt::test]
    async fn test_find_by_email_not_found() {
        let mut mock_repo = MockIUserRepository::new();
        let email_blind_index = vec![1, 2, 3, 4, 5];

        mock_repo
            .expect_find_by_email()
            .withf(|blind_index| blind_index == &email_blind_index)
            .times(1)
            .returning(move |_| Ok(None));

        let result = mock_repo.find_by_email(&email_blind_index).await;
        assert!(result.is_ok());
        let user_option = result.unwrap();
        assert!(user_option.is_none());
    }

    #[actix_rt::test]
    async fn test_confirm_email() {
        let mut mock_repo = MockIUserRepository::new();
        let token_digest = "token123";

        mock_repo
            .expect_confirm_email()
            .withf(|digest| digest == token_digest)
            .times(1)
            .returning(|digest| Ok(1));

        let result = mock_repo.confirm_email(token_digest).await;
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[actix_rt::test]
    async fn test_record_successful_login() {
        let mut mock_repo = MockIUserRepository::new();
        let user_id = Uuid::new_v4();
        let ip = Some(ipnet::IpNet::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 24).unwrap());

        mock_repo
            .expect_record_successful_login()
            .withf(|id, ip_addr| *id == user_id && *ip_addr == ip)
            .times(1)
            .returning(|_, _| Ok(1));

        let result = mock_repo.record_successful_login(&user_id, ip).await;
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[actix_rt::test]
    async fn test_get_user_roles() {
        let mut mock_repo = MockIUserRepository::new();
        let user_id = Uuid::new_v4();
        let roles = vec!["admin".to_string(), "user".to_string()];

        mock_repo
            .expect_get_user_roles()
            .withf(|id| *id == user_id)
            .times(1)
            .returning(move |id| Ok(vec!["admin".to_string(), "user".to_string()]));

        let result = mock_repo.get_user_roles(&user_id).await;
        assert!(result.is_ok());
        let user_roles = result.unwrap();
        assert_eq!(user_roles, roles);
    }
}