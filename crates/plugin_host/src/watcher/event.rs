//! File-system events reported by the folder watcher.

use std::path::PathBuf;

/// A change observed under the watched plugin folder.
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    /// A new plugin file appeared.
    Created(PathBuf),
    /// An existing plugin file changed.
    Modified(PathBuf),
    /// A plugin file was removed.
    Removed(PathBuf),
}

impl WatcherEvent {
    /// The path this event concerns.
    pub fn path(&self) -> &PathBuf {
        match self {
            WatcherEvent::Created(p) | WatcherEvent::Modified(p) | WatcherEvent::Removed(p) => p,
        }
    }

    /// Stable event name for telemetry labels.
    pub fn as_str(&self) -> &'static str {
        match self {
            WatcherEvent::Created(_) => "created",
            WatcherEvent::Modified(_) => "modified",
            WatcherEvent::Removed(_) => "removed",
        }
    }
}
