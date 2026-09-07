//! Optional compression before forwarding.
//!
//! Currently a no-op pass-through: returns the input bytes unchanged. Wiring a
//! real gzip encoder later needs no caller changes.

/// Return `data` unchanged (no-op compression placeholder).
pub fn compress(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}
