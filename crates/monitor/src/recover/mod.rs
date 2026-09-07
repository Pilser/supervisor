//! Recovery actions: restart, kill, cleanup, escalate.

pub mod cleanup;
pub mod escalate;
pub mod kill;
pub mod restart;

pub use cleanup::cleanup_after_crash;
pub use escalate::escalate_to_wrapper;
pub use kill::kill_process;
pub use restart::restart_program;
