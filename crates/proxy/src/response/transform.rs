//! Transform wrapper responses for supervisor use.

use super::Response;

/// Normalize `resp` in place: unwrap nested `data` envelopes and default the
/// error message on failures without one.
pub fn transform_response(resp: &mut Response) {
    if let serde_json::Value::Object(map) = &resp.data {
        if map.len() == 1 && map.contains_key("data") {
            if let Some(inner) = map.get("data").cloned() {
                resp.data = inner;
            }
        }
    }
    if !resp.ok && resp.error.is_none() {
        resp.error = Some(format!("wrapper returned status {}", resp.status_code));
    }
}
