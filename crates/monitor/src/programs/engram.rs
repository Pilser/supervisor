//! Engram memory-daemon monitoring config.

use super::registry::ProgramSpec;

/// Default spec for the engram binary (memory vector daemon, MCP 127.0.0.1:4545).
pub fn engram_spec(binary: &str) -> ProgramSpec {
    ProgramSpec {
        name: "engram".to_string(),
        binary: binary.to_string(),
        args: vec![],
        restart_on_crash: true,
        // launch-engram.sh ends in `exec engram serve`: ps shows "engram".
        match_name: Some("engram".to_string()),
    }
}
