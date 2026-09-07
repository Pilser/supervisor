//! Signing wrapper requests with the supervisor token.

use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

/// Header used to convey the supervisor token to the wrapper.
pub const AUTH_HEADER: &str = "x-supervisor-token";

/// Apply the supervisor token as the `x-supervisor-token` request header.
pub fn apply_auth_header(headers: &mut HeaderMap, token: &str) {
    if let Ok(value) = HeaderValue::from_str(token) {
        headers.insert(reqwest::header::HeaderName::from_static(AUTH_HEADER), value);
    }
}

/// Embed the supervisor token into the payload so the request is signed end to
/// end (useful for offline-deferred requests that need to re-authenticate).
pub fn embed_token(value: &mut Value, token: &str) -> anyhow::Result<()> {
    value[AUTH_HEADER] = Value::String(token.to_string());
    Ok(())
}

/// Embed the supervisor token into the envelope payload (`payload.token`).
///
/// The canonical wrapper envelope travels as NDJSON over plain TCP, so there
/// are no HTTP headers on the wire: auth rides inside `payload`. A missing or
/// non-object `payload` is replaced with a fresh object carrying the token.
pub fn embed_token_in_payload(envelope: &mut Value, token: &str) -> anyhow::Result<()> {
    match envelope.get_mut("payload") {
        Some(Value::Object(map)) => {
            map.insert("token".to_string(), Value::String(token.to_string()));
        }
        _ => {
            envelope["payload"] = serde_json::json!({ "token": token });
        }
    }
    Ok(())
}

/// Produce an `x-supervisor-token` header value string given the token.
pub fn token_header(token: &str) -> String {
    token.to_string()
}