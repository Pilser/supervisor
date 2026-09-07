//! Simple retry-with-backoff helper.

use std::future::Future;

/// Try `op` up to `max_attempts` times with exponential backoff from `base`.
pub async fn retry_with_backoff<F, Fut, T>(
    max_attempts: u32,
    base: std::time::Duration,
    op: F,
) -> Result<T, anyhow::Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, anyhow::Error>>,
{
    let mut attempt = 0u32;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if attempt >= max_attempts {
                    return Err(e);
                }
                let delay = base * (1 << attempt.min(8));
                tokio::time::sleep(delay).await;
            }
        }
    }
}