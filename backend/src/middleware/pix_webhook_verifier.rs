use std::rc::Rc;

use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web::Bytes;
use actix_web::{Error as ActixError, HttpResponse};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::config::AppConfig;

pub struct PixWebhookVerifier {
    enabled: bool,
}

impl Default for PixWebhookVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl PixWebhookVerifier {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl<S> Transform<S, ServiceRequest> for PixWebhookVerifier
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = PixWebhookVerifierMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PixWebhookVerifierMiddleware {
            service: Rc::new(service),
            enabled: self.enabled,
        }))
    }
}

pub struct PixWebhookVerifierMiddleware<S> {
    service: Rc<S>,
    enabled: bool,
}

impl<S> Service<ServiceRequest> for PixWebhookVerifierMiddleware<S>
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

            // Only verify Pix webhook endpoints
            if enabled && path.starts_with("/api/v1/webhooks/pix") {
                // Get Pix webhook secret from config
                if let Some(cfg) = req.app_data::<AppConfig>().cloned()
                    && !cfg.pix_webhook_secret.is_empty()
                {
                    // Verify Pix signature
                    let pix_signature = req
                        .headers()
                        .get("x-pix-signature")
                        .or_else(|| req.headers().get("x-signature"))
                        .and_then(|h| h.to_str().ok())
                        .map(|s| s.to_string());

                    if pix_signature.is_none() {
                        let response = HttpResponse::BadRequest()
                            .json(serde_json::json!({
                                "error": {
                                    "code": "PIX_SIGNATURE_MISSING",
                                    "message": t!("webhooks.pix.missing_signature").into_owned()
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
                    let signature = pix_signature.unwrap();
                    let body_bytes = Bytes::from(body_bytes);
                    let is_valid =
                        verify_pix_signature(&body_bytes, &signature, &cfg.pix_webhook_secret);

                    if !is_valid {
                        let response = HttpResponse::Forbidden()
                            .json(serde_json::json!({
                                "error": {
                                    "code": "PIX_SIGNATURE_INVALID",
                                    "message": t!("webhooks.pix.invalid_signature").into_owned()
                                }
                            }))
                            .map_into_boxed_body();

                        return Ok(ServiceResponse::new(req, response));
                    }

                    // Reconstruct the request with the body
                    let req = ServiceRequest::from_parts(req, body_bytes.into());
                    return svc.call(req).await;
                }
            }

            // For non-Pix webhooks or if verification is disabled, pass through
            svc.call(req).await
        })
    }
}

fn verify_pix_signature(payload: &Bytes, signature: &str, secret: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    // Pix signature is typically HMAC-SHA256 of the raw payload
    // Format: sha256=<hex_signature>
    let expected_signature = signature.strip_prefix("sha256=").unwrap_or(signature);

    let Ok(mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };

    let mut mac = mac;
    mac.update(payload);
    let computed = format!("{:x}", mac.finalize().into_bytes());

    // Constant-time comparison
    computed.len() == expected_signature.len()
        && computed
            .chars()
            .zip(expected_signature.chars())
            .all(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use sha2::Sha256;

    use super::*;

    fn make_valid_signature(payload: &[u8], secret: &str) -> String {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let sig = format!("{:x}", mac.finalize().into_bytes());
        format!("sha256={}", sig)
    }

    #[test]
    fn verify_pix_signature_valid() {
        let payload = b"{\"txid\":\"test123\"}";
        let secret = "pix_webhook_secret";
        let sig = make_valid_signature(payload, secret);

        assert!(verify_pix_signature(
            &Bytes::from_static(payload),
            &sig,
            secret
        ));
    }

    #[test]
    fn verify_pix_signature_without_prefix() {
        let payload = b"{\"txid\":\"test123\"}";
        let secret = "pix_webhook_secret";
        let sig = make_valid_signature(payload, secret);
        let without_prefix = sig.strip_prefix("sha256=").unwrap();

        assert!(verify_pix_signature(
            &Bytes::from_static(payload),
            without_prefix,
            secret
        ));
    }

    #[test]
    fn verify_pix_signature_rejects_wrong_secret() {
        let payload = b"test";
        let sig = make_valid_signature(payload, "wrong_secret");

        assert!(!verify_pix_signature(
            &Bytes::from_static(payload),
            &sig,
            "correct_secret"
        ));
    }

    #[test]
    fn verify_pix_signature_rejects_wrong_payload() {
        let payload = b"original";
        let secret = "pix_webhook_secret";
        let sig = make_valid_signature(payload, secret);

        assert!(!verify_pix_signature(
            &Bytes::from_static(b"tampered"),
            &sig,
            secret
        ));
    }
}
