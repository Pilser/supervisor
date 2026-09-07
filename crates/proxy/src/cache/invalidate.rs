//! Cache invalidation on writes.

use super::{CacheKey, CacheStore};

/// Record of a cache invalidation.
#[derive(Debug, Clone)]
pub struct CacheInvalidation {
    /// The invalidated key index.
    pub key: String,
    /// Whether an entry was actually removed.
    pub removed: bool,
}

impl CacheInvalidation {
    /// Describe the invalidation of `key`.
    pub fn new(key: &CacheKey, removed: bool) -> Self {
        Self {
            key: key.cache_index(),
            removed,
        }
    }
}

/// Remove `key` from `store`, returning the invalidation record.
pub fn invalidate_key(store: &mut CacheStore, key: &CacheKey) -> CacheInvalidation {
    let removed = store.invalidate(key);
    CacheInvalidation::new(key, removed)
}
