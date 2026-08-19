//! Locale resolution middleware.
//!
//! Inspects every incoming request, resolves the locale to use for that
//! request (via `X-Locale` header, then `Accept-Language`, then default),
//! and stores the result in `req.extensions_mut()` as a `String`.
//!
//! Handlers use `t!(key, locale = locale_from_request(req))` for compile-time
//! translations with runtime locale selection via rust_i18n.

use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;

/// Default fallback locale.
pub const DEFAULT_LOCALE: &str = "pt-BR";

/// Supported locales (must match Cargo.toml package.metadata.i18n.available-locales).
pub const SUPPORTED_LOCALES: &[&str] = &["pt-BR", "en", "es"];

/// Per-request locale stored in the request extensions.
#[derive(Debug, Clone)]
pub struct RequestLocale(pub String);

impl RequestLocale {
    /// Extract the locale from the current request, or return the default.
    pub fn from_request(req: &actix_web::HttpRequest) -> String {
        req.extensions()
            .get::<RequestLocale>()
            .map(|rl| rl.0.clone())
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string())
    }
}

pub struct LocaleMiddleware;

impl Default for LocaleMiddleware {
    fn default() -> Self {
        Self
    }
}

impl LocaleMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for LocaleMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LocaleMiddlewareInner<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LocaleMiddlewareInner {
            service: Rc::new(service),
        }))
    }
}

pub struct LocaleMiddlewareInner<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LocaleMiddlewareInner<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        // Extract the necessary data from req before the async block
        let headers = req.headers().clone();

        Box::pin(async move {
            // Reconstruct a minimal request for locale resolution
            let locale = {
                // 1. X-Locale header (explicit override)
                if let Some(value) = headers.get("x-locale").and_then(|h| h.to_str().ok()) {
                    let candidate = value.trim();
                    if SUPPORTED_LOCALES.contains(&candidate) {
                        candidate.to_string()
                    } else {
                        DEFAULT_LOCALE.to_string()
                    }
                }
                // 2. Accept-Language header (RFC 7231 negotiation)
                else if let Some(value) = headers
                    .get(actix_web::http::header::ACCEPT_LANGUAGE)
                    .and_then(|h| h.to_str().ok())
                {
                    let mut resolved = DEFAULT_LOCALE.to_string();
                    for raw in value.split(',') {
                        let tag = raw.split(';').next().unwrap_or("").trim();
                        if tag.is_empty() {
                            continue;
                        }
                        if SUPPORTED_LOCALES.contains(&tag) {
                            resolved = tag.to_string();
                            break;
                        }
                        if let Some(family) = tag.split('-').next()
                            && let Some(matched) = SUPPORTED_LOCALES
                                .iter()
                                .find(|l| l.split('-').next() == Some(family))
                        {
                            resolved = matched.to_string();
                            break;
                        }
                    }
                    resolved
                }
                // 3. Default locale
                else {
                    DEFAULT_LOCALE.to_string()
                }
            };

            req.extensions_mut().insert(RequestLocale(locale));
            service.call(req).await
        })
    }
}

/// Resolve the locale for an incoming request using the standard precedence:
///
/// 1. `X-Locale` header (explicit override, used by API clients).
/// 2. `Accept-Language` header (RFC 7231) — first supported language wins.
/// 3. Default locale.
pub fn resolve_request_locale(req: &actix_web::HttpRequest) -> String {
    // 1. X-Locale header (explicit override)
    if let Some(value) = req.headers().get("x-locale").and_then(|h| h.to_str().ok()) {
        let candidate = value.trim();
        if SUPPORTED_LOCALES.contains(&candidate) {
            return candidate.to_string();
        }
    }

    // 2. Accept-Language header (RFC 7231 negotiation)
    if let Some(value) = req
        .headers()
        .get(actix_web::http::header::ACCEPT_LANGUAGE)
        .and_then(|h| h.to_str().ok())
    {
        for raw in value.split(',') {
            let tag = raw.split(';').next().unwrap_or("").trim();
            if tag.is_empty() {
                continue;
            }
            // Exact match first
            if SUPPORTED_LOCALES.contains(&tag) {
                return tag.to_string();
            }
            // Language family match
            if let Some(family) = tag.split('-').next()
                && let Some(matched) = SUPPORTED_LOCALES
                    .iter()
                    .find(|l| l.split('-').next() == Some(family))
            {
                return matched.to_string();
            }
        }
    }

    // 3. Default locale
    DEFAULT_LOCALE.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_locale_precedence() {
        // Header > Accept-Language > Default

        // Exact header match
        let req = actix_web::test::TestRequest::default()
            .insert_header(("x-locale", "en"))
            .to_http_request();
        let res = resolve_request_locale(&req);
        assert_eq!(res, "en");

        // Accept-Language family fallback
        let req = actix_web::test::TestRequest::default()
            .insert_header(("accept-language", "es-ES,es;q=0.9"))
            .to_http_request();
        let res = resolve_request_locale(&req);
        assert_eq!(res, "es");

        // Default fallback
        let req = actix_web::test::TestRequest::default().to_http_request();
        let res = resolve_request_locale(&req);
        assert_eq!(res, DEFAULT_LOCALE);
    }

    #[test]
    fn unsupported_locale_falls_back() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("x-locale", "fr-FR"))
            .to_http_request();
        let res = resolve_request_locale(&req);
        assert_eq!(res, DEFAULT_LOCALE);
    }
}
