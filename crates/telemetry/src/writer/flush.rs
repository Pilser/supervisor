//! Periodic flush scheduling.

use tokio::time::{self, Duration};

pub async fn periodic_flush(
    interval_seconds: u64,
    mut flush: impl FnMut() -> anyhow::Result<()> + Send + 'static,
) -> anyhow::Result<()> {
    let mut interval = time::interval(Duration::from_secs(interval_seconds.max(1)));
    loop {
        interval.tick().await;
        if let Err(e) = flush() {
            let _ = e;
        }
    }
}