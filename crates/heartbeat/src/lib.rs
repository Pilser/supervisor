//! `heartbeat` — tells the wrapper this VM is alive.
//!
//! Builds a payload (program status, resources, custom fields) on an
//! interval, sends it via the proxy client, and tracks failure / retry /
//! offline state plus uptime, latency and history stats.

#![forbid(unsafe_code)]

pub mod failure;
pub mod sender;
pub mod stats;
pub mod status;

pub use sender::{build_heartbeat, send_heartbeat, HeartbeatInterval, HeartbeatPayload};
pub use stats::{HeartbeatHistory, LatencyTracker, UptimeTracker};
pub use status::{programs_status, resources_status, CustomFields};
