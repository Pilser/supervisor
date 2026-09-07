//! Best-effort teardown of a loaded plugin library.

use libloading::Library;

use crate::loader::symbols;
use crate::{PluginContext, SupervisorError};

/// Invoke the plugin's `unregister` entry point (best-effort) so it can
/// release any runtime resources. Errors are reported but do not prevent the
/// caller from dropping the `Library` handle afterwards.
pub fn unregister_plugin(lib: &Library, ctx: &PluginContext) -> crate::Result<()> {
    match unsafe { symbols::call_unregister(lib, ctx) } {
        Ok(()) => Ok(()),
        Err(e) => Err(SupervisorError::Plugin(format!("unregister failed: {e}"))),
    }
}