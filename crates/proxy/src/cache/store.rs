//! Cached wrapper responses keyed by tenant, type and operation.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::CacheTtl;
use crate::request::RequestType;

/// Key identifying one cached wrapper response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Tenant scope, empty when the request carries no tenant.
    pub tenant: String,
    /// Wire request type (`db_read`, ...).
    pub rtype: String,
    /// Wrapper operation name.
    pub operation: String,
}

impl CacheKey {
    /// Build a scoped key for `tenant` + `rtype` + `operation`.
    pub fn scoped(tenant: &str, rtype: &RequestType, operation: &str) -> Self {
        Self {
            tenant: tenant.to_string(),
            rtype: rtype.as_str().to_string(),
            operation: operation.to_string(),
        }
    }

    /// Stable string index used for map storage and telemetry labels.
    pub fn cache_index(&self) -> String {
        format!("{}:{}:{}", self.tenant, self.rtype, self.operation)
    }
}

/// One cached value with its expiry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached payload.
    pub data: Value,
    /// When the entry expires, if ever.
    pub expires_at: Option<Instant>,
}

impl CacheEntry {
    /// Whether the entry is expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|t| Instant::now() >= t).unwrap_or(false)
    }
}

/// In-memory store of wrapper responses.
#[derive(Debug)]
pub struct CacheStore {
    entries: HashMap<String, CacheEntry>,
    ttl: CacheTtl,
}

impl CacheStore {
    /// Create an empty store using `ttl` as the default TTL.
    pub fn new(ttl: CacheTtl) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    /// The TTL configuration of this store.
    pub fn ttl(&self) -> CacheTtl {
        self.ttl
    }

    /// Look up a cached value, returning `None` on miss or expiry.
    pub fn get(&mut self, key: &CacheKey) -> Option<Value> {
        let index = key.cache_index();
        match self.entries.get(&index) {
            Some(entry) if !entry.is_expired() => Some(entry.data.clone()),
            Some(_) => {
                self.entries.remove(&index);
                None
            }
            None => None,
        }
    }

    /// Store `value` under `key` with an optional TTL override.
    pub fn put(&mut self, key: CacheKey, value: Value, ttl: Option<Duration>) {
        let expires_at = ttl.or(Some(self.ttl.default)).map(|d| Instant::now() + d);
        self.entries.insert(
            key.cache_index(),
            CacheEntry {
                data: value,
                expires_at,
            },
        );
    }

    /// Remove the entry for `key`. Returns true if one existed.
    pub fn invalidate(&mut self, key: &CacheKey) -> bool {
        self.entries.remove(&key.cache_index()).is_some()
    }

    /// Number of live entries (including possibly-expired ones not yet evicted).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drop every entry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new(CacheTtl::default())
    }
}
