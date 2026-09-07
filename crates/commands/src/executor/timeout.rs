//! Command timeout handling.

use core::{Result, SupervisorError};

/// Run `fut` with a timeout; expiry maps to [`SupervisorError::Timeout`].
pub async fn run_with_timeout<T>(
    name: &str,
    timeout_secs: u64,
    fut: impl std::future::Future<Output = Result<T>>,
) -> Result<T> {
    tokio::time::timeout(std::time::Duration::from_secs(timeout_secs.max(1)), fut)
        .await
        .map_err(|_| SupervisorError::Timeout(format!("command {name} timed out")))?
}
