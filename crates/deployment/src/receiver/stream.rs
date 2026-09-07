//! Receive large file streams in chunks.

use core::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// One chunk of a streamed artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamedChunk {
    pub index: u32,
    pub total: u32,
    pub bytes: Vec<u8>,
}

/// Reassemble ordered chunks into the full byte stream.
pub fn assemble_chunks(chunks: &[StreamedChunk]) -> Result<Vec<u8>> {
    if chunks.is_empty() {
        return Err(SupervisorError::Deployment("no chunks received".into()));
    }
    let mut sorted = chunks.to_vec();
    sorted.sort_by_key(|c| c.index);
    let mut out = Vec::new();
    for chunk in &sorted {
        out.extend_from_slice(&chunk.bytes);
    }
    Ok(out)
}
