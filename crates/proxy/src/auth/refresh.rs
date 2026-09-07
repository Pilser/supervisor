//! Refresh the supervisor auth token.

use chrono::{DateTime, Utc};

use super::TokenStore;

/// Record of a token refresh.
#[derive(Debug, Clone)]
pub struct TokenRefresh {
    /// The refreshed token value.
    pub token: String,
    /// When the refresh happened.
    pub refreshed_at: DateTime<Utc>,
}

impl TokenRefresh {
    /// Describe a refresh that just produced `token`.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            refreshed_at: Utc::now(),
        }
    }
}

/// Refresh the token held by `store`, persisting the new value and returning
/// the refresh record. This stub keeps the current value when no rotation
/// backend is configured.
pub fn refresh_token(store: &TokenStore) -> anyhow::Result<TokenRefresh> {
    let current = store.get();
    let refreshed = TokenRefresh::new(current.clone());
    store.set(&current);
    Ok(refreshed)
}
