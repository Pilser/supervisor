//! TCP transport variant for the MCP server.
//!
//! The primary transport is the UNIX socket; this module offers a TCP variant
//! (e.g. for tests or remote clients) using the same JSON-RPC dispatch.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};

use crate::protocol::{decode, encode};
use crate::server::handshake;
use crate::server::McpServer;

/// Bind a `TcpListener` on the loopback address. When `port` is `0`, the OS
/// picks a free port, which is returned.
pub async fn bind(port: u16) -> anyhow::Result<(TcpListener, u16)> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    let bound = listener.local_addr()?.port();
    Ok((listener, bound))
}

/// Accept connections and serve them until canceled.
pub async fn serve(server: &McpServer, listener: TcpListener) -> anyhow::Result<()> {
    loop {
        let (stream, _addr) = listener.accept().await?;
        let server = server.clone();
        tokio::spawn(async move {
            handle_stream(&server, stream).await;
        });
    }
}

/// Serve a single TCP connection using the same handler logic as the UNIX
/// socket.
pub async fn handle_stream(server: &McpServer, stream: TcpStream) {
    server.emit_connected();

    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let result: anyhow::Result<()> = async {
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                break;
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