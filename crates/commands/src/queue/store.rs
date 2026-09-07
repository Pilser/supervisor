//! Queue commands while busy.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::priority::Priority;
use crate::listener::parse::IncomingCommand;

/// A command waiting for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedCommand {
    pub command: IncomingCommand,
    pub priority: Priority,
}

/// FIFO queue with priority insertion.
#[derive(Debug, Default)]
pub struct CommandQueue {
    queue: VecDeque<QueuedCommand>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, cmd: IncomingCommand, priority: Priority) {
        let queued = QueuedCommand { command: cmd, priority };
        if priority == Priority::High {
            self.queue.push_front(queued);
        } else {
            self.queue.push_back(queued);
        }
    }

    pub fn dequeue(&mut self) -> Option<QueuedCommand> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
