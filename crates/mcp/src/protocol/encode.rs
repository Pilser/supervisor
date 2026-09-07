//! Serialize JSON-RPC messages to the wire.

use crate::protocol::message::Message;

/// Encode a `Message` as a single-line JSON string suitable for streaming over
/// a socket.
pub fn encode(message: &Message) -> anyhow::Result<String> {
    serde_json::to_string(message).map_err(Into::into)
}