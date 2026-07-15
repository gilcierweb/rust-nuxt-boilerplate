use actix_web::{
    Error as ActixError, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::Bytes,
};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

use crate::config::AppConfig;

pub struct StripeWebhookVerifier {
    enabled: bool,
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
                // Get Stripe webhook secret from config
                if let Some(cfg) = req.app_data::<AppConfig>().cloned()
                    && !cfg.stripe_webhook_secret.is_empty()
                {
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
                                    "message": "Missing Stripe signature header"
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
                                    "message": "Invalid Stripe webhook signature"
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

            // For non-Stripe webhooks or if verification is disabled, pass through
            svc.call(req).await
        })
    }
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
                }
                "v1" | "v0" => signatures.push(value),
                _ => {}
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
    let signed_payload = format!("{}.{}", timestamp.unwrap_or(0), String::from_utf8_lossy(payload));
    
    let Ok(mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };

    let mut mac = mac;
    mac.update(signed_payload.as_bytes());
    let expected = format!("{:x}", mac.finalize().into_bytes());

    signatures.iter().any(|sig| {
        sig.len() == expected.len() && sig.chars().zip(expected.chars()).all(|(a, b)| a == b)
    })
}

#[cfg(test)]
mod tests {
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
                    }
                    "v1" | "v0" => signatures.push(value),
                    _ => {}
                }
            }
        }

        assert_eq!(timestamp, Some(1234567890));
        assert_eq!(signatures, vec!["abc123", "def456"]);
    }
}