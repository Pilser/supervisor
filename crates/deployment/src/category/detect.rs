//! Detect the current tenant category.

use core::Result;

/// Current category from the environment, defaulting to `"default"`.
pub fn detect_category() -> Result<String> {
    Ok(std::env::var("SUPERVISOR_CATEGORY").unwrap_or_else(|_| "default".to_string()))
}
