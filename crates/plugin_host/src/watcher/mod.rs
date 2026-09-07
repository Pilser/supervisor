//! Watch the `plugins/` folder for changes.

pub mod event;
pub mod folder;
pub mod scan;

pub use event::WatcherEvent;
pub use folder::FolderWatcher;
pub use scan::plugin_files;
