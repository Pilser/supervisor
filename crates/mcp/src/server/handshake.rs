//! MCP handshake and request dispatch: `initialize`, `tools/list`,
//! `tools/call`, notifications.

use anyhow::Context;
use serde_json::{json, Value};

use crate::protocol::{
    INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, Message, id_of, response_err, response_ok,
};
use crate::server::McpServer;

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "supervisor-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Route a decoded JSON-RPC message. Returns `None` for notifications (no
/// response expected) and `Some(Message)` for everything else.
pub async fn handle_message(
    server: &McpServer,
    msg: Message,
) -> anyhow::Result<Option<Message>> {
    let method = match msg.method.as_deref() {
        Some(m) => m,
        None => return Ok(Some(response_err(id_of(&msg), INVALID_REQUEST, "missing method"))),
    };

    match method {
        "initialize" => Ok(Some(handle_initialize(&msg))),
        // Client tells the server it is initialized — fire-and-forget.
        "notifications/initialized" | "initialized" => Ok(None),
        "ping" => Ok(Some(response_ok(id_of(&msg), json!({})))),
        "tools/list" => handle_tools_list(server, &msg).await,
        "tools/call" => handle_tools_call(server, &msg).await,
        other => Ok(Some(response_err(
            id_of(&msg),
            METHOD_NOT_FOUND,
            format!("method not found: {other}"),
        ))),
    }
}

/// The MCP `initialize` handshake result.
pub fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
    })
}

/// Build the `initialize` response.
fn handle_initialize(msg: &Message) -> Message {
    response_ok(id_of(msg), initialize_result())
}

/// Registry names (`save_memory`) vs grant dotted ops (`memory.save`):
/// a grant matches when the registry name is the dotted op itself, the
/// `verb_family` swap, or the bare verb.
fn grant_matches(grant: &str, registry_name: &str) -> bool {
    if grant == registry_name {
        return true;
    }
    if let Some((family, verb)) = grant.split_once('.') {
        if !verb.contains('_')
            && (format!("{verb}_{family}") == registry_name || verb == registry_name)
        {
            return true;
        }
        if verb == registry_name {
            return true;
        }
    }
    false
}

/// Respond with the advertised tool list from the registry, filtered to the
/// tenant category's grant set when one is active. The daemon only ever sees
/// the filtered names, so ungranted tools are invisible (not just rejected).
async fn handle_tools_list(server: &McpServer, msg: &Message) -> anyhow::Result<Option<Message>> {
    let grants = server.grants.read().await.clone();
    let tools: Vec<Value> = server
        .registry
        .list()
        .into_iter()
        .filter(|t| match &grants {
            // None = unrestricted (or wrapper unreachable: fail-open).
            None => true,
            // Some, even empty (= deny all categories), filters strictly.
            Some(set) => set.iter().any(|g| grant_matches(g, &t.name)),
        })
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.params,
            })
        })
        .collect();
    Ok(Some(response_ok(id_of(msg), json!({ "tools": tools }))))
}

/// Execute a `tools/call`, emitting telemetry around the invocation.
async fn handle_tools_call(server: &McpServer, msg: &Message) -> anyhow::Result<Option<Message>> {
    let params = msg.params.clone().unwrap_or(Value::Null);
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("tools/call requires a `name` string parameter"))?;
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
    let id = id_of(msg);

    server.emit_tool("called", name, &arguments);

    match server.registry.call(name, arguments).await {
        Ok(result) => {
            let text = serde_json::to_string(&result)
                .with_context(|| format!("failed to serialize result for tool `{name}`"))?;
            let payload = json!({
                "content": [{ "type": "text", "text": text }],
                "isError": false,
            });
            server.emit_tool("completed", name, &result);
            Ok(Some(response_ok(id, payload)))
        }
        Err(err) => {
            server.emit_tool_error("failed", name, &err.to_string());
            Ok(Some(response_err(
                id,
                INVALID_PARAMS,
                format!("tool `{name}` failed: {err}"),
            )))
        }
    }
}

