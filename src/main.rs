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

mod cli;
mod config;
mod logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if cli::is_portfolio_invocation(&args) {
        let nargs = cli::normalize_portfolio_args(args);
        return cli::run_from_args(nargs).await;
    }
    // 1. Load config.toml.
    let mut cfg = match config::load::load_from_file("config.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("failed to load config.toml, using defaults: {e}");
            config::SupervisorConfig::default()
        }
    };
    // Per-boot identity: the shared initramfs is identical for all tenants,
    // so /init exports the real TENANT_ID (from the tenant_id= kernel arg).
    // Env wins over the baked-in config default.
    if let Ok(tenant_id) = std::env::var("TENANT_ID") {
        if !tenant_id.trim().is_empty() {
            cfg.supervisor.tenant_id = tenant_id;
        }
    }
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

    // 7. Demand launcher scripts from the wrapper (retry forever — nothing
    //    is baked into the image), write them to the configured program
    //    paths, then register + launch. A later crash-loop re-demands.
    // Output paths are the configured program binaries (fetch keys are
    // "zw"/"engram"; the monitor registry calls the zw program "zerowrapper").
    let mut launcher_paths = std::collections::HashMap::new();
    launcher_paths.insert(
        "engram".to_string(),
        cfg.programs.engram.binary.clone(),
    );
    if let Some(ref zw) = cfg.programs.zerowrapper {
        launcher_paths.insert("zw".to_string(), zw.binary.clone());
    }
    let launcher = monitor::LauncherSource {
        address: cfg.wrapper.address.clone(),
        port: cfg.wrapper.port,
        token: cfg.wrapper.auth_token.clone(),
        tenant_id: cfg.supervisor.tenant_id.clone(),
        paths: launcher_paths,
    };
    launcher.demand_all(&telemetry).await;

    // 7b. monitor::init() — register engram and zerowrapper binaries,
    //    then launch them and start the health-check loop. (The zeroclaw
    //    agent is gone: zw runs its baked-in engine; no daemon socket exists
    //    anymore, so there is no socket-health healing either.)
    let mut registry = monitor::ProgramRegistry::new();
    registry.register(
        monitor::engram_spec(&cfg.programs.engram.binary),
        &telemetry,
    )?;
    if let Some(ref zw) = cfg.programs.zerowrapper {
        registry.register(
            monitor::zerowrapper_spec(&zw.binary),
            &telemetry,
        )?;
    }

    let mut monitor_loop = monitor::MonitorLoop::new(registry);
    monitor_loop = monitor_loop.with_launcher(launcher);
    if let Some(ref p) = cfg.monitor.agent_state_path {
        monitor_loop = monitor_loop.with_agent_state_path(std::path::PathBuf::from(p));
    }
    // Launch all registered programs (engram, zerowrapper).
    monitor_loop.launch_all(&telemetry).await?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.register")
        .category("monitor")
        .message(format!(
            "monitor launched engram={} zerowrapper={} (interval={}s)",
            cfg.programs.engram.binary,
            cfg.programs.zerowrapper.as_ref().map(|z| z.binary.as_str()).unwrap_or("none"),
            cfg.monitor.check_interval_seconds
        ))
        .emit();

    // 8. MCP servers — unix socket (legacy) + HTTP for the daemon.
    //    Registry + wrapper-backed tools, then POST /mcp on 127.0.0.1:http_port
    //    which the zeroclaw daemon dials with mcp.servers transport=http.
    let mcp_api = api::McpApi::new(&cfg.mcp.socket_path, cfg.mcp.max_connections);
    let mcp_registry = mcp::tools::ToolRegistry::new();
    let wrapper_proxy = mcp::handler::WrapperProxy::new(
        &cfg.wrapper.address,
        cfg.wrapper.port,
        &cfg.wrapper.auth_token,
        &cfg.supervisor.tenant_id,
    )?;
    mcp::built_in::register_with_proxy(
        &mcp_registry,
        std::sync::Arc::clone(&wrapper_proxy) as mcp::handler::SharedProxy,
    )?;
    let mcp_server = mcp::server::McpServer::with_registry(std::sync::Arc::new(mcp_registry));
    let mcp_http_port = cfg.mcp.http_port;
    // Grant loop: ask the wrapper which dotted ops this tenant's category
    // may list (tools.granted), refresh every 60s. Wrapper-down fails open
    // (list everything) so a blip never bricks the agent.
    {
        let proxy = std::sync::Arc::clone(&wrapper_proxy);
        let server = mcp_server.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            let mut was_restricted = false;
            let mut failures: u32 = 0;
            loop {
                match proxy.granted_tools().await {
                    Ok(grants) => {
                        failures = 0;
                        let restricted = grants.is_some();
                        let n = grants.as_ref().map(|g| g.len()).unwrap_or(0);
                        server.set_grants(grants).await;
                        if restricted != was_restricted {
                            was_restricted = restricted;
                            tracing::info!(
                                "supervisor tool grants refreshed: {n} tools (restricted={restricted})"
                            );
                        }
                    }
                    Err(e) => {
                        failures += 1;
                        server.set_grants(None).await;
                        // First failure and transitions warn; steady-state
                        // outage would otherwise fail open silently forever.
                        if was_restricted || failures == 1 {
                            was_restricted = false;
                            tracing::warn!("supervisor tool grants query failed (listing all): {e:#}");
                        } else {
                            tracing::debug!("supervisor tool grants still failing: {e:#}");
                        }
                    }
                }
                interval.tick().await;
            }
        });
    }
    tokio::spawn(async move {
        if let Err(e) = mcp::server::http::serve_http(mcp_server, mcp_http_port).await {
            tracing::error!("supervisor MCP HTTP server exited: {e}");
        }
    });
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("supervisor.connected")
        .category("mcp")
        .message(format!(
            "mcp server on {} (max_connections={}) + http 127.0.0.1:{}/mcp",
            mcp_api.socket_path, mcp_api.max_connections, mcp_http_port
        ))
        .emit();

    // 9. proxy::client::connect() — establish connection to wrapper.
    let _proxy_envelope = proxy_api.envelope(
        "heartbeat",
        &cfg.supervisor.tenant_id,
        "ping",
        serde_json::json!({ "vm_id": cfg.supervisor.vm_id }),
    );

    // 10. heartbeat::start() → send heartbeats to the wrapper every
    // interval. The wrapper feeds them into admin liveness (dashboard
    // heartbeat age) — a quiet VM still pulses.
    let heartbeat_api = api::HeartbeatApi::new(cfg.heartbeat.interval_seconds)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    {
        let hb_interval = std::time::Duration::from_secs(cfg.heartbeat.interval_seconds);
        let hb_vm = cfg.supervisor.vm_id.clone();
        let hb_tenant = cfg.supervisor.tenant_id.clone();
        let hb_addr = cfg.wrapper.address.clone();
        let hb_port = cfg.wrapper.port;
        let hb_token = cfg.wrapper.auth_token.clone();
        let hb_tel = telemetry.clone();
        tokio::spawn(async move {
            // Small delay so the wrapper listener is up before the first beat.
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let mut tick = tokio::time::interval(hb_interval);
            loop {
                tick.tick().await;
                let payload = heartbeat_api.payload(
                    &hb_vm,
                    &hb_tenant,
                    serde_json::json!({}),
                    serde_json::json!({}),
                );
                match payload {
                    Ok(p) => {
                        if let Err(e) = heartbeat::send_heartbeat(
                            &p, &hb_addr, hb_port, &hb_token, &hb_tel,
                        )
                        .await
                        {
                            tracing::warn!("heartbeat send failed: {e:#}");
                        }
                    }
                    Err(e) => tracing::warn!("heartbeat payload failed: {e}"),
                }
            }
        });
    }
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("heartbeat.send")
        .category("heartbeat")
        .message(format!(
            "heartbeat sender started every {}s: vm={}",
            cfg.heartbeat.interval_seconds, cfg.supervisor.vm_id
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

    // 15. monitor::start() → spawn the health-check loop. It polls every
    //    check_interval_seconds, reads the agent state file, and restarts any
    //    program that crashed (up to max_restarts before escalating).
    let monitor_loop = std::sync::Arc::new(tokio::sync::Mutex::new(monitor_loop));
    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    let loop_handle = tokio::spawn(monitor::MonitorLoop::run(
        monitor_loop,
        cfg.monitor.check_interval_seconds,
        telemetry.clone(),
        cancel_rx,
    ));
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("monitor.start")
        .category("monitor")
        .message("monitor health-check loop started")
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
    // Stop the monitor loop.
    let _ = cancel_tx.send(true);
    let _ = loop_handle.await;
    telemetry.flush().await;
    Ok(())
}
