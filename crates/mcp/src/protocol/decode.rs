//! Deserialize raw wire bytes into a `Message`.

use anyhow::Context;

use super::json_rpc::JSONRPC_VERSION;
use super::message::Message;

/// Decode a single received line (trimmed) into a `Message`, validating the
/// mandatory `jsonrpc` version field.
pub fn decode(raw: &str) -> anyhow::Result<Message> {
    let msg: Message = serde_json::from_str::<Message>(raw)
        .with_context(|| "failed to parse JSON-RPC message")?;
    if msg.jsonrpc != JSONRPC_VERSION {
        anyhow::bail!("unsupported jsonrpc version: {:?}", msg.jsonrpc);
    }
    Ok(msg)
}