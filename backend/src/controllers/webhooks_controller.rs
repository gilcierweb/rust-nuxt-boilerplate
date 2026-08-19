use actix_web::{HttpRequest, HttpResponse, post, web};

use crate::repositories::AppContainer;

/// Handle Stripe webhook events.
///
/// Stripe sends signed webhook events to this endpoint. The
/// `StripeWebhookVerifier` middleware validates the `Stripe-Signature`
/// header and, if valid, sets the `stripe-event-type` header before
/// forwarding the request here.
///
/// # Idempotency
/// Each Stripe event carries a unique `id` (e.g. `evt_...`). The handler
/// logs it to the audit log for replay tracking. Implement deduplication
/// against this `id` before processing business logic.
///
/// # Business Logic
/// Currently logs all events to the audit log. Extend the `match` block
/// to process `checkout.session.completed`, `invoice.paid`, etc.
#[utoipa::path(
    post,
    path = "/api/v1/webhooks/stripe",
    tag = "webhooks",
    request_body(
        content = String,
        content_type = "application/json",
        description = "Raw Stripe event payload (signature verified by middleware)"
    ),
    responses(
        (status = 200, description = "Webhook accepted"),
        (status = 400, description = "Invalid payload (UTF-8 or JSON parse failure)")
    )
)]
#[post("/stripe")]
pub async fn stripe_webhook(
    req: HttpRequest,
    payload: web::Bytes,
    container: web::Data<AppContainer>,
) -> HttpResponse {
    // Parse the event type from the header (set by StripeWebhookVerifier)
    let event_type = req
        .headers()
        .get("stripe-event-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Parse the webhook payload
    let payload_str = match std::str::from_utf8(&payload) {
        Ok(s) => s,
        Err(_) => {
            tracing::warn!(
                event = "stripe_webhook.invalid_payload",
                "Failed to parse webhook payload as UTF-8"
            );
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": t!("webhooks.stripe.invalid_payload").into_owned()
            }));
        },
    };

    let payload_json: serde_json::Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                event = "stripe_webhook.invalid_json",
                error = %e,
                "Failed to parse webhook payload as JSON"
            );
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": t!("webhooks.stripe.invalid_json").into_owned()
            }));
        },
    };

    // Extract event ID for idempotency
    let event_id = payload_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Log the webhook event to audit log
    let audit_log = crate::models::audit_log::NewAuditLog {
        actor_user_id: None,
        actor_role_snapshot: Some("system:stripe".to_string()),
        action: format!("webhook.{}", event_type),
        resource_type: "payment".to_string(),
        resource_id: None,
        ip_address: None,
        user_agent: req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        request_id: None,
        changes: serde_json::json!({
            "event_id": event_id,
            "event_type": event_type,
        }),
        metadata: payload_json.clone(),
        prev_hash: None,
        hash: String::new(),
    };

    if let Err(e) = container.domain_audit_logs.create(&audit_log).await {
        tracing::error!(
            event = "stripe_webhook.audit_log_failed",
            error = %e,
            "Failed to create audit log for webhook event"
        );
    }

    // Process event based on type
    match event_type {
        "checkout.session.completed" => {
            tracing::info!(
                event = "stripe_webhook.session_completed",
                event_id = %event_id,
                "Stripe checkout session completed"
            );
            // TODO: Implement actual business logic
            // - Extract customer email from payload
            // - Find or create user
            // - Create subscription record
            // - Send welcome email
        },
        "invoice.paid" => {
            tracing::info!(
                event = "stripe_webhook.invoice_paid",
                event_id = %event_id,
                "Stripe invoice paid"
            );
            // TODO: Implement actual business logic
            // - Update subscription status to active
            // - Extend user access period
        },
        "invoice.payment_failed" => {
            tracing::warn!(
                event = "stripe_webhook.payment_failed",
                event_id = %event_id,
                "Stripe payment failed"
            );
            // TODO: Implement actual business logic
            // - Mark subscription as past_due
            // - Send payment failure notification
            // - Retry payment or suspend access
        },
        "customer.subscription.updated" => {
            tracing::info!(
                event = "stripe_webhook.subscription_updated",
                event_id = %event_id,
                "Stripe subscription updated"
            );
            // TODO: Implement actual business logic
            // - Update subscription plan/period
            // - Sync features/limits
        },
        "customer.subscription.deleted" => {
            tracing::info!(
                event = "stripe_webhook.subscription_deleted",
                event_id = %event_id,
                "Stripe subscription cancelled"
            );
            // TODO: Implement actual business logic
            // - Mark subscription as cancelled
            // - Revoke access
            // - Send cancellation email
        },
        _ => {
            tracing::info!(
                event = "stripe_webhook.unhandled_event",
                event_id = %event_id,
                event_type = %event_type,
                "Unhandled Stripe webhook event type"
            );
        },
    }

    HttpResponse::Ok().json(serde_json::json!({
        "received": true,
        "event_type": event_type
    }))
}

/// Handle Pix webhook events (Brazilian instant payment system)
///
/// Pix webhooks may have different verification mechanisms depending on
/// the payment provider (e.g., Mercado Pago, PagSeguro, Gerencianet).
///
/// # Business Logic
/// Currently logs all events for audit purposes. Extend the implementation
/// to process actual payment confirmations.
#[utoipa::path(
    post,
    path = "/api/v1/webhooks/pix",
    tag = "webhooks",
    request_body(
        content = String,
        content_type = "application/json",
        description = "Raw Pix event payload"
    ),
    responses(
        (status = 200, description = "Webhook accepted"),
        (status = 400, description = "Invalid payload (UTF-8 or JSON parse failure)")
    )
)]
#[post("/pix")]
pub async fn pix_webhook(
    req: HttpRequest,
    payload: web::Bytes,
    container: web::Data<AppContainer>,
) -> HttpResponse {
    // Parse the webhook payload
    let payload_str = match std::str::from_utf8(&payload) {
        Ok(s) => s,
        Err(_) => {
            tracing::warn!(
                event = "pix_webhook.invalid_payload",
                "Failed to parse Pix webhook payload as UTF-8"
            );
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": t!("webhooks.pix.invalid_payload").into_owned()
            }));
        },
    };

    let payload_json: serde_json::Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                event = "pix_webhook.invalid_json",
                error = %e,
                "Failed to parse Pix webhook payload as JSON"
            );
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": t!("webhooks.pix.invalid_json").into_owned()
            }));
        },
    };

    // Extract transaction ID for idempotency
    let transaction_id = payload_json
        .get("txid")
        .or_else(|| payload_json.get("transaction_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Determine event type from payload
    let event_type = payload_json
        .get("event")
        .or_else(|| payload_json.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("pix.received");

    // Log the webhook event to audit log
    let audit_log = crate::models::audit_log::NewAuditLog {
        actor_user_id: None,
        actor_role_snapshot: Some("system:pix".to_string()),
        action: format!("webhook.{}", event_type),
        resource_type: "payment".to_string(),
        resource_id: None,
        ip_address: None,
        user_agent: req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        request_id: None,
        changes: serde_json::json!({
            "transaction_id": transaction_id,
            "event_type": event_type,
        }),
        metadata: payload_json.clone(),
        prev_hash: None,
        hash: String::new(),
    };

    if let Err(e) = container.domain_audit_logs.create(&audit_log).await {
        tracing::error!(
            event = "pix_webhook.audit_log_failed",
            error = %e,
            "Failed to create audit log for Pix webhook event"
        );
    }

    // Process Pix payment event
    match event_type {
        "pix.received" | "payment.received" => {
            tracing::info!(
                event = "pix_webhook.payment_received",
                transaction_id = %transaction_id,
                "Pix payment received"
            );
            // TODO: Implement actual business logic
            // - Extract amount and payer info
            // - Find associated order/subscription
            // - Update payment status
            // - Grant access
        },
        "pix.refund" | "payment.refunded" => {
            tracing::info!(
                event = "pix_webhook.payment_refunded",
                transaction_id = %transaction_id,
                "Pix payment refunded"
            );
            // TODO: Implement actual business logic
            // - Process refund
            // - Revoke access if needed
        },
        _ => {
            tracing::info!(
                event = "pix_webhook.unhandled_event",
                transaction_id = %transaction_id,
                event_type = %event_type,
                "Unhandled Pix webhook event type"
            );
        },
    }

    HttpResponse::Ok().json(serde_json::json!({
        "received": true,
        "transaction_id": transaction_id
    }))
}

pub fn stripe_config(cfg: &mut web::ServiceConfig) {
    cfg.service(stripe_webhook);
}

pub fn pix_config(cfg: &mut web::ServiceConfig) {
    cfg.service(pix_webhook);
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(stripe_webhook).service(pix_webhook);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/webhooks").configure(config));
}
