use opentelemetry::Context;
use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId};
use reqwest::RequestBuilder;

/// Inject the current OpenTelemetry trace context into a reqwest request
/// as a W3C `traceparent` header (Level 1).
///
/// When `tracing-opentelemetry` is active, the current `tracing` span corresponds to
/// an OTel span. This function extracts the span context and serializes it as the
/// `traceparent` header, enabling downstream services (Resend, Stripe, Bunny.net, etc.)
/// to receive the trace context for end-to-end distributed tracing.
///
/// If no active span exists (e.g. called outside a request handler), the header is
/// silently omitted — downstream calls still work, they just won't be correlated.
pub fn inject_traceparent(builder: RequestBuilder) -> RequestBuilder {
    let cx = Context::current();
    let span_ctx = cx.span().span_context().clone();

    if !span_ctx.is_valid() {
        return builder;
    }

    let header_value = format_traceparent(span_ctx);
    builder.header("traceparent", header_value)
}

/// Inject trace context into an existing reqwest `HeaderMap`.
pub fn inject_traceparent_into_headers(headers: &mut reqwest::header::HeaderMap) {
    let cx = Context::current();
    let span_ctx = cx.span().span_context().clone();

    if !span_ctx.is_valid() {
        return;
    }

    let header_value = format_traceparent(span_ctx);
    if let Ok(val) = reqwest::header::HeaderValue::from_str(&header_value) {
        headers.insert("traceparent", val);
    }
}

/// Parse an inbound `traceparent` header value into an OTel `Context`.
///
/// The returned context carries the remote span context so it can be used as a
/// parent when creating child spans for downstream operations (DB queries, Redis,
/// outbound HTTP).
///
/// Format: `{version}-{trace-id}-{parent-span-id}-{trace-flags}`
/// Example: `00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01`
pub fn extract_traceparent_from_request(
    headers: &actix_web::http::header::HeaderMap,
) -> Option<Context> {
    let raw = headers.get("traceparent")?.to_str().ok()?;
    parse_traceparent(raw)
}

/// Parse a W3C `traceparent` string into an OTel `Context` with a remote span.
pub fn parse_traceparent(raw: &str) -> Option<Context> {
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() != 4 {
        return None;
    }

    let trace_id = TraceId::from_hex(parts[1]).ok()?;
    let span_id = SpanId::from_hex(parts[2]).ok()?;
    let flags = u8::from_str_radix(parts[3], 16).ok()?;
    let trace_flags = TraceFlags::new(flags);

    let span_ctx = SpanContext::new(trace_id, span_id, trace_flags, true, Default::default());
    if !span_ctx.is_valid() {
        return None;
    }

    Some(Context::current().with_remote_span_context(span_ctx))
}

fn format_traceparent(ctx: SpanContext) -> String {
    let trace_id = format!("{trace_id}", trace_id = ctx.trace_id());
    let span_id = format!("{span_id}", span_id = ctx.span_id());
    let flags = if ctx.is_sampled() { "01" } else { "00" };
    format!("00-{trace_id}-{span_id}-{flags}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_traceparent_zero_padded() {
        let trace_id = TraceId::from(0x1u128);
        let span_id = SpanId::from(0x2u64);
        let ctx = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            Default::default(),
        );
        assert_eq!(
            format_traceparent(ctx),
            "00-00000000000000000000000000000001-0000000000000002-01"
        );
    }

    #[test]
    fn format_traceparent_not_sampled() {
        let trace_id = TraceId::from(0xABu128);
        let span_id = SpanId::from(0xCDu64);
        let ctx = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::NOT_SAMPLED,
            false,
            Default::default(),
        );
        assert_eq!(
            format_traceparent(ctx),
            "00-000000000000000000000000000000ab-00000000000000cd-00"
        );
    }

    #[test]
    fn parse_traceparent_valid() {
        let ctx = parse_traceparent("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01");
        assert!(ctx.is_some());
    }

    #[test]
    fn parse_traceparent_invalid_format() {
        assert!(parse_traceparent("invalid").is_none());
        assert!(parse_traceparent("00-abc-def").is_none());
    }

    #[test]
    fn parse_traceparent_zero_trace_id() {
        let ctx = parse_traceparent("00-00000000000000000000000000000000-b7ad6b7169203331-01");
        assert!(ctx.is_none());
    }

    #[test]
    fn parse_traceparent_zero_span_id() {
        let ctx = parse_traceparent("00-0af7651916cd43dd8448eb211c80319c-0000000000000000-01");
        assert!(ctx.is_none());
    }

    #[test]
    fn inject_traceparent_skips_when_no_active_span() {
        let client = reqwest::Client::new();
        let builder = client.get("https://example.com");
        let patched = inject_traceparent(builder);
        let req = patched.build().unwrap();
        assert!(req.headers().get("traceparent").is_none());
    }

    #[test]
    fn inject_traceparent_into_headers_skips_when_no_active_span() {
        let mut headers = reqwest::header::HeaderMap::new();
        inject_traceparent_into_headers(&mut headers);
        assert!(headers.get("traceparent").is_none());
    }

    #[test]
    fn extract_traceparent_from_request_returns_none_when_missing() {
        let headers = actix_web::http::header::HeaderMap::new();
        assert!(extract_traceparent_from_request(&headers).is_none());
    }

    #[test]
    fn extract_traceparent_from_request_parses_valid_header() {
        let mut headers = actix_web::http::header::HeaderMap::new();
        headers.insert(
            actix_web::http::header::HeaderName::from_static("traceparent"),
            actix_web::http::header::HeaderValue::from_static(
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            ),
        );
        let ctx = extract_traceparent_from_request(&headers);
        assert!(ctx.is_some());
    }

    #[test]
    fn roundtrip_traceparent() {
        let trace_id = TraceId::from(0xDEADBEEF_CAFEBABE_12345678_9ABCDEF0u128);
        let span_id = SpanId::from(0x12345678_9ABCDEF0u64);
        let ctx = SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            false,
            Default::default(),
        );
        let formatted = format_traceparent(ctx);
        let parsed = parse_traceparent(&formatted).unwrap();
        let parsed_span_ctx = parsed.span().span_context().clone();
        assert_eq!(parsed_span_ctx.trace_id(), trace_id);
        assert_eq!(parsed_span_ctx.span_id(), span_id);
    }
}
