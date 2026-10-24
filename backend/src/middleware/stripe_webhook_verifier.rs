use std::rc::Rc;

use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::web::{self, Bytes};
use actix_web::{Error as ActixError, HttpResponse};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::AppState;
use crate::repositories::container::AppContainer;
use crate::utils::constant_time::constant_time_eq_hex;

pub struct StripeWebhookVerifier {
    enabled: bool,
}

impl Default for StripeWebhookVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl StripeWebhookVerifier {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl<S> Transform<S, ServiceRequest> for StripeWebhookVerifier
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = StripeWebhookVerifierMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(StripeWebhookVerifierMiddleware {
            service: Rc::new(service),
            enabled: self.enabled,
        }))
    }
}

pub struct StripeWebhookVerifierMiddleware<S> {
    service: Rc<S>,
    enabled: bool,
}

impl<S> Service<ServiceRequest> for StripeWebhookVerifierMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let enabled = self.enabled;

        Box::pin(async move {
            let path = req.uri().path();

            // Only verify Stripe webhook endpoints
            if enabled && path.starts_with("/api/v1/webhooks/stripe") {
                // Resolve the config the same way `RateLimiter` and `RequireApiKey`
                // do. A bare `req.app_data::<AppConfig>()` resolves to None in
                // production wiring (only `Data<AppState>` / `Data<AppContainer>`
                // are registered), which used to skip verification entirely.
                let config = req
                    .app_data::<web::Data<AppContainer>>()
                    .map(|container| (*container.config).clone())
                    .or_else(|| {
                        req.app_data::<web::Data<AppState>>()
                            .map(|state| (*state.config).clone())
                    });

                let Some(cfg) = config else {
                    tracing::error!(
                        event = "webhook.config_missing",
                        path = %path,
                        "AppConfig not found in app_data — refusing to process Stripe webhook"
                    );
                    return Ok(fail_closed(
                        req,
                        "WEBHOOK_CONFIG_UNAVAILABLE",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                };

                if cfg.stripe_webhook_secret.is_empty() {
                    tracing::error!(
                        event = "webhook.secret_not_configured",
                        path = %path,
                        "STRIPE_WEBHOOK_SECRET is empty — refusing to process unsigned Stripe webhook"
                    );
                    return Ok(fail_closed(
                        req,
                        "WEBHOOK_SECRET_NOT_CONFIGURED",
                        StatusCode::SERVICE_UNAVAILABLE,
                    ));
                }

                // Verify Stripe signature
                let stripe_signature = req
                    .headers()
                    .get("stripe-signature")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string());

                if stripe_signature.is_none() {
                    let response = HttpResponse::BadRequest()
                        .json(serde_json::json!({
                            "error": {
                                "code": "STRIPE_SIGNATURE_MISSING",
                                "message": t!("webhooks.stripe.missing_signature").into_owned()
                            }
                        }))
                        .map_into_boxed_body();

                    let (req, _) = req.into_parts();
                    return Ok(ServiceResponse::new(req, response));
                }

                // Read the request body for verification
                let (req, mut body) = req.into_parts();
                let mut body_bytes = Vec::new();
                use futures::StreamExt;
                while let Some(chunk) = body.next().await {
                    if let Ok(bytes) = chunk {
                        body_bytes.extend_from_slice(&bytes);
                    }
                }

                // Verify the signature
                let signature = stripe_signature.unwrap();
                let body_bytes = Bytes::from(body_bytes);
                let is_valid = verify_stripe_signature(
                    &body_bytes,
                    &signature,
                    &cfg.stripe_webhook_secret,
                );

                if !is_valid {
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": {
                                "code": "STRIPE_SIGNATURE_INVALID",
                                "message": t!("webhooks.stripe.invalid_signature").into_owned()
                            }
                        }))
                        .map_into_boxed_body();

                    return Ok(ServiceResponse::new(req, response));
                }

                // Reconstruct the request with the body
                let req = ServiceRequest::from_parts(req, body_bytes.into());
                return svc.call(req).await;
            }

            // Non-Stripe paths keep their own verifier; nothing to check here.
            svc.call(req).await
        })
    }
}

/// Build a response that rejects the webhook instead of letting it through.
///
/// Used whenever the signature cannot be verified for an operational reason
/// (config unavailable, secret not configured). An unverifiable webhook must
/// never be treated as a verified one.
fn fail_closed(
    req: ServiceRequest,
    code: &'static str,
    status: StatusCode,
) -> ServiceResponse<BoxBody> {
    let response = HttpResponse::build(status)
        .json(serde_json::json!({
            "error": {
                "code": code,
                "message": "Webhook signature could not be verified"
            }
        }))
        .map_into_boxed_body();

    let (http_req, _) = req.into_parts();
    ServiceResponse::new(http_req, response)
}

fn verify_stripe_signature(payload: &Bytes, signature: &str, secret: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    // Stripe signature format: t=timestamp,v1=signature,v0=signature
    let parts: Vec<&str> = signature.split(',').collect();

    let mut timestamp: Option<i64> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "t" => {
                    if let Ok(ts) = value.parse::<i64>() {
                        timestamp = Some(ts);
                    }
                },
                "v1" | "v0" => signatures.push(value),
                _ => {},
            }
        }
    }

    // Check if timestamp is within 5 minutes
    if let Some(ts) = timestamp {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if (now - ts).abs() > 300 {
            return false;
        }
    } else {
        return false;
    }

    // Verify signature
    let signed_payload = format!(
        "{}.{}",
        timestamp.unwrap_or(0),
        String::from_utf8_lossy(payload)
    );

    let Ok(mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };

    let mut mac = mac;
    mac.update(signed_payload.as_bytes());
    let expected = format!("{:x}", mac.finalize().into_bytes());

    signatures
        .iter()
        .any(|sig| constant_time_eq_hex(&expected, sig))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, HttpResponse, http::StatusCode, web};
    use serde_json::Value;
    use sha2::Sha256;

    use super::*;
    use crate::repositories::container::AppContainer;
    use crate::repositories::mocks::mock_container;

    // NOTE: `actix_web::test` is deliberately NOT imported by name here. It is
    // the `#[test]` attribute macro from actix-macros, and importing it would
    // shadow the built-in `#[test]` used by the sync tests below. Call the
    // helpers through the fully qualified `actix_web::test::` path instead.

    fn make_valid_signature(payload: &[u8], secret: &str, timestamp: i64) -> String {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed_payload.as_bytes());
        let sig = format!("{:x}", mac.finalize().into_bytes());
        format!("t={},v1={}", timestamp, sig)
    }

    fn now_secs() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Container carrying a webhook secret, so tests control the two
    /// configuration paths (secret set / secret empty).
    fn container_with_webhook_secret(secret: &str) -> AppContainer {
        let mut container = mock_container();
        let mut config = (*container.config).clone();
        config.stripe_webhook_secret = secret.to_string();
        container.config = Arc::new(config);
        container
    }

    /// Build the Stripe webhook scope and POST to it, returning the status and
    /// the error payload. `container = None` registers no config source at all.
    async fn post_status(
        container: Option<AppContainer>,
        signature: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut app = App::new();
        app = match container {
            Some(container) => app.app_data(web::Data::new(container)),
            None => app.app_data(web::Data::new("unrelated-config".to_string())),
        };
        let app = actix_web::test::init_service(
            app.service(
                web::scope("/api/v1/webhooks/stripe")
                    .wrap(StripeWebhookVerifier::new())
                    .route(
                        "",
                        web::post()
                            .to(|| async { HttpResponse::Ok().json(serde_json::json!({"handled": true})) }),
                    ),
            ),
        )
        .await;

        let payload = b"{\"id\":\"evt_regression\"}";
        let mut request = actix_web::test::TestRequest::post().uri("/api/v1/webhooks/stripe");
        if let Some(signature) = signature {
            request = request.insert_header(("stripe-signature", signature));
        }
        let response = actix_web::test::call_service(
            &app,
            request.set_payload(payload.to_vec()).to_request(),
        )
        .await;
        let status = response.status();
        let body: Value = actix_web::test::read_body_json(response).await;
        (status, body)
    }

    #[actix_web::test]
    async fn verifies_signature_resolved_from_the_container() {
        // Regression: the verifier used to read `req.app_data::<AppConfig>()`,
        // which is never registered in production, so the check was skipped and
        // every unsigned webhook reached the handler.
        let container = container_with_webhook_secret("whsec_regression");
        let signature =
            make_valid_signature(b"{\"id\":\"evt_regression\"}", "whsec_regression", now_secs());

        let (status, body) = post_status(Some(container), Some(&signature)).await;
        assert_eq!(status, StatusCode::OK, "a valid signature must pass verification");
        assert_eq!(body["handled"], true);
    }

    #[actix_web::test]
    async fn rejects_missing_signature() {
        // Before the fix this request passed straight through to the handler.
        let container = container_with_webhook_secret("whsec_regression");

        let (status, body) = post_status(Some(container), None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "STRIPE_SIGNATURE_MISSING");
    }

    #[actix_web::test]
    async fn rejects_invalid_signature() {
        let container = container_with_webhook_secret("whsec_regression");
        let signature =
            make_valid_signature(b"{\"id\":\"evt_regression\"}", "whsec_wrong", now_secs());

        let (status, body) = post_status(Some(container), Some(&signature)).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"]["code"], "STRIPE_SIGNATURE_INVALID");
    }

    #[actix_web::test]
    async fn fails_closed_when_secret_is_not_configured() {
        // An empty secret must never be treated as "verification disabled".
        let container = container_with_webhook_secret("");

        let (status, body) = post_status(Some(container), None).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], "WEBHOOK_SECRET_NOT_CONFIGURED");
    }

    #[actix_web::test]
    async fn fails_closed_when_no_config_source_is_registered() {
        let (status, body) = post_status(None, None).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "WEBHOOK_CONFIG_UNAVAILABLE");
    }

    #[test]
    fn test_signature_format_parsing() {
        let signature = "t=1234567890,v1=abc123,v0=def456";
        let parts: Vec<&str> = signature.split(',').collect();

        let mut timestamp: Option<i64> = None;
        let mut signatures: Vec<&str> = Vec::new();

        for part in parts {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "t" => {
                        if let Ok(ts) = value.parse::<i64>() {
                            timestamp = Some(ts);
                        }
                    },
                    "v1" | "v0" => signatures.push(value),
                    _ => {},
                }
            }
        }

        assert_eq!(timestamp, Some(1234567890));
        assert_eq!(signatures, vec!["abc123", "def456"]);
    }

    #[test]
    fn verify_stripe_signature_valid() {
        let payload = b"{\"id\":\"evt_123\"}";
        let secret = "whsec_test_secret";
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let sig = make_valid_signature(payload, secret, now);

        assert!(verify_stripe_signature(
            &Bytes::from_static(payload),
            &sig,
            secret
        ));
    }

    #[test]
    fn verify_stripe_signature_rejects_expired_timestamp() {
        let payload = b"test";
        let secret = "whsec_test";
        let old_timestamp = 1000000; // way in the past
        let sig = make_valid_signature(payload, secret, old_timestamp);

        assert!(!verify_stripe_signature(
            &Bytes::from_static(payload),
            &sig,
            secret
        ));
    }

    #[test]
    fn verify_stripe_signature_rejects_wrong_secret() {
        let payload = b"test";
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let sig = make_valid_signature(payload, "wrong_secret", now);

        assert!(!verify_stripe_signature(
            &Bytes::from_static(payload),
            &sig,
            "correct_secret"
        ));
    }

    #[test]
    fn verify_stripe_signature_rejects_wrong_payload() {
        let payload = b"original";
        let secret = "whsec_test";
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let sig = make_valid_signature(payload, secret, now);

        assert!(!verify_stripe_signature(
            &Bytes::from_static(b"tampered"),
            &sig,
            secret
        ));
    }

    #[test]
    fn verify_stripe_signature_rejects_missing_timestamp() {
        let sig = "v1=abc123";
        assert!(!verify_stripe_signature(
            &Bytes::from_static(b"test"),
            sig,
            "secret"
        ));
    }

    #[test]
    fn verify_stripe_signature_rejects_empty_signatures() {
        let sig = "t=1234567890";
        assert!(!verify_stripe_signature(
            &Bytes::from_static(b"test"),
            sig,
            "secret"
        ));
    }
}
