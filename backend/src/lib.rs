#[macro_use]
extern crate rust_i18n;

pub mod api_docs;
pub mod auth;
pub mod authz;
pub mod config;
pub mod controllers;
pub mod db;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod security;
pub mod services;
pub mod traced_http;
pub mod utils;
pub mod ws;

#[cfg(test)]
pub mod test_utils;

use std::sync::Arc;

pub use db::database::DBPool;

pub struct AppState {
    pub db: crate::db::database::DBPool,
    pub redis: deadpool_redis::Pool,
    pub config: Arc<crate::config::AppConfig>,
    pub metrics: Arc<crate::services::metrics_service::MetricsRegistry>,
    pub ws: crate::ws::WsRedisState,
    /// Cached `Arc<Vec<JwtSecretKey>>` for O(1) clones during JWT verification.
    /// Avoids cloning the full `Vec<JwtSecretKey>` from `config.jwt_secrets`
    /// on every authenticated request.
    pub jwt_secrets: Arc<Vec<crate::config::app_config::JwtSecretKey>>,
}

i18n!("locales");
