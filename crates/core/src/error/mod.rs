//! Central error type used across the supervisor.

mod supervisor_error;

pub use supervisor_error::{Result, SupervisorError, SupervisorErrorKind};