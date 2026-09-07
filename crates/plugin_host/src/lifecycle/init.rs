//! Initialize the plugin host by loading every plugin in its folder.

use telemetry::Telemetry;

use crate::PluginHost;

/// Load every `.so` under the host folder.
pub fn init(host: &PluginHost, telemetry: &Telemetry) -> crate::Result<()> {
    host.init(telemetry)
}
