//! Cache time-to-live management.

use std::time::Duration;

use crate::request::RequestType;

/// How long cached wrapper responses stay valid.
#[derive(Debug, Clone, Copy)]
pub struct CacheTtl {
    /// Default TTL applied to cacheable reads.
    pub default: Duration,
}

impl Default for CacheTtl {
    fn default() -> Self {
        Self {
            default: Duration::from_secs(60),
        }
    }
}

impl CacheTtl {
    /// Build a TTL with the given default duration.
    pub fn new(default: Duration) -> Self {
        Self { default }
    }

    /// TTL for the given request type. Only reads are cached; writes return
    /// a zero TTL so callers never store them.
    pub fn for_request(&self, rtype: RequestType) -> Duration {
        if rtype.is_read() {
            self.default
        } else {
            Duration::ZERO
        }
    }
}
