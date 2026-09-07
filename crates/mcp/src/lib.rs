//! `mcp` — MCP-compatible tool server exposed to the in-VM agent.
//!
//! Re-exports every subsystem so plugins and the daemon get a stable surface:
//! `protocol` (JSON-RPC wire), `tools` (registry/builder/handler/validate),
//! `built_in` (built-in tool families), `handler` (wrapper-proxying handlers),
//! `permissions` (scopes + enforcement) and `server` (transports + dispatch).

pub mod built_in;
pub mod handler;
pub mod permissions;
pub mod protocol;
pub mod server;
pub mod tools;

pub use built_in::{register_all, register_with_proxy};
pub use handler::{
    DataProxy, NoopProxy, ProxyFuture, SharedProxy, noop_proxy, proxy_call, proxy_request,
};
pub use permissions::{Decision, Scope, can_call, enforce, enforce_result, enforce_tenant};
pub use protocol::{
    INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, JSONRPC_VERSION, METHOD_NOT_FOUND,
    PARSE_ERROR, JsonRpcError, Message, decode, encode, id_of, response_err, response_ok,
};
pub use server::{McpServer, Shutdown, ShutdownSignal, handle_message, initialize_result};
pub use tools::{BoxFuture, Tool, ToolBuilder, ToolHandler, ToolRegistry};
pub use tools::validate::validate_params;
