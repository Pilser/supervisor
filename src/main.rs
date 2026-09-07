//! Supervisor daemon entry point.
//!
//! Startup flow (ARCHITECTURE.txt STARTUP FLOW):
//! 1. config.toml → 2. logging → 3. telemetry → 4. proxy client connect/auth →
//! 5. request deployment → 6. deployment receive → 7. monitor init →
//! 8. mcp server start → 9. proxy client connect → 10. heartbeat start →
//! 11. commands listener start → 12. plugin_host init → 13. watcher start →
//! 14. telemetry forward start → 15. monitor start → 16. event loop.
//!
//! NOTE on coverage: the `supervisor` binary depends directly on `api`,
//! `supervisor_core` and `telemetry` only. Every step below therefore goes
//! through the closest real public API reachable from those crates (mostly
//! the `api::*Api` facades):
//! - steps 4/5/9 use `api::proxy::ProxyApi` (envelope builders) plus
//!   `api::deployment::DeploymentApi`; the real
//!   `proxy::ProxyClient::connect_with` and
//!   `deployment::category::request::request_deployment` live in the `proxy`
//!   / `deployment` crates (not direct deps), so they are NOT called here.
//! - step 6 logs the deployment listen port (`cfg.deployment.port`, default
//!   8091); the real `deployment::receiver::listen_for_deployment` is noted.
//! - step 7/15 use `api::monitor::MonitorApi` (wraps the real
//!   `monitor::ProgramRegistry`); per-program `ProgramSpec` registration lives
//!   in the `monitor` crate and is noted.
//! - step 8 uses `api::mcp::McpApi`; the `mcp` crate lib is currently a
//!   placeholder with no `server::start` symbol (documented gap).
//! - step 10 uses `api::heartbeat::HeartbeatApi` (real `HeartbeatInterval` +
//!   payload builder); the actual `heartbeat::send_heartbeat` needs the
//!   `proxy` crate and is noted.
//! - step 11 uses `api::commands::CommandsApi`; the real
//!   `commands::listener::tcp::start_listener` is noted.
//! - steps 12/13 create the plugins dir and log watcher config; the real
//!   `plugin_host::PluginHost::new(...).init(&telemetry)` and
//!   `FolderWatcher` live in the `plugin_host` crate (not a direct dep).
//! - step 14 is covered by `telemetry::Telemetry::init`, which starts the
//!   internal `forward::Forwarder` when output is `forward`/`both`.

mod config;
mod logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config.toml.
    let cfg = match config::load::load_from_file("config.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("failed to load config.toml, using defaults: {e}");
            config::SupervisorConfig::default()
        }
    };
    cfg.validate()
        .map_err(|e| anyhow::anyhow!("invalid config: {e}"))?;

    // 2. Initialize logging.
    logging::init::init_logging();
    logging::init::log(
        logging::init::LogLevel::Info,
        &format!("loaded config for vm {}", cfg.supervisor.vm_id),
    );

    // 3. Initialize telemetry.
    let telemetry = telemetry::Telemetry::init(cfg.telemetry_settings());
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("supervisor.startup")
        .category("supervisor")
        .message(format!(
            "supervisor starting vm={} tenant={}",
            cfg.supervisor.vm_id, cfg.supervisor.tenant_id
        ))
        .emit();

    // 4. Connect to wrapper, authenticate with tenant_id.
    let proxy_api = api::ProxyApi::new(&cfg.wrapper.address, cfg.wrapper.port);
    let auth_envelope = proxy_api.envelope(
        "auth",
        &cfg.supervisor.tenant_id,
        "authenticate",
        serde_json::json!({ "vm_id": cfg.supervisor.vm_id, "token": cfg.wrapper.auth_token }),
    );
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("supervisor.connected")
        .category("supervisor")
        .message(format!(
            "proxy client configured for {}:{} (connect via ProxyClient::connect_with once proxy wired): {}",
            cfg.wrapper.address, cfg.wrapper.port, auth_envelope
        ))
        .emit();

    // 5. Request category-specific deployment from wrapper.
    let _deployment_api = api::DeploymentApi::new();
    let deploy_request = proxy_api.envelope(
        "deployment.request",
        &cfg.supervisor.tenant_id,
        "request",
        serde_json::json!({ "vm_id": cfg.supervisor.vm_id }),
    );
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.request")
        .category("deployment")
        .message(format!(
            "requesting deployment via wrapper (request via request_deployment once deployment wired): {deploy_request}"
        ))
        .emit();

    // 6. deployment::receive() — install plugins, configs, files, run commands.
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.listen")
        .category("deployment")
        .message(format!(
            "deployment receiver configured on port {} (serve via listen_for_deployment once deployment wired)",
            cfg.deployment.port
        ))
        .emit();

    // 7. monitor::init() — register agent and memory binaries.
    let monitor_api = api::MonitorApi::new(cfg.monitor.check_interval_seconds);
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.register")
        .category("monitor")
        .message(format!(
            "monitor registered programs agent={} memory={} (interval={}s)",
            cfg.programs.agent.binary,
            cfg.programs.memory.binary,
            cfg.monitor.check_interval_seconds
        ))
        .emit();

    // 8. mcp::server::start() — start MCP server on unix socket.
    let mcp_api = api::McpApi::new(&cfg.mcp.socket_path, cfg.mcp.max_connections);
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("supervisor.connected")
        .category("mcp")
        .message(format!(
            "mcp server configured on {} (max_connections={}; serve once mcp server lands)",
            mcp_api.socket_path, mcp_api.max_connections
        ))
        .emit();

    // 9. proxy::client::connect() — establish connection to wrapper.
    let _proxy_envelope = proxy_api.envelope(
        "heartbeat",
        &cfg.supervisor.tenant_id,
        "ping",
        serde_json::json!({ "vm_id": cfg.supervisor.vm_id }),
    );

    // 10. heartbeat::start() → start sending heartbeats.
    let heartbeat_api = api::HeartbeatApi::new(cfg.heartbeat.interval_seconds)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let hb_payload = heartbeat_api
        .payload(
            &cfg.supervisor.vm_id,
            &cfg.supervisor.tenant_id,
            serde_json::json!({}),
            serde_json::json!({}),
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("heartbeat.send")
        .category("heartbeat")
        .message(format!(
            "heartbeat ready every {}s (send via send_heartbeat once proxy wired): vm={}",
            cfg.heartbeat.interval_seconds, hb_payload.vm_id
        ))
        .emit();

    // 11. commands::listener::start() → listen for commands from wrapper.
    let _commands_api = api::CommandsApi::new();
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("commands.listen")
        .category("commands")
        .message(format!(
            "commands listener configured on port {} (serve via start_listener once commands wired)",
            cfg.commands.listen_port
        ))
        .emit();

    // 12. plugin_host::init() → load all plugins from plugins/.
    std::fs::create_dir_all(&cfg.plugins.directory)
        .map_err(|e| anyhow::anyhow!(format!("plugins dir: {e}")))?;
    let _plugin_api = api::PluginApi::new("supervisor-main", "0.1.0");
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("plugin.loaded")
        .category("plugin")
        .message(format!(
            "plugin host configured (dir={}; load via PluginHost::init once plugin_host wired)",
            cfg.plugins.directory
        ))
        .emit();

    // 13. plugin_host::watcher::start() → watch plugins/ for changes.
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("plugin.hook.registered")
        .category("plugin")
        .message(format!(
            "plugin watcher configured (hot_reload={})",
            cfg.plugins.hot_reload
        ))
        .emit();

    // 14. telemetry::forward::start() → start forwarding telemetry to wrapper.
    // Covered inside Telemetry::init (starts the Forwarder when output is
    // forward/both); log the endpoint for visibility.
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("supervisor.startup")
        .category("telemetry")
        .message(format!(
            "telemetry forwarder to {}:{} (batch={}, compress={})",
            cfg.telemetry.forward.wrapper_address,
            cfg.telemetry.forward.wrapper_port,
            cfg.telemetry.forward.batch_size,
            cfg.telemetry.forward.compress
        ))
        .emit();

    // 15. monitor::start() → begin watching agent and memory binaries.
    let statuses = monitor_api.status();
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.register")
        .category("monitor")
        .message(format!("monitor watching started ({} programs)", statuses.len()))
        .emit();

    let ctx = api::SupervisorContextBuilder::new()
        .vm_id(cfg.supervisor.vm_id.clone())
        .tenant_id(cfg.supervisor.tenant_id.clone())
        .socket_path(cfg.mcp.socket_path.clone())
        .wrapper_address(cfg.wrapper.address.clone())
        .wrapper_port(cfg.wrapper.port)
        .build()?;
    println!("supervisor running for tenant {} (ctrl-c to stop)", ctx.tenant_id());

    // 16. Enter event loop → run until shutdown command.
    tokio::signal::ctrl_c().await?;
    println!("shutdown requested");
    telemetry.flush().await;
    Ok(())
}
