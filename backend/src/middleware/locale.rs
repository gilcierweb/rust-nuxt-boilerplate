//! Locale resolution middleware.
//!
//! Inspects every incoming request, resolves the locale to use for that
//! request (via `X-Locale` header, then `Accept-Language`, then default), and
//! stores the result in `req.extensions_mut()` as a [`RequestLocale`].
//!
//! Per-request locale avoids the global mutable state of `rust-i18n` and
//! guarantees no locale leakage between concurrent requests.

use std::cell::RefCell;
use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;

use crate::services::translation_service::{
    LocaleResolution, Translations, resolve_request_locale,
};

/// Per-request locale stored in the request extensions.
#[derive(Debug, Clone)]
pub struct RequestLocale {
    pub resolution: LocaleResolution,
    pub translations: Translations,
}

impl RequestLocale {
    /// Translate a key for this request's locale with optional pattern args.
    pub async fn t(
        &self,
        key: &str,
        args: Option<&std::collections::HashMap<String, String>>,
    ) -> String {
        self.translations
            .translate(&self.resolution.locale, key, args)
            .await
    }

    /// Synchronous translation helper. Used by error builders and other
    /// non-async contexts where awaiting the translation lock is impossible.
    pub fn t_blocking(
        &self,
        key: &str,
        args: Option<&std::collections::HashMap<String, String>>,
    ) -> String {
        self.translations
            .translate_blocking(&self.resolution.locale, key, args)
    }
}

pub struct LocaleMiddleware {
    translations: Translations,
}

impl LocaleMiddleware {
    pub fn new(translations: Translations) -> Self {
        Self { translations }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LocaleMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LocaleMiddlewareInner<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LocaleMiddlewareInner {
            service: Rc::new(service),
            translations: self.translations.clone(),
        }))
    }
}

pub struct LocaleMiddlewareInner<S> {
    service: Rc<S>,
    translations: Translations,
}

thread_local! {
    /// Thread-local fallback for the current request locale.
    ///
    /// Actix-web handlers and `ResponseError::error_response` (which does not
    /// receive the `HttpRequest`) need a way to read the active locale. The
    /// request extensions remain the primary source — this thread-local is
    /// populated by the middleware *before* invoking the inner service and
    /// cleared after, so within the lifetime of a single request any code
    /// running on the worker thread can read it via [`current_request_locale`].
    ///
    /// Because Actix-web pins each request to a single worker thread for the
    /// duration of the future, this thread-local cannot leak across concurrent
    /// requests. The middleware always restores the previous value (or clears
    /// it) after the inner future resolves.
    static CURRENT_REQUEST_LOCALE: RefCell<Option<RequestLocale>> = const { RefCell::new(None) };
}

/// Read the current request locale from thread-local storage.
///
/// Returns `None` if called outside of a request handler (e.g., startup,
/// background tasks) or if the locale middleware did not populate the value.
pub fn current_request_locale() -> Option<RequestLocale> {
    CURRENT_REQUEST_LOCALE.with(|cell| cell.borrow().clone())
}

/// Convenience extractor: pull the [`RequestLocale`] out of an `HttpRequest`.
pub fn locale_from_request(req: &actix_web::HttpRequest) -> Option<RequestLocale> {
    req.extensions().get::<RequestLocale>().cloned()
}

impl<S, B> Service<ServiceRequest> for LocaleMiddlewareInner<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let translations = self.translations.clone();

        Box::pin(async move {
            let available_locales = translations.available_locales().await;
            let resolution = resolve_request_locale(req.request(), &available_locales);

            let request_locale = RequestLocale {
                resolution: resolution.clone(),
                translations: translations.clone(),
            };

            req.extensions_mut().insert(request_locale.clone());

            // Populate the thread-local so non-async error builders can read it.
            let previous = CURRENT_REQUEST_LOCALE.with(|cell| cell.borrow().clone());
            CURRENT_REQUEST_LOCALE.with(|cell| *cell.borrow_mut() = Some(request_locale));

            let result = service.call(req).await;

            // Restore the previous value (usually `None`) so the worker thread
            // is clean before the next request lands on it.
            CURRENT_REQUEST_LOCALE.with(|cell| *cell.borrow_mut() = previous);

            result
        })
    }
}

/// `LocaleSource` is re-exported to avoid leaking the internal types.
pub use crate::services::translation_service::LocaleSource as RequestLocaleSource;
