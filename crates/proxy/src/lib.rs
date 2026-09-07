//! `proxy` — forwards data requests from the supervisor (and its tools) to the
//! WRAPPER over TCP (NDJSON).
//!
//! The wrapper owns the tenant databases; the supervisor is the only gateway
//! that talks to it. This crate implements that gateway: a typed request
//! pipeline (`RequestType` / `RequestBuilder`) that packages every call into the
//! wrapper envelope `{ "type", "tenant_id", "timestamp", "payload" }`, embeds
//! the supervisor token into `payload`, sends it over a TCP connection as one
//! NDJSON line, parses and transforms the wrapper response line, and falls back
//! to an offline queue when the wrapper is unreachable. Local caching reduces
//! wrapper round-trips, and auth handling keeps tokens refreshed and rotated.

#![forbid(unsafe_code)]

pub mod auth;
pub mod cache;
pub mod client;
pub mod queue;
pub mod request;
pub mod response;

pub use auth::{refresh_token, TokenRefresh, TokenRotation, TokenRotator, TokenStore};
pub use cache::invalidate::{invalidate_key, CacheInvalidation};
pub use cache::{CacheEntry, CacheKey, CacheStore, CacheTtl};
pub use client::{ConnectionPool, PooledConn, ProxyClient, RetryPolicy, TlsConfig};
pub use queue::{flush_queue, OfflineMessage, OfflineQueue, Priority, PriorityQueue};
pub use request::{Request, RequestBuilder, RequestType};
pub use response::{ProxyError, Response};