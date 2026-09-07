//! Agent-specific monitoring config.

use super::registry::ProgramSpec;

/// Default spec for the agent binary.
pub fn agent_spec(binary: &str) -> ProgramSpec {
    ProgramSpec {
        name: "agent".to_string(),
        binary: binary.to_string(),
        args: vec!["--mcp-socket".to_string(), "/tmp/supervisor.sock".to_string()],
        restart_on_crash: true,
    }
}
