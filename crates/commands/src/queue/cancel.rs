//! Cancel queued commands.

use super::store::CommandQueue;

/// Remove a queued command by id. Returns true when something was removed.
pub fn cancel_queued(queue: &mut CommandQueue, id: &str) -> bool {
    let before = queue.len();
    let mut kept = CommandQueue::new();
    while let Some(q) = queue.dequeue() {
        if q.command.id != id {
            kept.enqueue(q.command, q.priority);
        }
    }
    while let Some(q) = kept.dequeue() {
        queue.enqueue(q.command, q.priority);
    }
    queue.len() < before
}
