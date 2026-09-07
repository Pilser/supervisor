//! UNIX-socket transport for the MCP server.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UnixStream;

use crate::protocol::{decode, encode};
use crate::server::handshake;
use crate::server::McpServer;

/// Serve a single UNIX-socket connection: read newline-delimited JSON-RPC
/// messages, dispatch them, and write responses back.
///
/// A `supervisor.connected` event is emitted on arrival and a
/// `supervisor.disconnected` event once the loop ends.
pub async fn handle_stream(server: &McpServer, stream: UnixStream) {
    server.emit_connected();

    let (reader, writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let result: anyhow::Result<()> = async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                break; // remote closed the connection
            }
            let raw = line.trim();
            if raw.is_empty() {
                continue;
            }
            let msg = decode(raw)?;
            if let Some(resp) = handshake::handle_message(server, msg).await? {
                let out = encode(&resp)?;
                writer.write_all(out.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }
    .await;

    if let Err(err) = result {
        server.emit_log_error(&err.to_string());
    }
    server.emit_disconnected();
}