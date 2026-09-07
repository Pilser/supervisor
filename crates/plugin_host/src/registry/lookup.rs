//! Lookup helpers over the shared plugin registry map.

use std::collections::HashMap;

use crate::{HandledPlugin, PluginInfo};

/// Clone the metadata of the plugin registered under `name`, if present.
pub fn by_name(map: &HashMap<String, HandledPlugin>, name: &str) -> Option<PluginInfo> {
    map.get(name).map(|p| p.info.clone())
}

/// True if `name` is currently registered.
pub fn contained(map: &HashMap<String, HandledPlugin>, name: &str) -> bool {
    map.contains_key(name)
}

/// Registered plugin names.
pub fn names(map: &HashMap<String, HandledPlugin>) -> Vec<String> {
    map.keys().cloned().collect()
}

/// Clone metadata for every registered plugin.
pub fn infos(map: &HashMap<String, HandledPlugin>) -> Vec<PluginInfo> {
    map.values().map(|p| p.info.clone()).collect()
}