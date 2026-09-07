//! Reload a single plugin by name.

use telemetry::Telemetry;

use crate::PluginHost;

/// Reload the plugin named `name` from disk.
pub fn reload_plugin(
    host: &PluginHost,
    name: &str,
    telemetry: &Telemetry,
) -> crate::Result<()> {
    host.reload(name, telemetry)
}
