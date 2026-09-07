//! Verify a command comes from the wrapper.

use shared::{Result, SupervisorError};

/// Compare the presented bearer token against the configured one.
pub fn verify_token(provided: &str, expected: &str) -> Result<()> {
    if provided.is_empty() || provided != expected {
        return Err(SupervisorError::Unauthorized("bad command auth token".into()));
    }
    Ok(())
}
