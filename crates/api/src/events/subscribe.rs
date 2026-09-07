//! Subscribe to API-level events.

use serde::{Deserialize, Serialize};

/// A filter over API event kinds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventSubscription {
    pub kinds: Vec<String>,
}

impl EventSubscription {
    pub fn new(kinds: Vec<String>) -> Self {
        Self { kinds }
    }

    /// True when `kind` matches the subscription (empty matches all).
    pub fn matches(&self, kind: &str) -> bool {
        self.kinds.is_empty() || self.kinds.iter().any(|k| k == kind)
    }
}
