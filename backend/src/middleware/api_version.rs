use actix_web::{
    Error as ActixError, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
};
use futures::future::{LocalBoxFuture, Ready, ready};
use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// Metadata for a single API version.
#[derive(Clone, Debug)]
pub struct ApiVersionInfo {
    pub version: u16,
    pub released: String,
    pub deprecated: Option<String>,
    pub sunset: Option<String>,
    pub docs_url: Option<String>,
}

/// Configuration for the API versioning middleware.
#[derive(Clone, Debug)]
pub struct ApiVersionConfig {
    pub supported: Vec<u16>,
    pub deprecated: HashMap<u16, ApiVersionInfo>,
}

impl ApiVersionConfig {
    pub fn new() -> Self {
        // v1 is the initial version — mark as deprecated when v2 ships
        Self {
            supported: vec![1],
            deprecated: HashMap::new(),
        }
    }

    pub fn with_deprecated(mut self, info: ApiVersionInfo) -> Self {
        self.deprecated.insert(info.version, info);
        self
    }
}

impl Default for ApiVersionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Response body for version-related errors.
#[derive(Serialize)]
struct VersionError {
    error: VersionErrorBody,
}

#[derive(Serialize)]
struct VersionErrorBody {
    code: &'static str,
    message: String,
}

// Pre-allocated header names for custom headers.
const HEADER_DEPRECATION: header::HeaderName =
    header::HeaderName::from_static("deprecation");
const HEADER_SUNSET: header::HeaderName =
    header::HeaderName::from_static("sunset");
const HEADER_X_API_WARN: header::HeaderName =
    header::HeaderName::from_static("x-api-warn");
const HEADER_X_API_VERSION: header::HeaderName =
    header::HeaderName::from_static("x-api-version");
const HEADER_X_API_SUPPORTED: header::HeaderName =
    header::HeaderName::from_static("x-api-supported");

pub struct ApiVersionGuard {
    config: ApiVersionConfig,
}

impl ApiVersionGuard {
    pub fn new(config: ApiVersionConfig) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiVersionGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = ApiVersionGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiVersionGuardMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct ApiVersionGuardMiddleware<S> {
    service: Rc<S>,
    config: ApiVersionConfig,
}

impl<S> ApiVersionGuardMiddleware<S> {
    /// Extract API version from request.
    ///
    /// Priority:
    /// 1. URL path segment (`/api/v1/...`)
    /// 2. `Accept` header (`application/vnd.app-boilerplate.v{N}+json`)
    /// 3. `X-API-Version` header
    fn extract_version(req: &ServiceRequest) -> Option<u16> {
        let path = req.uri().path();
        if let Some(version) = Self::parse_version_from_path(path) {
            return Some(version);
        }

        if let Some(accept) = req.headers().get(header::ACCEPT) {
            if let Ok(accept_str) = accept.to_str() {
                if let Some(version) = Self::parse_version_from_accept(accept_str) {
                    return Some(version);
                }
            }
        }

        if let Some(header_val) = req.headers().get(header::HeaderName::from_static("x-api-version")) {
            if let Ok(val_str) = header_val.to_str() {
                if let Ok(v) = u16::from_str(val_str.trim()) {
                    return Some(v);
                }
            }
        }

        None
    }

    /// Parse version from URL path like `/api/v1/...` or `/api/v12/...`
    fn parse_version_from_path(path: &str) -> Option<u16> {
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() >= 3 && segments[1] == "api" {
            let v_segment = segments[2];
            if let Some(num_str) = v_segment.strip_prefix('v') {
                return u16::from_str(num_str).ok();
            }
        }
        None
    }

    /// Parse version from Accept header like:
    /// `application/vnd.app-boilerplate.v1+json` or
    /// `application/vnd.app-boilerplate.v2+json, application/json`
    fn parse_version_from_accept(accept: &str) -> Option<u16> {
        for part in accept.split(',') {
            let part = part.trim();
            if let Some(media) = part.strip_prefix("application/vnd.app-boilerplate.v") {
                if let Some(num_str) = media.strip_suffix("+json") {
                    return u16::from_str(num_str).ok();
                }
            }
        }
        None
    }
}

impl<S, B> Service<ServiceRequest> for ApiVersionGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
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
        let config = self.config.clone();

        Box::pin(async move {
            let path = req.uri().path();
            let is_api_path = path.starts_with("/api/v");

            if is_api_path {
                let version = Self::extract_version(&req);

                match version {
                    Some(v) if config.supported.contains(&v) => {
                        let mut res = svc.call(req).await?;

                        if let Some(deprecated_info) = config.deprecated.get(&v) {
                            let headers = res.response_mut().headers_mut();

                            headers.insert(
                                HEADER_DEPRECATION,
                                header::HeaderValue::from_static("true"),
                            );

                            if let Some(sunset) = &deprecated_info.sunset {
                                if let Ok(val) = header::HeaderValue::from_str(sunset) {
                                    headers.insert(HEADER_SUNSET, val);
                                }
                            }

                            if let Some(docs) = &deprecated_info.docs_url {
                                let link = format!("<{}>; rel=\"deprecation\"", docs);
                                if let Ok(val) = header::HeaderValue::from_str(&link) {
                                    headers.insert(header::LINK, val);
                                }
                            }

                            let warn_msg = format!(
                                "API v{} is deprecated. Migrate to latest version.",
                                v
                            );
                            if let Ok(val) = header::HeaderValue::from_str(&warn_msg) {
                                headers.insert(HEADER_X_API_WARN, val);
                            }
                        }

                        Ok(res.map_into_boxed_body())
                    }
                    Some(v) => {
                        let supported_list: Vec<String> =
                            config.supported.iter().map(|s| format!("v{}", s)).collect();
                        let body = serde_json::to_string(&VersionError {
                            error: VersionErrorBody {
                                code: "UNSUPPORTED_API_VERSION",
                                message: format!(
                                    "API version v{} is not supported. Supported versions: {}",
                                    v,
                                    supported_list.join(", ")
                                ),
                            },
                        })
                        .unwrap_or_default();

                        let (http_req, _) = req.into_parts();
                        let response = HttpResponse::BadRequest()
                            .insert_header((
                                HEADER_X_API_SUPPORTED,
                                header::HeaderValue::from_str(&supported_list.join(", ")).unwrap(),
                            ))
                            .content_type("application/json")
                            .body(body)
                            .map_into_boxed_body();

                        Ok(ServiceResponse::new(http_req, response))
                    }
                    None => {
                        let mut res = svc.call(req).await?;
                        res.response_mut().headers_mut().insert(
                            HEADER_X_API_VERSION,
                            header::HeaderValue::from_static("1"),
                        );
                        Ok(res.map_into_boxed_body())
                    }
                }
            } else {
                let res = svc.call(req).await?;
                Ok(res.map_into_boxed_body())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_from_path_v1() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_path("/api/v1/users"),
            Some(1)
        );
    }

    #[test]
    fn parse_version_from_path_v12() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_path("/api/v12/admin/roles"),
            Some(12)
        );
    }

    #[test]
    fn parse_version_from_path_no_version() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_path("/api/users"),
            None
        );
    }

    #[test]
    fn parse_version_from_path_health() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_path("/health"),
            None
        );
    }

    #[test]
    fn parse_version_from_accept_vendor() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_accept(
                "application/vnd.app-boilerplate.v1+json"
            ),
            Some(1)
        );
    }

    #[test]
    fn parse_version_from_accept_multiple() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_accept(
                "application/vnd.app-boilerplate.v2+json, application/json"
            ),
            Some(2)
        );
    }

    #[test]
    fn parse_version_from_accept_plain_json() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_accept("application/json"),
            None
        );
    }

    #[test]
    fn parse_version_from_accept_v3() {
        assert_eq!(
            ApiVersionGuardMiddleware::<()>::parse_version_from_accept(
                "text/html, application/vnd.app-boilerplate.v3+json"
            ),
            Some(3)
        );
    }

    #[test]
    fn config_supported_versions() {
        let config = ApiVersionConfig::new();
        assert!(config.supported.contains(&1));
        assert!(!config.supported.contains(&99));
    }

    #[test]
    fn config_deprecated_versions() {
        let config = ApiVersionConfig::new().with_deprecated(ApiVersionInfo {
            version: 1,
            released: "2024-01-01".to_string(),
            deprecated: Some("2025-06-01".to_string()),
            sunset: Some("2025-12-31".to_string()),
            docs_url: Some("https://docs.example.com/migration".to_string()),
        });

        assert!(config.deprecated.contains_key(&1));
        let info = config.deprecated.get(&1).unwrap();
        assert_eq!(info.sunset.as_deref(), Some("2025-12-31"));
        assert_eq!(info.docs_url.as_deref(), Some("https://docs.example.com/migration"));
    }

    #[test]
    fn config_default_has_v1() {
        let config = ApiVersionConfig::default();
        assert_eq!(config.supported, vec![1]);
        assert!(config.deprecated.is_empty());
    }
}
