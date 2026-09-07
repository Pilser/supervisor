//! Access control for the MCP surface: `Scope` plus enforcement helpers.

pub mod enforce;
pub mod scope;

pub use enforce::{Decision, can_call, enforce, enforce_result, enforce_tenant};
pub use scope::Scope;