//! JSON-RPC 2.0 / Model Context Protocol wire protocol for the `mcp` crate.

pub mod decode;
pub mod encode;
pub mod json_rpc;
pub mod message;

pub use decode::decode;
pub use encode::encode;
pub use json_rpc::{
    INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, JSONRPC_VERSION, METHOD_NOT_FOUND,
    PARSE_ERROR, id_of, response_err, response_ok,
};
pub use message::{JsonRpcError, Message};