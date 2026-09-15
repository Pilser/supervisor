//! Engram memory-daemon monitoring config.

use super::registry::ProgramSpec;

/// Default spec for the engram binary (memory vector daemon, MCP 127.0.0.1:4545).
pub fn engram_spec(binary: &str) -> ProgramSpec {
    ProgramSpec {
        name: "engram".to_string(),
        binary: binary.to_string(),
        args: vec![],
        restart_on_crash: true,
        // `engram serve` double-forks: the spawned parent exits at once and
        // the daemon grandchild (PPID 1) does the work. Supervise the :4545
        // listener, not the launcher PID.
        daemonizes: true,
        health_port: Some(4545),
    }
}
