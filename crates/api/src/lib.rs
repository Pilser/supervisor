//! `api` — stable plugin-facing surface over every supervisor subsystem.
//!
//! Re-exports handles for MCP, proxy, monitor, heartbeat, commands,
//! deployment and plugins, plus a shared context, event bus and macros.
//! Each module holds lightweight handles (config + telemetry) so plugins get
//! a stable API even while subsystems evolve.

#![forbid(unsafe_code)]

pub mod commands;
pub mod context;
pub mod deployment;
pub mod events;
pub mod heartbeat;
pub mod macros;
pub mod mcp;
pub mod monitor;
pub mod plugin;
pub mod proxy;

pub use commands::CommandsApi;
pub use context::{SupervisorContext, SupervisorContextBuilder};
pub use deployment::DeploymentApi;
pub use heartbeat::HeartbeatApi;
pub use mcp::McpApi;
pub use monitor::MonitorApi;
pub use plugin::PluginApi;
pub use proxy::ProxyApi;
