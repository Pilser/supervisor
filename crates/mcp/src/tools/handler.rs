//! The `ToolHandler` trait every tool implementation must satisfy.

use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

/// A boxed future that outlives the call.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Handles a single tool invocation. Implementors are `Send + Sync` and can be
/// wrapped in an `Arc` to share across the registry.
pub trait ToolHandler: Send + Sync {
    /// Execute the tool. `params` has already been validated against the
    /// tool's schema by the registry.
    fn call(&self, params: Value) -> BoxFuture<'static, anyhow::Result<Value>>;
}