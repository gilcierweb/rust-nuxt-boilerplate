pub mod blind_index;
pub mod encryption;
pub mod key_manager;
mod normalization;
mod service;
#[cfg(test)]
pub(crate) mod test_utils;

#[allow(unused_imports)]
pub use normalization::{normalize_cpf, normalize_email, normalize_phone};
#[allow(unused_imports)]
pub use service::{ProtectedValue, SecurityService};
