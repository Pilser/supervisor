//! Buffer wrapper requests issued while the wrapper is unreachable.

use std::collections::VecDeque;

use serde_json::Value;
use tokio::sync::Mutex;

use super::Priority;

/// One request parked in the offline queue.
#[derive(Debug, Clone)]
pub struct OfflineMessage {
    /// The wrapper envelope that could not be sent.
    pub payload: Value,
    /// Priority used when flushing.
    pub priority: Priority,
    /// How many flush attempts have been made.
    pub attempts: u32,
}

impl OfflineMessage {
    /// Wrap `payload` at the default priority.
    pub fn new(payload: Value) -> Self {
        Self {
            payload,
            priority: Priority::Normal,
            attempts: 0,
        }
    }

    /// Wrap `payload` with an explicit `priority`.
    pub fn with_priority(payload: Value, priority: Priority) -> Self {
        Self {
            payload,
            priority,
            attempts: 0,
        }
    }
}

/// FIFO buffer of requests to retry once the wrapper is reachable again.
#[derive(Debug, Default)]
pub struct OfflineQueue {
    messages: Mutex<VecDeque<OfflineMessage>>,
}

impl OfflineQueue {
    /// Create an empty queue.
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::new()),
        }
    }

    /// Enqueue a raw wrapper envelope.
    pub async fn enqueue(&self, payload: Value) {
        self.messages
            .lock()
            .await
            .push_back(OfflineMessage::new(payload));
    }

    /// Enqueue a full offline message (with priority metadata).
    pub async fn enqueue_message(&self, message: OfflineMessage) {
        self.messages.lock().await.push_back(message);
    }

    /// Pop the oldest message, if any.
    pub async fn pop(&self) -> Option<OfflineMessage> {
        self.messages.lock().await.pop_front()
    }

    /// Number of queued messages.
    pub async fn len(&self) -> usize {
        self.messages.lock().await.len()
    }

    /// Whether the queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.messages.lock().await.is_empty()
    }
}
