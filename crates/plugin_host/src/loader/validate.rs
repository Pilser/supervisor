//! Validate a loaded plugin library and its resolved metadata.

use libloading::Library;

use crate::loader::symbols;
use crate::{PluginInfo, SupervisorError};

/// Validate that `lib` exports the required FFI entry points and that its
/// `plugin_info` metadata is well-formed. Returns the resolved `PluginInfo`.
pub fn validate(lib: &Library) -> crate::Result<PluginInfo> {
    let info = unsafe { symbols::call_info(lib)? };

    // Required FFI surface: all three symbols must resolve.
    symbols::symbol_register(lib)?;
    symbols::symbol_unregister(lib)?;

    validate_info(&info)?;
    Ok(info)
}

/// Validate the shape of a plugin's `PluginInfo` metadata.
pub fn validate_info(info: &PluginInfo) -> crate::Result<()> {
    if info.name.trim().is_empty() {
        return Err(SupervisorError::InvalidInput(
            "plugin declares an empty name".into(),
        ));
    }
    Ok(())
}