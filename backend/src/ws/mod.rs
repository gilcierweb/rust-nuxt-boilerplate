#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod server;
pub mod validation;

pub use server::WsState;

#[derive(Clone)]
pub struct WsStateOld {
    pub connections: HashMap<String, String>,
}

impl WsStateOld {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
}

impl Default for WsStateOld {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsMessageOld {
    pub msg_type: String,
    pub payload: serde_json::Value,
}
