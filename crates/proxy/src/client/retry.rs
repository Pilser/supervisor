//! Retry helper for wrapper requests.
//!
//! [`RetryPolicy`] applies exponential backoff to a fallible async closure,
//! which is useful for transient connection failures and wrapper hiccups.

use std::future::Future;
use std::time::Duration;

use tokio::time::sleep;

/// Retry configuration: max attempts and exponential backoff bounds.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Max number of attempts (including the first).
    pub max_attempts: u32,
    /// Base delay (ms) — doubled each attempt.
    pub base_delay_ms: u64,
    /// Upper bound on any single delay (ms).
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 2_000,
        }
    }
}

impl RetryPolicy {
    /// A retry policy with `max_attempts` attempts.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            ..Default::default()
        }
    }

    /// Delay for the given 0-indexed attempt (exponential backoff).
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let backoff = self.base_delay_ms.saturating_mul(1u64 << attempt.min(6));
        Duration::from_millis(backoff.min(self.max_delay_ms))
    }

    /// Run `f` up to `max_attempts` times, sleeping between retries.
    pub async fn retry<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Clone,
    {
        let mut last_err: Option<E> = None;
        for attempt in 0..self.max_attempts {
            match f().await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last_err = Some(e);
                    if attempt + 1 < self.max_attempts {
                        sleep(self.delay_for(attempt)).await;
                    }
                }
            }
        }
        Err(last_err.expect("retry policy has at least one attempt"))
    }
}