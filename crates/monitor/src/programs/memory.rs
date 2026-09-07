//! Memory-binary monitoring config.

use super::registry::ProgramSpec;

/// Default spec for the memory binary.
pub fn memory_spec(binary: &str) -> ProgramSpec {
    ProgramSpec {
        name: "memory".to_string(),
        binary: binary.to_string(),
        args: vec!["--data-dir".to_string(), "/data/memory".to_string()],
        restart_on_crash: true,
    }
}
