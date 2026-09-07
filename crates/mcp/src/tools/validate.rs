//! Lightweight param validation against a tool's JSON schema.

use anyhow::bail;
use serde_json::Value;

/// Ensure all parameters listed as `required` in `schema` are present in
/// `params`. Empty/root-level objects and non-object params are treated as
/// having no required constraints.
pub fn validate_params(schema: &Value, params: &Value) -> anyhow::Result<()> {
    let Some(required) = schema
        .get("required")
        .and_then(|r| r.as_array())
    else {
        return Ok(());
    };
    if required.is_empty() {
        return Ok(());
    }
    let map = params
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    for req in required {
        if let Some(key) = req.as_str() {
            if !map.contains_key(key) {
                bail!("missing required parameter: {key}");
            }
        }
    }
    Ok(())
}