//! Parsed and transformed wrapper responses.

pub mod error;
pub mod parse;
pub mod transform;

pub use error::ProxyError;

use std::time::Duration;

use serde_json::Value;

use crate::request::RequestType;

/// A typed wrapper response: raw payload plus parsed/transformed view.
#[derive(Debug, Clone)]
pub struct Response {
    /// The wrapper HTTP status code.
    pub status_code: u16,
    /// Whether the request succeeded.
    pub ok: bool,
    /// The request type this response answers.
    pub rtype: Option<RequestType>,
    /// Round-trip duration.
    pub duration: Duration,
    /// Structured error message, when not ok.
    pub error: Option<String>,
    /// The raw envelope returned by the wrapper.
    pub raw: Value,
    /// The transformed payload usable by the supervisor.
    pub data: Value,
}

impl Response {
    /// Build a response directly from a cached value (no network round trip).
    pub fn from_cache(data: Value) -> Self {
        Self {
            status_code: 200,
            ok: true,
            rtype: None,
            duration: Duration::ZERO,
            error: None,
            raw: data.clone(),
            data,
        }
    }

    /// Reference to the transformed data.
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Consume and return the transformed data.
    pub fn into_data(self) -> Value {
        self.data
    }

    /// Whether the response indicates success.
    pub fn is_ok(&self) -> bool {
        self.ok
    }
}