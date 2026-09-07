//! Supervisor configuration: load and validate `config.toml`.

pub mod load;
pub mod validate;

use serde::{Deserialize, Serialize};

/// Full supervisor configuration (mirrors `config.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    pub supervisor: SupervisorSection,
    pub wrapper: WrapperSection,
    pub mcp: McpSection,
    pub monitor: MonitorSection,
    pub heartbeat: HeartbeatSection,
    pub commands: CommandsSection,
    pub deployment: DeploymentSection,
    pub plugins: PluginsSection,
    pub programs: ProgramsSection,
    pub telemetry: TelemetrySection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorSection {
    pub vm_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperSection {
    pub address: String,
    pub port: u16,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSection {
    pub socket_path: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSection {
    pub check_interval_seconds: u64,
    pub freeze_timeout_seconds: u64,
    pub auto_restart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatSection {
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsSection {
    pub listen_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSection {
    pub plugins_dir: String,
    pub agent_plugins_dir: String,
    pub configs_dir: String,
    pub binaries_dir: String,
    pub temp_dir: String,
    /// TCP port this supervisor listens on for deployment pushes (default 8091).
    #[serde(default = "default_deployment_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsSection {
    pub directory: String,
    pub hot_reload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramSection {
    pub binary: String,
    pub args: Vec<String>,
    pub restart_on_crash: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramsSection {
    pub agent: ProgramSection,
    pub memory: ProgramSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySection {
    pub enabled: bool,
    pub output: String,
    pub disk: TelemetryDiskSection,
    pub forward: TelemetryForwardSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDiskSection {
    pub path: String,
    pub filename_pattern: String,
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryForwardSection {
    pub wrapper_address: String,
    pub wrapper_port: u16,
    pub batch_size: usize,
    pub flush_interval_seconds: u64,
    pub retry_attempts: u32,
    pub compress: bool,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        load::load_from_str(include_str!("../../config.toml"))
            .expect("bundled config.toml must parse")
    }
}

fn default_deployment_port() -> u16 {
    8091
}

impl SupervisorConfig {
    pub fn validate(&self) -> supervisor_core::Result<()> {
        validate::validate_config(self)
    }

    /// Convert the `[telemetry]` section into telemetry settings.
    pub fn telemetry_settings(&self) -> telemetry::TelemetrySettings {
        telemetry::TelemetrySettings {
            enabled: self.telemetry.enabled,
            output: self.telemetry.output.clone(),
            disk: telemetry::config::DiskSettings {
                enabled: true,
                path: self.telemetry.disk.path.clone(),
                filename_pattern: self.telemetry.disk.filename_pattern.clone(),
                buffer_size: self.telemetry.disk.buffer_size,
                flush_interval_seconds: self.telemetry.disk.flush_interval_seconds,
                retention_days: self.telemetry.disk.retention_days,
            },
            forward: telemetry::config::ForwardSettings {
                enabled: true,
                wrapper_address: self.telemetry.forward.wrapper_address.clone(),
                wrapper_port: self.telemetry.forward.wrapper_port,
                tenant_id: self.supervisor.tenant_id.clone(),
                batch_size: self.telemetry.forward.batch_size,
                flush_interval_seconds: self.telemetry.forward.flush_interval_seconds,
                retry_attempts: self.telemetry.forward.retry_attempts,
                compress: self.telemetry.forward.compress,
            },
            metrics: telemetry::config::MetricsSettings::default(),
        }
    }
}
