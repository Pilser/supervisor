# Supervisor

The **guest-side daemon** that runs inside every Firecracker microVM of the AI
Learning Companion platform. While the
[wrapper](https://github.com) (a separate repo) is the **brain** on the host, the
supervisor is the **hands** inside each VM: it monitors the AI agent, exposes data
tools to the agent over MCP, proxies data requests back to the wrapper, receives
deployments, and reports telemetry.

The supervisor does **not** own any data — it is a protocol layer. When the agent asks
to save a memory, the supervisor asks the wrapper, which validates the tenant,
opens the correct tenant DB, and performs the write.

## Architecture

```
[Wrapper (host)] ◄──── MCP/data requests + deployments ──── [Supervisor (this repo)]
       ▲                                                        │
       │                                                        │ runs & monitors
       │                                                        ▼
       │                                                [AI Agent + Memory binary]
       │                                                        │
       └──────────────── tenant DB reads/writes ────────────────┘
```

**Data flow:** agent calls a supervisor tool → supervisor attaches the tenant id →
supervisor sends the request to the wrapper over the network → wrapper validates the
tenant and performs the DB operation → response flows back to the agent.

## Roles

1. **Monitor** — watches the agent and memory binaries, restarts them if they crash.
2. **MCP server** — exposes tools (memory, profile, history, projects, skills, reports,
   config, tenant info) to the agent over a Unix socket using JSON-RPC 2.0.
3. **Proxy** — forwards agent data requests to the wrapper, with caching, an offline
   queue, connection pooling and retry.
4. **Deployment agent** — receives category-based plugins, configs, binaries and
   commands from the wrapper and installs them.
5. **Telemetry** — tracks every event/metric, writes daily JSON files locally, and
   **forwards** telemetry to the wrapper for central monitoring.

## Tech stack

- **Language:** Rust (edition 2024)
- **MCP transport:** JSON-RPC 2.0 over a Unix domain socket (`/tmp/supervisor.sock`)
- **Wrapper protocol:** JSON over HTTP via `reqwest`
- **Process monitoring:** [`sysinfo`](https://github.com/GuillaumeGomez/sysinfo)
- **Async runtime:** Tokio
- **Serialization:** serde / serde_json

## Workspace structure

Each crate is small and focused, matching the planned folder layout. Crates are listed
in dependency order (implement top-down):

| # | Crate | Responsibility |
|---|-------|----------------|
| 1 | `core` | Shared types (`TenantContext`, `AgentInfo`, `MemoryEntry`, `Project`, `Skill`, `Report`, `CategoryInfo`, `DeploymentArtifact`, …), the `Plugin` trait, `SupervisorError`/`Result`, utilities |
| 2 | `telemetry` | Structured events + metrics (same schema as the wrapper), daily disk files, and a **forwarder** that batches and POSTs telemetry to the wrapper |
| 3 | `proxy` | Request pipeline to the wrapper (`RequestType`/`RequestBuilder`), connection pool, retry, local cache, offline queue, auth token handling |
| 4 | `mcp` | JSON-RPC 2.0 server on a Unix socket, tool registry/builder/handler, built-in tools (memory, profile, history, projects, skills, reports, config, tenant), permissions |
| 5 | `monitor` | Watches agent/memory processes, detects crashes/freeze/leak, recovers (restart, kill, escalate) |
| 6 | `heartbeat` | Sends heartbeats to the wrapper, tracks failure/recovery, uptime/latency stats |
| 7 | `commands` | Receives and executes wrapper commands (install, configure, restart, shutdown, run-script, status) |
| 8 | `deployment` | Receives deployment packages, handles artifacts (plugins/configs/binaries/commands), installs them, rolls back on failure |
| 9 | `plugin_host` | Loads `.so` supervisor plugins, folder watch + hot-reload, registry, hook dispatch, inter-plugin API |
| 10 | `api` | Public plugin-facing API surface tying every subsystem together |

The binary lives in `src/` (`main.rs` + `config/`, `logging/`).

## Built-in MCP tools

Exposed to the agent (all proxy to the wrapper):

- **Memory:** `save_memory`, `get_memory`, `list_memory`, `search_memory`, `delete_memory`
- **Profile:** `get_profile`, `update_profile`
- **History:** `log_history`, `get_history`
- **Projects:** `create_project`, `update_project`, `list_projects`, `get_project`
- **Skills:** `add_skill`, `update_skill`, `list_skills`, `get_skill`
- **Reports:** `get_report`, `list_reports`
- **Config:** `get_config`, `set_config`
- **Tenant:** `get_tenant_info`, `get_category_info`

## Hook points

Plugins can subscribe to events like `OnAgentConnected`, `OnToolCalled`,
`OnProcessCrashed`, `OnHeartbeatSent`, `OnDeploymentCompleted`, `OnCategoryChanged`,
and many more (see `core::HookPoint`).

## Category-based deployment

Tenants belong to categories (robotics, painting, music, staff, …). When a tenant is
assigned to a category, the wrapper pushes a deployment package to the supervisor:
supervisor plugins, agent plugins, config files, binaries and post-deployment commands.
The supervisor installs everything and rolls back on failure.

## Telemetry forwarding

Events are written to daily files under `/var/log/supervisor/telemetry/` **and**
forwarded in batches to the wrapper (with retry/backoff and optional compression) for
central monitoring.

## Configuration

`config.toml` at the repo root holds the VM/tenant identity, wrapper address, MCP
socket path, monitor/heartbeat/command/deployment settings, the programs to monitor,
and telemetry config.

## Build & verify

```bash
# Only ever run cargo check (never build/test/run)
cargo check --workspace
```

> Rule: verification is by `cargo check` only. The goal is to confirm the code
> compiles, not to run it.

## License

TBD
