//! The batching worker that forwards events to the wrapper.

use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use super::client::ForwardClient;
use super::retry::retry_with_backoff;
use crate::models::Event;

/// Collects events and POSTs them in batches, flushing on size or interval.
pub struct Batcher;

impl Batcher {
    pub async fn run(
        client: ForwardClient,
        mut rx: mpsc::UnboundedReceiver<Event>,
        batch_size: usize,
        interval_seconds: u64,
        retries: u32,
    ) {
        let mut pending: Vec<Event> = Vec::with_capacity(batch_size);
        let mut tick = time::interval(Duration::from_secs(interval_seconds));
        loop {
            tokio::select! {
                maybe = rx.recv() => {
                    match maybe {
                        Some(ev) => {
                            pending.push(ev);
                            if pending.len() >= batch_size {
                                flush(&client, &mut pending, retries).await;
                            }
                        }
                        None => {
                            flush(&client, &mut pending, retries).await;
                            break;
                        }
                    }
                }
                _ = tick.tick() => {
                    flush(&client, &mut pending, retries).await;
                }
            }
        }
    }
}

async fn flush(client: &ForwardClient, pending: &mut Vec<Event>, retries: u32) {
    if pending.is_empty() {
        return;
    }
    let batch = std::mem::take(pending);
    let _ = retry_with_backoff(retries, Duration::from_millis(500), || async {
        client.send_batch(&batch).await
    })
    .await;
}