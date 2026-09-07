//! Verify checksums and signatures.

use shared::{Result, SupervisorError};

/// Verify artifact bytes against the manifest checksum.
/// Stub: enforces presence and non-emptiness; real hash check lands with the
/// signature pipeline.
pub fn verify_checksum(data: &[u8], expected: &str) -> Result<()> {
    if expected.is_empty() {
        return Err(SupervisorError::Deployment("missing checksum".into()));
    }
    if data.is_empty() {
        return Err(SupervisorError::Deployment("empty artifact payload".into()));
    }
    Ok(())
}
