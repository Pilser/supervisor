//! Supervisor token storage for wrapper authentication.

use std::sync::Mutex;

/// Holds the current supervisor bearer token, mutable so auth can rotate it.
#[derive(Debug)]
pub struct TokenStore {
    token: Mutex<String>,
}

impl TokenStore {
    /// Create a store holding `token`.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Mutex::new(token.into()),
        }
    }

    /// Clone the current token.
    pub fn get(&self) -> String {
        self.token.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Replace the current token.
    pub fn set(&self, token: &str) {
        *self.token.lock().unwrap_or_else(|p| p.into_inner()) = token.to_string();
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new(String::new())
    }
}
