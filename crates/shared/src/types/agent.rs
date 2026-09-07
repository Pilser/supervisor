//! Agent process state, observed by the monitor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    #[default]
    Unknown,
    Starting,
    Running,
    Healthy,
    Frozen,
    Crashed,
    Stopped,
}

/// Snapshot of a monitored agent process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub pid: Option<u32>,
    pub status: AgentStatus,
    pub uptime_seconds: u64,
    pub healthy: bool,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub last_heartbeat: Option<String>,
}