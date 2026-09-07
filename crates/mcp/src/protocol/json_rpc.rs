//! JSON-RPC 2.0 constants and response helpers.

use serde_json::Value;

use super::message::{JsonRpcError, Message};

/// The JSON-RPC 2.0 version string every message must carry.
pub const JSONRPC_VERSION: &str = "2.0";

/// Request payload was not valid JSON.
pub const PARSE_ERROR: i64 = -32700;
/// The JSON sent is not a valid Request object.
pub const INVALID_REQUEST: i64 = -32600;
/// The method does not exist / is not recognisable.
pub const METHOD_NOT_FOUND: i64 = -32601;
/// Invalid method parameter(s).
pub const INVALID_PARAMS: i64 = -32602;
/// Internal JSON-RPC error.
pub const INTERNAL_ERROR: i64 = -32603;

/// Build a successful response carrying `result`.
pub fn response_ok(id: Option<Value>, result: Value) -> Message {
    Message::response(id, result)
}

/// Build an error response.
pub fn response_err(id: Option<Value>, code: i64, message: impl Into<String>) -> Message {
    Message::response_error(id, JsonRpcError::new(code, message))
}

/// Parse a request id out of a message for reuse on the response.
pub fn id_of(msg: &Message) -> Option<Value> {
    msg.id.clone()
}