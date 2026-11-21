use crate::repositories::access_token_blacklist::is_token_revoked_by_epoch;

/// What the shared revocation rule decided for a verified token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationOutcome {
    /// The token is live: neither the blacklist nor the epoch rejects it.
    Valid,
    /// The token was revoked: the caller must refuse it.
    Revoked,
    /// The revocation status could not be determined (the store is
    /// unavailable). The caller must fail closed.
    Indeterminate,
}

/// One answer set of the two revocation sources.
pub struct RevocationAnswers {
    pub blacklisted: Result<bool, RevocationError>,
    pub epoch: Result<Option<u64>, RevocationError>,
}

pub type RevocationError = Box<dyn std::error::Error + Send + Sync>;

impl RevocationAnswers {
    /// The answer set of a context where the store does not exist at all:
    /// the status is indeterminable, so the caller fails closed.
    pub fn unavailable() -> Self {
        let reason: RevocationError = "revocation store unavailable".into();
        Self {
            blacklisted: Err(reason),
            epoch: Err("revocation store unavailable".into()),
        }
    }

    /// Which answer made the status indeterminable, so a caller can log the
    /// exact reason before it fails closed. The event names are the ones the
    /// HTTP path has always emitted.
    pub fn indeterminacy_cause(&self) -> Option<(&'static str, &RevocationError)> {
        if let Err(error) = &self.blacklisted {
            Some(("auth.blacklist_unavailable", error))
        } else if let Err(error) = &self.epoch {
            Some(("auth.epoch_unavailable", error))
        } else {
            None
        }
    }
}

/// The shared revocation rule: the per-token blacklist first, then the
/// per-user credential epoch. An answer that cannot be obtained never
/// reads as "not revoked".
pub fn decide_revocation(answers: &RevocationAnswers, issued_at: usize) -> RevocationOutcome {
    match &answers.blacklisted {
        Err(_) => return RevocationOutcome::Indeterminate,
        Ok(true) => return RevocationOutcome::Revoked,
        Ok(false) => {},
    }

    match &answers.epoch {
        Err(_) => RevocationOutcome::Indeterminate,
        Ok(Some(epoch)) if is_token_revoked_by_epoch(issued_at, Some(*epoch)) => {
            RevocationOutcome::Revoked
        },
        Ok(_) => RevocationOutcome::Valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::Claims;

    fn ok_answers(blacklisted: bool, epoch: Option<u64>) -> RevocationAnswers {
        RevocationAnswers {
            blacklisted: Ok(blacklisted),
            epoch: Ok(epoch),
        }
    }

    #[test]
    fn a_blacklisted_token_is_revoked_before_the_epoch_is_ever_read() {
        let answers = RevocationAnswers {
            blacklisted: Ok(true),
            // The epoch answer would accept the token: it must never matter.
            epoch: Ok(None),
        };
        assert_eq!(
            decide_revocation(&answers, 1_000),
            RevocationOutcome::Revoked
        );
    }

    #[test]
    fn a_token_issued_before_the_credential_change_is_revoked() {
        let answers = ok_answers(false, Some(2_000));
        assert_eq!(
            decide_revocation(&answers, 1_000),
            RevocationOutcome::Revoked
        );
    }

    #[test]
    fn a_token_issued_after_the_credential_change_is_valid() {
        let answers = ok_answers(false, Some(1_000));
        assert_eq!(decide_revocation(&answers, 2_000), RevocationOutcome::Valid);
        assert_eq!(decide_revocation(&answers, 1_000), RevocationOutcome::Valid);
    }

    #[test]
    fn a_subject_without_a_credential_change_is_valid() {
        let answers = ok_answers(false, None);
        assert_eq!(decide_revocation(&answers, 1_000), RevocationOutcome::Valid);
    }

    #[test]
    fn an_unavailable_blacklist_fails_closed() {
        let answers = RevocationAnswers {
            blacklisted: Err("redis down".into()),
            epoch: Ok(None),
        };
        assert_eq!(
            decide_revocation(&answers, 1_000),
            RevocationOutcome::Indeterminate
        );
    }

    #[test]
    fn an_unavailable_epoch_fails_closed_even_when_the_token_is_not_blacklisted() {
        let answers = RevocationAnswers {
            blacklisted: Ok(false),
            epoch: Err("redis down".into()),
        };
        assert_eq!(
            decide_revocation(&answers, 1_000),
            RevocationOutcome::Indeterminate
        );
    }

    #[test]
    fn the_rule_never_trusts_the_claims_it_is_given() {
        // The signature documents that the decision reads only the issued-at
        // timestamp of the claims, never their identity fields.
        let claims_issued_at = 500_usize;
        let answers = ok_answers(false, Some(600));
        assert_eq!(
            decide_revocation(&answers, claims_issued_at),
            RevocationOutcome::Revoked
        );
    }

    #[test]
    fn a_context_without_the_store_fails_closed_and_names_the_reason() {
        let answers = RevocationAnswers::unavailable();
        assert_eq!(
            decide_revocation(&answers, 1_000),
            RevocationOutcome::Indeterminate
        );
        let (event, _) = answers.indeterminacy_cause().expect("a cause is named");
        assert_eq!(event, "auth.blacklist_unavailable");
    }

    #[test]
    fn the_indeterminacy_cause_names_the_epoch_answer_when_the_blacklist_answered() {
        let answers = RevocationAnswers {
            blacklisted: Ok(false),
            epoch: Err("redis down".into()),
        };
        let (event, _) = answers.indeterminacy_cause().expect("a cause is named");
        assert_eq!(event, "auth.epoch_unavailable");
    }

    #[test]
    fn a_live_token_names_no_cause() {
        assert!(ok_answers(false, None).indeterminacy_cause().is_none());
    }

    #[test]
    fn the_claims_type_is_the_one_the_verifier_produces() {
        let claims = Claims {
            sub: uuid::Uuid::nil(),
            profile_id: uuid::Uuid::nil(),
            role: 0,
            token_use: "access".to_string(),
            exp: 0,
            iat: 0,
            nbf: 0,
            aud: "test".to_string(),
            jti: None,
        };
        assert_eq!(claims.iat, 0);
    }
}
