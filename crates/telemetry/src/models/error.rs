//! Structured error payload attached to error events.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub kind: String,
    pub message: String,
    pub stack: Option<String>,
    pub context: std::collections::HashMap<String, serde_json::Value>,
}

impl ErrorPayload {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            stack: None,
            context: Default::default(),
        }
    }
}