//! `commands` — receives and executes commands from the wrapper.
//!
//! Listens on TCP, authenticates the sender, parses commands, executes them
//! (built-in set plus plugin-registered ones) with timeouts, queues while
//! busy, and reports success / failure / progress back.

#![forbid(unsafe_code)]

pub mod built_in;
pub mod executor;
pub mod listener;
pub mod queue;
pub mod response;

pub use executor::{run_with_timeout, CommandResult, IncomingCommand};
pub use queue::{cancel_queued, CommandQueue, Priority, QueuedCommand};
