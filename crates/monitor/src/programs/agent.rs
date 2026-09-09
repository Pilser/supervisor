//! Agent-specific monitoring config.

use super::registry::ProgramSpec;

/// Default spec for the agent binary.
///
/// NOTE: no default CLI args. `zeroclaw daemon` rejects foreign flags (it
/// prints usage and exits, which looks like a crash-loop), so the guest
/// launch script (`launch-zeroclaw.sh`) configures the daemon via
/// environment (ZEROCLAW_DATA_DIR/ZEROCLAW_SOCKET) instead. Pass program
/// specific args explicitly via `custom_spec` when the binary supports them.
pub fn agent_spec(binary: &str) -> ProgramSpec {
    ProgramSpec {
        name: "agent".to_string(),
        binary: binary.to_string(),
        args: vec![],
        restart_on_crash: true,
    }
}
