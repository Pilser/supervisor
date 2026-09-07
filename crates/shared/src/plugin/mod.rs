//! Plugin contract for supervisor plugins.

pub mod api;
pub mod context;
pub mod hooks;
pub mod info;
#[path = "trait.rs"]
pub mod plugin_trait;

pub use api::PluginApi;
pub use context::PluginContext;
pub use hooks::HookPoint;
pub use info::{PluginInfo, PluginType, PluginVersion};
pub use plugin_trait::Plugin;