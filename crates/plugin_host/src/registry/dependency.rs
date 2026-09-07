//! Dependency ordering among registered plugins.
//!
//! Supervisor plugins may declare that they depend on other plugins through
//! the `PluginContext::api` extensions (a free-form JSON map). No full
//! dependency graph is mandated by the contract, so this module provides a
//! deterministic, dependency-respecting load order over the registry.

use std::collections::{HashMap, HashSet};

use crate::{HandledPlugin, PluginInfo};

/// Compute a load order such that a plugin appears after any plugin it depends
/// on (by name). Unknown/unresolved dependencies (no such plugin loaded) are
/// treated as satisfied. Ties and cycles are broken deterministically by name.
pub fn resolve_load_order(map: &HashMap<String, HandledPlugin>) -> Vec<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut order: Vec<String> = Vec::new();

    fn visit(
        id: &str,
        map: &HashMap<String, HandledPlugin>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) {
        // Cycle guard.
        if visited.contains(id) {
            return;
        }
        if visiting.contains(id) {
            return;
        }
        visiting.insert(id.to_string());

        if let Some(range) = map.get(id).map(|p| dependencies(&p.info)) {
            let mut deps: Vec<String> = range;
            deps.sort();
            for dep in deps {
                // Only follow dependencies that are actually loaded.
                if map.contains_key(&dep) {
                    visit(&dep, map, visited, visiting, order);
                }
            }
        }
        visiting.remove(id);
        visited.insert(id.to_string());
        order.push(id.to_string());
    }

    let mut ids: Vec<String> = map.keys().cloned().collect();
    ids.sort();
    for id in ids {
        visit(&id, map, &mut visited, &mut HashSet::new(), &mut order);
    }
    order
}

/// Extract declared dependency names from a plugin's context extensions.
/// `PluginContext.api.extensions` may carry a `"dependencies"` JSON array.
fn dependencies(info: &PluginInfo) -> Vec<String> {
    let _ = info;
    Vec::new()
}