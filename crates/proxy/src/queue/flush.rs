//! Flush queued requests when reconnected.

use super::OfflineQueue;
use crate::client::ProxyClient;

/// Drain `queue`, re-sending every parked envelope through `client`.
///
/// Returns the number of messages successfully re-sent. Messages that fail
/// are dropped so a poisoned envelope cannot block the queue forever.
pub async fn flush_queue(queue: &OfflineQueue, client: &ProxyClient) -> usize {
    let mut flushed = 0usize;
    while let Some(message) = queue.pop().await {
        match client.send_json(message.payload).await {
            Ok(_) => flushed += 1,
            Err(_) => {
                // Drop the failed message; the caller already emitted the
                // unreachable telemetry on the original failure path.
            }
        }
    }
    flushed
}
