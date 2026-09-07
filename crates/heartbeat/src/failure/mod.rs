//! Heartbeat failure handling: detect, retry, offline mode.

pub mod detect;
pub mod offline;
pub mod retry;

pub use detect::detect_failure;
pub use offline::OfflineMode;
pub use retry::RetryState;
