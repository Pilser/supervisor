//! Offline mode when the wrapper is unreachable.

use serde::{Deserialize, Serialize};

/// Whether heartbeats are currently being queued instead of sent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OfflineMode {
    pub offline: bool,
}

impl OfflineMode {
    pub fn enter(&mut self, telemetry: &telemetry::Telemetry) {
        self.offline = true;
        telemetry
            .builder()
            .level(telemetry::Level::Warn)
            .kind("heartbeat.offline")
            .category("heartbeat")
            .message("entering offline mode")
            .emit();
    }

    pub fn exit(&mut self, telemetry: &telemetry::Telemetry) {
        self.offline = false;
        telemetry
            .builder()
            .level(telemetry::Level::Info)
            .kind("heartbeat.online")
            .category("heartbeat")
            .message("back online")
            .emit();
    }

    pub fn is_offline(&self) -> bool {
        self.offline
    }
}
