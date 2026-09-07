//! Shut down the plugin host by unloading every registered plugin.

use telemetry::{Level, Telemetry};

use crate::PluginHost;

/// Unload all plugins, best-effort, then emit a shutdown event.
pub fn shutdown(host: &PluginHost, telemetry: &Telemetry) -> crate::Result<()> {
    for info in host.list() {
        let _ = host.unload(&info.name, telemetry);
    }
    telemetry
        .builder()
        .level(Level::Info)
        .kind("plugin_host.shutdown")
        .category("plugin_host")
        .message("plugin host shut down")
        .emit();
    Ok(())
}
