//! Prioritize critical wrapper requests.

use std::cmp::Ordering;

/// Priority of a queued request. Lower-priority variants sort first only when
/// explicitly ordered; critical requests should be flushed ahead of the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl Priority {
    /// Numeric rank, higher means more urgent.
    pub fn rank(&self) -> u8 {
        match self {
            Priority::Low => 0,
            Priority::Normal => 1,
            Priority::High => 2,
            Priority::Critical => 3,
        }
    }

    /// Wire string for telemetry labels.
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

/// An ordered queue of priorities used to decide flush order.
#[derive(Debug, Default)]
pub struct PriorityQueue {
    items: Vec<Priority>,
}

impl PriorityQueue {
    /// Create an empty priority queue.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Push a priority level.
    pub fn push(&mut self, priority: Priority) {
        self.items.push(priority);
        self.items.sort();
    }

    /// Pop the highest priority, if any.
    pub fn pop(&mut self) -> Option<Priority> {
        self.items.pop()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Number of queued priorities.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}
