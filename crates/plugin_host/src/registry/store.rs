//! Storage operations over the shared plugin registry map.

use std::collections::HashMap;

use crate::{HandledPlugin, SupervisorError};

/// Insert `plugin` under its own name, failing `InvalidInput` if a plugin
/// with that name is already present.
pub fn insert_unless_present(
    map: &mut HashMap<String, HandledPlugin>,
    plugin: HandledPlugin,
) -> crate::Result<String> {
    if plugin.name.trim().is_empty() {
        return Err(SupervisorError::InvalidInput(
            "cannot store a plugin with an empty name".into(),
        ));
    }
    let name = plugin.name.clone();
    if map.contains_key(&name) {
        return Err(SupervisorError::InvalidInput(format!(
            "plugin `{name}` is already registered"
        )));
    }
    map.insert(name.clone(), plugin);
    Ok(name)
}

/// Register a plugin by name, replacing any existing registration. Returns
/// true when an existing entry was replaced (a reload).
pub fn register_file_under_name(
    map: &mut HashMap<String, HandledPlugin>,
    plugin: HandledPlugin,
) -> bool {
    let replaced = map.remove(&plugin.name).is_some();
    let name = plugin.name.clone();
    map.insert(name, plugin);
    replaced
}

/// Fetch the plugin registered under `name`, if any.
pub fn by_name<'a>(
    map: &'a HashMap<String, HandledPlugin>,
    name: &str,
) -> Option<&'a HandledPlugin> {
    map.get(name)
}