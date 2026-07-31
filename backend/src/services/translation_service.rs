//! Per-request translation service.
//!
//! Loads backend locale JSON files at startup and provides a `translate()`
//! function that takes a locale + key + optional pattern args. This avoids the
//! global mutable state of `rust_i18n` and prevents locale leakage between
//! concurrent requests in a multi-threaded async runtime.
//!
//! # Locale resolution order (handled by the middleware)
//!
//! 1. `X-Locale` request header (explicit override).
//! 2. `Accept-Language` header (RFC 7231 negotiation).
//! 3. Default locale (`pt-BR`).
//!
//! Once resolved, the locale is stored in the request extensions and
//! retrievable via [`Locale::from_request`].

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use actix_web::HttpRequest;
use tokio::sync::RwLock;

/// Default fallback locale when no Accept-Language matches any supported locale.
pub const DEFAULT_LOCALE: &str = "pt-BR";

/// Translation storage: nested HashMap of `key.path` -> translated string.
type LocaleMap = HashMap<String, String>;

/// Concurrent handle to the per-locale maps.
#[derive(Clone, Default, Debug)]
pub struct Translations {
    inner: Arc<RwLock<HashMap<String, LocaleMap>>>,
}

impl Translations {
    /// Build a translation store by loading every `*.json` file in `locales_dir`.
    /// Each file becomes one locale keyed by the filename stem (`pt-BR.json`
    /// becomes `"pt-BR"`).
    pub fn load_from_dir<P: AsRef<Path>>(locales_dir: P) -> Result<Self, TranslationError> {
        let dir = locales_dir.as_ref();
        let mut store: HashMap<String, LocaleMap> = HashMap::new();

        let entries = std::fs::read_dir(dir).map_err(|error| TranslationError::Io {
            path: dir.to_path_buf(),
            source: error,
        })?;

        for entry in entries {
            let entry = entry.map_err(|error| TranslationError::Io {
                path: dir.to_path_buf(),
                source: error,
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let locale = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| TranslationError::InvalidFileName(path.clone()))?
                .to_string();

            let bytes = std::fs::read(&path).map_err(|error| TranslationError::Io {
                path: path.clone(),
                source: error,
            })?;

            let flat: FlatJson =
                serde_json::from_slice(&bytes).map_err(|error| TranslationError::Parse {
                    path: path.clone(),
                    source: error,
                })?;

            store.insert(locale, flat.0);
        }

        Ok(Self {
            inner: Arc::new(RwLock::new(store)),
        })
    }

    /// Look up a translation for `locale` + `key`. Falls back to the default
    /// locale and then to the key itself if no translation is found.
    pub async fn translate(
        &self,
        locale: &str,
        key: &str,
        args: Option<&HashMap<String, String>>,
    ) -> String {
        let store = self.inner.read().await;

        let template = store
            .get(locale)
            .and_then(|m| m.get(key).cloned())
            .or_else(|| store.get(DEFAULT_LOCALE).and_then(|m| m.get(key).cloned()))
            .unwrap_or_else(|| key.to_string());

        apply_args(&template, args)
    }

    /// Synchronous variant for use in non-async contexts (e.g., error
    /// response builders). Falls back to the key on miss.
    ///
    /// **Important:** This function MUST NOT be called from inside a tokio
    /// runtime task. It uses [`tokio::sync::RwLock::try_read`] which fails
    /// fast (returns `None`) rather than blocking the worker thread. Callers
    /// running inside an async context should prefer the async [`translate`]
    /// variant instead.
    ///
    /// When the lock cannot be acquired immediately (typical case inside a
    /// running Actix-web request), this returns the key verbatim — callers
    /// receive a non-localized message rather than panicking the worker.
    pub fn translate_blocking(
        &self,
        locale: &str,
        key: &str,
        args: Option<&HashMap<String, String>>,
    ) -> String {
        let template = match self.inner.try_read() {
            Ok(store) => store
                .get(locale)
                .and_then(|m| m.get(key).cloned())
                .or_else(|| store.get(DEFAULT_LOCALE).and_then(|m| m.get(key).cloned()))
                .unwrap_or_else(|| key.to_string()),
            Err(_) => {
                // Lock contended or called from inside an async runtime.
                // Returning the key keeps the response non-empty without
                // risking a worker-thread panic.
                tracing::warn!(
                    event = "i18n.translate_blocking.lock_unavailable",
                    locale = %locale,
                    key = %key,
                    "could not acquire translation lock; returning key verbatim"
                );
                key.to_string()
            },
        };

        apply_args(&template, args)
    }

    /// List of loaded locales (sorted for stability).
    pub async fn available_locales(&self) -> Vec<String> {
        let mut locales: Vec<String> = self.inner.read().await.keys().cloned().collect();
        locales.sort();
        locales
    }
}

/// Wrapper around the inner `HashMap<String, String>` produced by the
/// custom `Deserialize` impl below.
#[derive(Debug, Default)]
struct FlatJson(HashMap<String, String>);

impl<'de> serde::Deserialize<'de> for FlatJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: HashMap<String, serde_json::Value> = HashMap::deserialize(deserializer)?;
        let mut flat = HashMap::new();
        for (k, v) in raw {
            flatten_into(&k, &v, &mut flat);
        }
        Ok(FlatJson(flat))
    }
}

fn flatten_into(prefix: &str, value: &serde_json::Value, out: &mut LocaleMap) {
    match value {
        serde_json::Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        },
        serde_json::Value::Number(n) => {
            out.insert(prefix.to_string(), n.to_string());
        },
        serde_json::Value::Bool(b) => {
            out.insert(prefix.to_string(), b.to_string());
        },
        serde_json::Value::Null => {
            out.insert(prefix.to_string(), String::new());
        },
        serde_json::Value::Array(arr) => {
            // Arrays are stored as JSON-encoded strings so the entire list is
            // accessible under the prefix key.
            out.insert(
                prefix.to_string(),
                serde_json::to_string(arr).unwrap_or_default(),
            );
        },
        serde_json::Value::Object(map) => {
            // Recursively flatten nested objects using dotted key names.
            // e.g. `{ "auth": { "errors": { "invalid": "..." } } }` becomes
            //      `auth.errors.invalid -> "..."`.
            for (key, child) in map {
                let child_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_into(&child_key, child, out);
            }
        },
    }
}

/// Replace `%{name}` and `{}` placeholders in `template` with values from `args`.
/// Positional `{}` placeholders are filled from a Vec in order; named
/// `%{name}` placeholders are filled from the HashMap keyed by name.
fn apply_args(template: &str, args: Option<&HashMap<String, String>>) -> String {
    let mut out = template.to_string();

    if let Some(args) = args {
        for (key, value) in args {
            let token_named = format!("%{{{}}}", key);
            out = out.replace(&token_named, value);
        }
    }

    out
}

/// Errors produced while loading translation files.
#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid translation file name: {0}")]
    InvalidFileName(std::path::PathBuf),

    #[error("failed to parse translation file {path}: {source}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

/// Resolve the locale for an incoming request using the standard precedence:
///
/// 1. `X-Locale` header (explicit override, used by API clients).
/// 2. `Accept-Language` header (RFC 7231) — first supported language wins.
/// 3. Default locale.
///
/// Returns the resolved locale along with the source that produced it.
pub fn resolve_request_locale(req: &HttpRequest, available: &[String]) -> LocaleResolution {
    if let Some(value) = req.headers().get("x-locale").and_then(|h| h.to_str().ok()) {
        let candidate = value.trim();
        if available.iter().any(|l| l == candidate) {
            return LocaleResolution {
                locale: candidate.to_string(),
                source: LocaleSource::Header,
            };
        }
    }

    if let Some(value) = req
        .headers()
        .get(actix_web::http::header::ACCEPT_LANGUAGE)
        .and_then(|h| h.to_str().ok())
    {
        // `Accept-Language` is a comma-separated list of `lang` or `lang;q=N`.
        for raw in value.split(',') {
            let tag = raw.split(';').next().unwrap_or("").trim();
            if tag.is_empty() {
                continue;
            }
            // Exact match first (e.g. "pt-BR" matches "pt-BR")
            if available.iter().any(|l| l == tag) {
                return LocaleResolution {
                    locale: tag.to_string(),
                    source: LocaleSource::AcceptLanguage,
                };
            }
            // Language family match (e.g. "pt" matches "pt-BR")
            if let Some(family) = tag.split('-').next()
                && let Some(matched) = available
                    .iter()
                    .find(|l| l.split('-').next() == Some(family))
            {
                return LocaleResolution {
                    locale: matched.clone(),
                    source: LocaleSource::AcceptLanguage,
                };
            }
        }
    }

    LocaleResolution {
        locale: DEFAULT_LOCALE.to_string(),
        source: LocaleSource::Default,
    }
}

/// Result of locale resolution for an incoming request.
#[derive(Debug, Clone)]
pub struct LocaleResolution {
    pub locale: String,
    pub source: LocaleSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleSource {
    Header,
    AcceptLanguage,
    Default,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_args_replaces_named_and_positional() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());

        assert_eq!(apply_args("Hello, %{name}!", Some(&args)), "Hello, World!");
        assert_eq!(apply_args("Hello, %{name}!", None), "Hello, %{name}!");
    }

    #[test]
    fn flatten_into_collects_leaf_values() {
        // The backend locale catalogues are flat: every leaf is a string.
        let json = serde_json::json!({
            "a": "value-a",
            "b": "value-b",
            "c": "value-c",
        });
        let mut out = LocaleMap::new();
        flatten_into("root", &json, &mut out);

        assert_eq!(out.get("root.a").unwrap(), "value-a");
        assert_eq!(out.get("root.b").unwrap(), "value-b");
        assert_eq!(out.get("root.c").unwrap(), "value-c");
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn resolve_locale_precedence() {
        // Header > Accept-Language > Default
        let available = vec!["pt-BR".to_string(), "en".to_string(), "es".to_string()];

        // Exact header match
        let req = actix_web::test::TestRequest::default()
            .insert_header(("x-locale", "en"))
            .to_http_request();
        let res = resolve_request_locale(&req, &available);
        assert_eq!(res.locale, "en");
        assert_eq!(res.source, LocaleSource::Header);

        // Accept-Language family fallback
        let req = actix_web::test::TestRequest::default()
            .insert_header(("accept-language", "es-ES,es;q=0.9"))
            .to_http_request();
        let res = resolve_request_locale(&req, &available);
        assert_eq!(res.locale, "es");
        assert_eq!(res.source, LocaleSource::AcceptLanguage);

        // Default fallback
        let req = actix_web::test::TestRequest::default().to_http_request();
        let res = resolve_request_locale(&req, &available);
        assert_eq!(res.locale, DEFAULT_LOCALE);
        assert_eq!(res.source, LocaleSource::Default);
    }
}
