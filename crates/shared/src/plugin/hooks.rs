//! Hook points a supervisor plugin can subscribe to.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookPoint {
    OnAgentConnected,
    OnAgentDisconnected,
    OnToolCalled,
    OnToolResponse,
    OnToolError,
    OnProcessCrashed,
    OnProcessFrozen,
    OnProcessRestarted,
    OnResourceThresholdExceeded,
    OnHealthCheckFailed,
    OnRecoveryStarted,
    OnRecoveryFailed,
    OnHeartbeatSent,
    OnHeartbeatFailed,
    OnHeartbeatRecovered,
    OnCommandReceived,
    OnCommandExecuted,
    OnCommandFailed,
    OnDeploymentReceived,
    OnDeploymentStarted,
    OnDeploymentCompleted,
    OnDeploymentFailed,
    OnPluginLoaded,
    OnPluginUnloaded,
    OnPluginReloaded,
    OnCategoryChanged,
    OnRollbackStarted,
    OnCacheHit,
    OnCacheMiss,
    OnEventEmitted,
    OnCustom,
}

impl HookPoint {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookPoint::OnAgentConnected => "agent.connected",
            HookPoint::OnAgentDisconnected => "agent.disconnected",
            HookPoint::OnToolCalled => "tool.called",
            HookPoint::OnToolResponse => "tool.response",
            HookPoint::OnToolError => "tool.error",
            HookPoint::OnProcessCrashed => "monitor.process_crashed",
            HookPoint::OnProcessFrozen => "monitor.process_frozen",
            HookPoint::OnProcessRestarted => "monitor.process_restarted",
            HookPoint::OnResourceThresholdExceeded => "monitor.resource_threshold",
            HookPoint::OnHealthCheckFailed => "monitor.health_check_failed",
            HookPoint::OnRecoveryStarted => "monitor.recovery_started",
            HookPoint::OnRecoveryFailed => "monitor.recovery_failed",
            HookPoint::OnHeartbeatSent => "heartbeat.sent",
            HookPoint::OnHeartbeatFailed => "heartbeat.failed",
            HookPoint::OnHeartbeatRecovered => "heartbeat.recovered",
            HookPoint::OnCommandReceived => "command.received",
            HookPoint::OnCommandExecuted => "command.executed",
            HookPoint::OnCommandFailed => "command.failed",
            HookPoint::OnDeploymentReceived => "deployment.received",
            HookPoint::OnDeploymentStarted => "deployment.started",
            HookPoint::OnDeploymentCompleted => "deployment.completed",
            HookPoint::OnDeploymentFailed => "deployment.failed",
            HookPoint::OnPluginLoaded => "plugin.loaded",
            HookPoint::OnPluginUnloaded => "plugin.unloaded",
            HookPoint::OnPluginReloaded => "plugin.reloaded",
            HookPoint::OnCategoryChanged => "category.changed",
            HookPoint::OnRollbackStarted => "rollback.started",
            HookPoint::OnCacheHit => "cache.hit",
            HookPoint::OnCacheMiss => "cache.miss",
            HookPoint::OnEventEmitted => "telemetry.event_emitted",
            HookPoint::OnCustom => "custom",
        }
    }
}