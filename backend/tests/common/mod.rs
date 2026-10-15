//! Shared helpers for integration tests.
//!
//! This module follows the Cargo convention (`tests/common/mod.rs`): it is
//! not compiled as its own test binary — tests pull it in via `mod common;`.

/// Dispatch a request through an app, normalizing BOTH middleware outcomes
/// into a `ServiceResponse`:
/// - `Ok(response)` — the normal path
/// - `Err(error)` — how `JwtAuth` rejects requests (`InternalError` carries
///   the real HTTP response; actix renders it to the client)
///
/// `test::call_service` panics on the `Err` path, so tests that assert on
/// rejection responses (401/503) must use this macro instead.
///
/// The builder closure is invoked twice: once to harvest an `HttpRequest`
/// for `ServiceResponse::from_err`, once for the actual call.
macro_rules! dispatch {
    ($app:expr, $req_builder:expr) => {{
        let http_request = $req_builder().to_srv_request().into_parts().0;
        let result = $app.call($req_builder().to_request()).await;
        match result {
            Ok(res) => res,
            Err(err) => actix_web::dev::ServiceResponse::from_err(err, http_request),
        }
    }};
}

pub(crate) use dispatch;
