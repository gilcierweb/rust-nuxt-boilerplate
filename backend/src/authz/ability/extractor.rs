//! Request extractor for the ability of the authenticated subject.
//!
//! The `JwtAuth` middleware loads the grants of the subject and attaches the
//! resulting ability to the request. Handlers read it through this extractor;
//! a request without the extension is unauthenticated.
//!
//! The type is the facade of the CanCanCan `current_ability`: the handlers
//! ask it for a permission (`authorize`, `authorize_subject`) or for a check
//! (`can_check`, `can_check_subject`) without touching the raw ability.

use std::future::{Ready, ready};

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use uuid::Uuid;

use super::authorizer::{authorize, authorize_subject};
use super::engine::Ability;
use super::types::{AbilityAction, AbilityResource, Permissions};
use crate::errors::{AppError, AppResult};

/// The ability of the authenticated subject (the CanCanCan `current_ability`).
#[derive(Debug, Clone)]
pub struct CurrentAbility {
    ability: Ability,
}

impl CurrentAbility {
    /// Wraps the ability of the subject.
    pub fn new(ability: Ability) -> Self {
        Self { ability }
    }

    /// The identifier of the authenticated subject.
    pub fn subject(&self) -> Uuid {
        self.ability.subject()
    }

    /// Checks a class level permission (the CanCanCan `authorize!`).
    pub fn authorize(&self, resource: AbilityResource, action: AbilityAction) -> AppResult<()> {
        authorize(&self.ability, resource, action)
    }

    /// Checks a permission on one subject (the CanCanCan `authorize!` with an
    /// instance).
    pub fn authorize_subject(
        &self,
        resource: AbilityResource,
        action: AbilityAction,
        subject_id: Uuid,
    ) -> AppResult<()> {
        authorize_subject(&self.ability, resource, action, subject_id)
    }

    /// The CanCanCan `can?` on the class level.
    pub fn can_check(&self, action: AbilityAction, resource: AbilityResource) -> bool {
        self.ability.can_check(action, resource)
    }

    /// The CanCanCan `cannot?` on the class level.
    pub fn cannot_check(&self, action: AbilityAction, resource: AbilityResource) -> bool {
        self.ability.cannot_check(action, resource)
    }

    /// The CanCanCan `can?` on one subject.
    pub fn can_check_subject(
        &self,
        action: AbilityAction,
        resource: AbilityResource,
        subject_id: Uuid,
    ) -> bool {
        self.ability.can_check_subject(action, resource, subject_id)
    }

    /// The CanCanCan `cannot?` on one subject.
    pub fn cannot_check_subject(
        &self,
        action: AbilityAction,
        resource: AbilityResource,
        subject_id: Uuid,
    ) -> bool {
        self.ability
            .cannot_check_subject(action, resource, subject_id)
    }

    /// The effective permissions of the subject (the CanCanCan `permissions`).
    pub fn permissions(&self) -> Permissions {
        self.ability.permissions()
    }

    /// Checks whether the subject holds a class level allow over at least one
    /// resource of the domain.
    ///
    /// The scoped rules never open a domain: they only answer the instance
    /// checks of the handlers.
    pub fn can_check_domain(&self, domain: &[AbilityResource]) -> bool {
        self.ability.has_class_allow_in_domain(domain)
    }
}

impl FromRequest for CurrentAbility {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<CurrentAbility>()
                .cloned()
                .ok_or_else(|| {
                    AppError::Unauthorized(t!("middleware.not_authenticated").into_owned())
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use actix_web::dev::Payload;
    use actix_web::test::TestRequest;
    use actix_web::{FromRequest, HttpMessage};
    use uuid::Uuid;

    use super::CurrentAbility;
    use crate::authz::ability::types::DOMAIN_RESOURCES;
    use crate::authz::ability::{Ability, AbilityAction, AbilityResource};

    #[actix_web::test]
    async fn extractor_is_unauthorized_without_the_extension() {
        let req = TestRequest::get().to_http_request();

        let result = CurrentAbility::from_request(&req, &mut Payload::None).await;

        assert!(
            result.is_err(),
            "a request without the ability is unauthorized"
        );
    }

    #[actix_web::test]
    async fn extractor_returns_the_attached_ability() {
        let req = TestRequest::get().to_http_request();
        req.extensions_mut()
            .insert(CurrentAbility::new(Ability::new(Uuid::new_v4())));

        let result = CurrentAbility::from_request(&req, &mut Payload::None).await;

        assert!(result.is_ok(), "the attached ability must be readable");
    }

    #[test]
    fn the_facade_answers_the_cancancan_questions() {
        let mut ability = Ability::new(Uuid::new_v4());
        ability.can(AbilityAction::Read, AbilityResource::Users);
        let current = CurrentAbility::new(ability);

        assert!(current.can_check(AbilityAction::Read, AbilityResource::Users));
        assert!(current.cannot_check(AbilityAction::Delete, AbilityResource::Users));
        assert!(
            current
                .authorize(AbilityResource::Users, AbilityAction::Read)
                .is_ok()
        );
        assert!(
            current
                .authorize(AbilityResource::Users, AbilityAction::Delete)
                .is_err()
        );
        assert!(current.permissions().can.contains("users:read"));
    }

    #[test]
    fn a_class_level_grant_opens_the_domain() {
        let mut ability = Ability::new(Uuid::new_v4());
        ability.can(AbilityAction::Read, AbilityResource::Users);
        let current = CurrentAbility::new(ability);

        assert!(current.can_check_domain(&DOMAIN_RESOURCES));
    }

    #[test]
    fn a_scoped_grant_never_opens_the_domain() {
        let mut ability = Ability::new(Uuid::new_v4());
        ability.can_self(AbilityAction::Read, AbilityResource::Profiles);
        let current = CurrentAbility::new(ability);

        assert!(
            !current.can_check_domain(&DOMAIN_RESOURCES),
            "a self scoped grant must not open the admin domain"
        );
    }
}
