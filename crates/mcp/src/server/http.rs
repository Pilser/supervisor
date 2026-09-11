//! HTTP transport for the MCP server (streamable-http compatible).
//!
//! Mirrors what `zw` exposes: `POST /mcp` with a JSON-RPC body, routed
//! through [`handshake::handle_message`] against the shared [`McpServer`]
//! registry. No bearer — localhost-only, same as the unix socket transport.
//! This is the endpoint the zeroclaw daemon dials with
//! `mcp.servers.<name>.transport=http`.

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use super::McpServer;
use crate::protocol::Message;

/// Serve `POST /mcp` on 127.0.0.1:`port` until the process exits.
pub async fn serve_http(server: McpServer, port: u16) -> anyhow::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    tracing::info!("supervisor MCP HTTP listening on 127.0.0.1:{port}/mcp");
    loop {
        let (stream, _addr) = listener.accept().await?;
        let server = server.clone();
        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let server = server.clone();
                async move { handle(req, &server).await }
            });
            let conn = hyper_util::server::conn::auto::Builder::new(
                hyper_util::rt::TokioExecutor::new(),
            );
            if let Err(e) = conn.serve_connection(TokioIo::new(stream), svc).await {
                tracing::debug!("supervisor MCP HTTP conn error: {e}");
            }
        });
    }
}

async fn handle(
    req: Request<Incoming>,
    server: &McpServer,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    if req.method() != Method::POST || req.uri().path() != "/mcp" {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("not found")))
            .unwrap());
    }
    let body = req.into_body().collect().await?.to_bytes();
    let msg: Message = match serde_json::from_slice(&body) {
        Ok(m) => m,
        Err(e) => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                &serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32700, "message": format!("parse: {e}")}}),
            ));
        }
    };
    match super::handshake::handle_message(server, msg).await {
        Ok(Some(resp)) => {
            let text = serde_json::to_string(&resp).unwrap_or_else(|e| {
                serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32603, "message": e.to_string()}}).to_string()
            });
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("cache-control", "no-store")
                .body(Full::new(Bytes::from(text)))
                .unwrap())
        }
        Ok(None) => Ok(Response::builder()
            .status(StatusCode::ACCEPTED)
            .body(Full::new(Bytes::new()))
            .unwrap()),
        Err(e) => Ok(json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32603, "message": e.to_string()}}),
        )),
    }
}

fn json_response(status: StatusCode, v: &serde_json::Value) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("cache-control", "no-store")
        .body(Full::new(Bytes::from(v.to_string())))
        .unwrap()
}
