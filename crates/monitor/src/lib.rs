//! `monitor` — watches the agent and memory binaries inside the VM.
//!
//! Registers programs to watch, polls process health and resource usage on a
//! schedule, detects crashes / freezes / leaks / zombies, recovers via
//! restart / kill / cleanup, and escalates to the wrapper when recovery fails.

#![forbid(unsafe_code)]

pub mod detect;
pub mod monitor_loop;
pub mod programs;
pub mod recover;
pub mod report;
pub mod watcher;

pub use monitor_loop::MonitorLoop;
pub use programs::{engram_spec, zerowrapper_spec, ProgramRegistry, ProgramSpec};
pub use report::{EventLog, MonitorStats, ProgramStatus};
pub use watcher::{health_check, is_running, read_usage, HealthStatus, ResourceUsage};
