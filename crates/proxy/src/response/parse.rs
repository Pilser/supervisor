//! Parse wrapper responses into [`Response`](super::Response).

use std::time::Duration;

use serde_json::Value;

use super::error::parse_error;
use super::Response;
use crate::request::RequestType;

/// Build a typed [`Response`] from one wrapper NDJSON response envelope.
///
/// Wrapper response lines look like
/// `{ "type": ..., "tenant_id": ..., "timestamp": ..., "payload": ... }`.
/// The returned [`Response::data`] is the unwrapped `payload` (or
/// `payload.data` when the payload itself wraps a `data` field); the full
/// envelope is preserved as [`Response::raw`]. Envelopes without a `payload`
/// fall back to the legacy `{ "status": "ok", "data": ... }` shape.
pub fn parse_response(rtype: Option<RequestType>, raw: Value) -> Response {
    let inner = raw.get("payload").cloned().unwrap_or_else(|| raw.clone());
    let error = parse_error(&inner).or_else(|| parse_error(&raw));
    let ok = error.is_none();
    let data = inner
        .get("data")
        .cloned()
        .unwrap_or_else(|| inner.clone());
    let status_code = inner
        .get("status_code")
        .or_else(|| raw.get("status_code"))
        .and_then(Value::as_u64)
        .unwrap_or(if ok { 200 } else { 500 }) as u16;
    Response {
        status_code,
        ok,
        rtype,
        duration: Duration::ZERO,
        error,
        raw,
        data,
    }
}
