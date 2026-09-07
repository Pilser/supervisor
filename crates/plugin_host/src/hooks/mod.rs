//! Hook dispatch for supervisor plugins.
//!
//! Plugins subscribe to `core::HookPoint` variants (OnPluginLoaded,
//! OnProcessCrashed, OnHeartbeatSent, …) and the host routes events to every
//! subscriber, ordered by priority then insertion order.

pub mod dispatch;
pub mod priority;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::Result;

pub use core::HookPoint;

/// A plugin hook handler: receives the deserialized event payload.
pub type HookCallback = dyn Fn(&serde_json::Value) -> Result<()> + Send + Sync;

/// One registration of a handler against a hook point.
#[derive(Clone)]
pub struct HookEntry {
    pub name: String,
    pub priority: i32,
    pub order: u64,
    pub callback: Arc<HookCallback>,
}

impl HookEntry {
    pub fn new(name: impl Into<String>, priority: i32, order: u64, callback: Arc<HookCallback>) -> Self {
        Self {
            name: name.into(),
            priority,
            order,
            callback,
        }
    }
}

/// Routes events to every plugin subscribed to a `HookPoint`.
pub struct HookDispatcher {
    subscriptions: Mutex<HashMap<HookPoint, Vec<HookEntry>>>,
    counter: AtomicU64,
}

impl HookDispatcher {
    /// A dispatcher with no subscriptions.
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(0),
        }
    }

    fn next_order(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Subscribe `name` to `point` at the default priority.
    pub fn subscribe(
        &self,
        point: HookPoint,
        name: impl Into<String>,
        callback: Arc<HookCallback>,
    ) -> Result<()> {
        self.subscribe_priority(point, name, callback, priority::DEFAULT_PRIORITY)
    }

    /// Subscribe `name` to `point` at `priority` (lower runs first).
    pub fn subscribe_priority(
        &self,
        point: HookPoint,
        name: impl Into<String>,
        callback: Arc<HookCallback>,
        priority: i32,
    ) -> Result<()> {
        let name = name.into();
        let entry = HookEntry::new(name, priority, self.next_order(), callback);
        let mut map = self.subscriptions.lock().unwrap_or_else(|p| p.into_inner());
        let list = map.entry(point).or_default();
        // Remove an existing registration for the same plugin so reloads do
        // not accumulate stale entries.
        list.retain(|e| e.name != entry.name);
        if let Some(insert_at) = list.iter().position(|e| priority::order(e, &entry) == std::cmp::Ordering::Greater)
        {
            list.insert(insert_at, entry);
        } else {
            list.push(entry);
        }
        Ok(())
    }

    /// Remove every subscription of `name` to `point`.
    pub fn unsubscribe(&self, point: HookPoint, name: &str) -> Result<()> {
        let mut map = self.subscriptions.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(list) = map.get_mut(&point) {
            list.retain(|e| e.name != name);
        }
        Ok(())
    }

    /// Names subscribed to `point` in dispatch (priority, then insertion) order.
    pub fn subscribed(&self, point: HookPoint) -> Vec<String> {
        let map = self.subscriptions.lock().unwrap_or_else(|p| p.into_inner());
        map.get(&point)
            .map(|list| list.iter().map(|e| e.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Return owned snapshot of entries for `point` so dispatch never holds
    /// the internal lock while invoking callbacks.
    pub(crate) fn snapshot(&self, point: HookPoint) -> Vec<HookEntry> {
        let map = self.subscriptions.lock().unwrap_or_else(|p| p.into_inner());
        map.get(&point)
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for HookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for HookDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookDispatcher").finish_non_exhaustive()
    }
}