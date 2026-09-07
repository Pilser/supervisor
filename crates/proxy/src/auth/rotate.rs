//! Handle supervisor token rotation.

use chrono::{DateTime, Utc};

use super::TokenStore;

/// Record of a token rotation.
#[derive(Debug, Clone)]
pub struct TokenRotation {
    /// The previous token value.
    pub previous: String,
    /// The new token value.
    pub current: String,
    /// When the rotation happened.
    pub rotated_at: DateTime<Utc>,
}

/// Rotates the supervisor token held by a [`TokenStore`].
#[derive(Debug)]
pub struct TokenRotator {
    store: std::sync::Arc<TokenStore>,
}

impl TokenRotator {
    /// Build a rotator over `store`.
    pub fn new(store: std::sync::Arc<TokenStore>) -> Self {
        Self { store }
    }

    /// Rotate to `next`, returning the rotation record.
    pub fn rotate(&self, next: &str) -> TokenRotation {
        let previous = self.store.get();
        self.store.set(next);
        TokenRotation {
            previous,
            current: next.to_string(),
            rotated_at: Utc::now(),
        }
    }

    /// Clone the current token.
    pub fn current(&self) -> String {
        self.store.get()
    }
}
