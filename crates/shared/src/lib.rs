//! `core` — shared types, plugin contract and utilities for the supervisor.
//!
//! This is the foundation crate for the supervisor daemon. It defines the
//! domain vocabulary (tenant context, memory, history, projects, skills,
//! reports, categories, deployment) and the plugin contract every other crate
//! depends on. It has no I/O subsystem knowledge by design.

#![forbid(unsafe_code)]

pub mod error;
pub mod plugin;
pub mod types;
pub mod util;

pub use error::{Result, SupervisorError, SupervisorErrorKind};
pub use plugin::{
    HookPoint, Plugin, PluginApi, PluginContext, PluginInfo, PluginType, PluginVersion,
};
pub use types::*;
pub use util::{now_rfc3339, now_unix_millis};