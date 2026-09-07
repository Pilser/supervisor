//! The JSON-RPC 2.0 message envelope used by MCP.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC error object (section 5.1 of the spec).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// One JSON-RPC message. It can be a request, a notification (no `id`), or a
/// response (either `result` or `error`). All optional fields are skipped so a
/// single type round-trips through both directions of the wire.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl Message {
    /// A request/notification message with no result.
    pub fn request(id: Option<Value>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// A successful response carrying `result`.
    pub fn response(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// An error response.
    pub fn response_error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// True when this message carries a method (a request or notification).
    pub fn is_request(&self) -> bool {
        self.method.is_some()
    }

    /// True for a notification: a request without an `id` (no response expected).
    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    /// True when this message is a response (carries result or error, no method).
    pub fn is_response(&self) -> bool {
        self.method.is_none()
    }

    /// True if this is a response that carried an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}