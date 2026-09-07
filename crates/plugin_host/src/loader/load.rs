//! `dlopen` a `.so` supervisor plugin and resolve its static metadata.

use std::path::Path;

use libloading::Library;

use crate::loader::validate::validate;
use crate::{PluginContext, PluginInfo, SupervisorError};

/// Open a shared object with `libloading`.
pub fn open(path: &Path) -> crate::Result<Library> {
    unsafe { Library::new(path) }.map_err(|e| {
        SupervisorError::Plugin(format!("failed to open `{}`: {e}", path.display()))
    })
}

/// Open `path`, validate its FFI surface and resolve its `PluginInfo`.
///
/// On success the caller owns the `Library` handle and must keep it alive for
/// as long as the plugin is registered.
pub fn load_library(path: &Path) -> crate::Result<(Library, PluginInfo)> {
    let lib = open(path)?;
    let info = validate(&lib)?;
    Ok((lib, info))
}

/// Invoke the plugin's `register` entry point so it can declare its runtime
/// contract. Errors are reported to the caller.
pub fn register_plugin(lib: &Library, ctx: &PluginContext) -> crate::Result<()> {
    match unsafe { crate::loader::symbols::call_register(lib, ctx) } {
        Ok(()) => Ok(()),
        Err(e) => Err(SupervisorError::Plugin(format!("register failed: {e}"))),
    }
}