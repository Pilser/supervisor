//! Validate loaded configuration.

use super::SupervisorConfig;

/// Check required values and sane ranges.
pub fn validate_config(cfg: &SupervisorConfig) -> supervisor_core::Result<()> {
    if cfg.supervisor.vm_id.is_empty() {
        return Err(supervisor_core::SupervisorError::Config("supervisor.vm_id is empty".into()));
    }
    if cfg.supervisor.tenant_id.is_empty() {
        return Err(supervisor_core::SupervisorError::Config("supervisor.tenant_id is empty".into()));
    }
    if cfg.wrapper.address.is_empty() {
        return Err(supervisor_core::SupervisorError::Config("wrapper.address is empty".into()));
    }
    if cfg.wrapper.port == 0 {
        return Err(supervisor_core::SupervisorError::Config("wrapper.port is 0".into()));
    }
    if cfg.monitor.check_interval_seconds == 0 {
        return Err(supervisor_core::SupervisorError::Config(
            "monitor.check_interval_seconds must be > 0".into(),
        ));
    }
    if cfg.heartbeat.interval_seconds == 0 {
        return Err(supervisor_core::SupervisorError::Config(
            "heartbeat.interval_seconds must be > 0".into(),
        ));
    }
    if cfg.commands.listen_port == 0 {
        return Err(supervisor_core::SupervisorError::Config("commands.listen_port is 0".into()));
    }
    if cfg.deployment.port == 0 {
        return Err(supervisor_core::SupervisorError::Config("deployment.port is 0".into()));
    }
    if cfg.programs.agent.binary.is_empty() || cfg.programs.memory.binary.is_empty() {
        return Err(supervisor_core::SupervisorError::Config("program binary is empty".into()));
    }
    match cfg.telemetry.output.as_str() {
        "disk" | "forward" | "both" | "none" => {}
        other => {
            return Err(supervisor_core::SupervisorError::Config(format!(
                "telemetry.output must be disk|forward|both|none, got {other}"
            )));
        }
    }
    Ok(())
}
