//! Heartbeat sender: interval config, payload, send.

pub mod interval;
pub mod payload;
pub mod send;

pub use interval::HeartbeatInterval;
pub use payload::{build_heartbeat, HeartbeatPayload};
pub use send::send_heartbeat;
