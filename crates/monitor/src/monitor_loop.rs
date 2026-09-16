//! The monitor loop: poll all supervised programs, restart on crash.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::interval;

use shared::Result;

use crate::programs::{ProgramRegistry, ProgramSpec};
use crate::recover::{escalate_to_wrapper, restart_program};
use crate::watcher::{
    health_check, read_agent_state, watch_process, HealthStatus, WatchedProcess,
};

/// Tracks running programs and their PIDs.
pub struct MonitorLoop {
    pub registry: ProgramRegistry,
    pub running: HashMap<String, WatchedProcess>,
    /// Owned child handles. Kept (never forgotten): `try_wait()` both
    /// detects exits AND reaps the zombie, which a bare PID watch cannot do.
    children: HashMap<String, tokio::process::Child>,
    /// Path to the agent's state file (state/daemon_state.json), if any.
    pub agent_state_path: Option<PathBuf>,
    /// Consecutive restart failures per program.
    pub failures: HashMap<String, u32>,
    /// After this many restart failures, re-demand launcher scripts from
    /// the wrapper and try again (never give up: a fixed portal-side script
    /// eventually arrives and the programs start working).
    pub max_restarts: u32,
    /// Where launcher scripts come from (None = baked paths only, used in
    /// tests). Set in production: enables the demand-loop on boot and the
    /// re-demand after repeated crashes.
    pub launcher: Option<crate::launcher::LauncherSource>,
}

impl MonitorLoop {
    pub fn new(registry: ProgramRegistry) -> Self {
        Self {
            registry,
            running: HashMap::new(),
            children: HashMap::new(),
            agent_state_path: None,
            failures: HashMap::new(),
            max_restarts: 3,
            launcher: None,
        }
    }

    /// Set the path the agent writes its health state to.
    pub fn with_agent_state_path(mut self, path: PathBuf) -> Self {
        self.agent_state_path = Some(path);
        self
    }

    /// Set the launcher source (wrapper demand endpoint + output paths).
    pub fn with_launcher(mut self, source: crate::launcher::LauncherSource) -> Self {
        self.launcher = Some(source);
        self
    }

    /// Launch every registered program that isn't already running.
    pub async fn launch_all(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        // Collect specs first to avoid borrowing `self` immutably (registry)
        // while mutating it (running) inside launch_one.
        let specs: Vec<ProgramSpec> = self
            .registry
            .list()
            .into_iter()
            .cloned()
            .collect();
        for spec in &specs {
            if self.running.contains_key(&spec.name) {
                continue;
            }
            self.launch_one(spec, telemetry).await?;
        }
        Ok(())
    }

    /// Launch a single program and register its PID for watching.
    pub async fn launch_one(
        &mut self,
        spec: &ProgramSpec,
        telemetry: &telemetry::Telemetry,
    ) -> Result<()> {
        let mut cmd = tokio::process::Command::new(&spec.binary);
        cmd.args(&spec.args);
        let child = cmd.spawn().map_err(|e| {
            shared::SupervisorError::Monitor(format!("failed to launch {}: {e}", spec.name))
        })?;
        let pid = child.id().ok_or_else(|| {
            shared::SupervisorError::Monitor(format!("no pid for {}", spec.name))
        })?;
        // Keep the child handle: try_wait() in tick() both detects exits and
        // reaps them. A forgotten handle leaks a zombie that reads as alive.
        let watched = watch_process(pid, &spec.name, telemetry)?;
        self.running.insert(spec.name.clone(), watched);
        self.children.insert(spec.name.clone(), child);
        Ok(())
    }

    /// Seconds after a (re)launch during which failures don't count.
    /// Slow starters (engram loads the helix DB + warms embeddings before
    /// :4545 opens) would otherwise be killed before they ever get healthy.
    const STARTUP_GRACE_SECS: u64 = 90;

    /// Run one health-check tick: probe every program, restart the dead ones.
    pub async fn tick(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        let mut crashed: Vec<String> = Vec::new();
        let now = shared::now_unix_millis();

        // Owned handles first: try_wait() reaps exited children (no zombies)
        // and reports the exit, race-free. A `daemonizes` launcher exiting
        // is EXPECTED (it forked the real daemon) — drop the handle quietly.
        let mut exited: Vec<String> = Vec::new();
        for (name, child) in self.children.iter_mut() {
            match child.try_wait() {
                Ok(Some(_)) => exited.push(name.clone()),
                Ok(None) => {}
                Err(_) => exited.push(name.clone()),
            }
        }
        for name in exited {
            self.children.remove(&name);
            let daemonizes = self
                .registry
                .get(&name)
                .ok()
                .is_some_and(|s| s.daemonizes);
            if !daemonizes && !crashed.contains(&name) {
                crashed.push(name);
            }
        }

        for (name, proc) in &self.running {
            // Already flagged via try_wait() above: pushing twice would
            // restart the program twice and duplicate the daemon.
            if crashed.contains(name) {
                continue;
            }
            // Startup grace: a freshly (re)launched program gets time to
            // open its port before failures count. Reaping above still ran.
            if now.saturating_sub(proc.started_unix_millis) < Self::STARTUP_GRACE_SECS * 1000 {
                continue;
            }
            // Daemonizing servers are watched by port, not by launcher PID.
            if let Ok(spec) = self.registry.get(name) {
                if spec.daemonizes {
                    if let Some(port) = spec.health_port {
                        if !Self::probe_port(port).await {
                            telemetry
                                .builder()
                                .level(telemetry::Level::Warn)
                                .kind("monitor.port_unreachable")
                                .category("monitor")
                                .message(format!("{name} port {port} unreachable"))
                                .emit();
                            crashed.push(name.clone());
                        }
                    }
                    continue;
                }
            }
            match health_check(proc.pid, name, telemetry)? {
                HealthStatus::Alive => {}
                _ => crashed.push(name.clone()),
            }
        }

        // For the agent, also inspect its internal component state.
        if let Some(ref path) = self.agent_state_path {
            if let Some(state) = read_agent_state(path) {
                if !state.all_healthy() {
                    for (comp, c) in state.unhealthy_components() {
                        telemetry
                            .builder()
                            .level(telemetry::Level::Warn)
                            .kind("monitor.agent_unhealthy")
                            .category("monitor")
                            .message(format!("agent component {comp} status={}", c.status))
                            .data("component", comp.clone())
                            .data("status", c.status.clone())
                            .emit();
                    }
                }
            }
        }

        for name in crashed {
            self.handle_crash(&name, telemetry).await?;
        }
        Ok(())
    }

    /// Restart a crashed program (or escalate if it keeps failing).
    async fn handle_crash(
        &mut self,
        name: &str,
        telemetry: &telemetry::Telemetry,
    ) -> Result<()> {
        let spec = match self.registry.get(name) {
            Ok(s) => s.clone(),
            Err(_) => return Ok(()),
        };
        if !spec.restart_on_crash {
            escalate_to_wrapper(name, "crash; restart_on_crash=false", telemetry)?;
            return Ok(());
        }
        let failures = self.failures.entry(name.to_string()).or_insert(0);
        *failures += 1;
        if *failures > self.max_restarts {
            // Crashed max_restarts times: the launcher script itself is
            // suspect. Wait, re-demand scripts from the wrapper (a fixed
            // portal-side script eventually arrives), reset the counter and
            // relaunch. Never give up.
            telemetry
                .builder()
                .level(telemetry::Level::Warn)
                .kind("monitor.redemand")
                .category("monitor")
                .message(format!(
                    "{name} crashed {} times, re-demanding launcher scripts",
                    self.max_restarts
                ))
                .emit();
            if let Some(ref source) = self.launcher {
                source.demand_all(telemetry).await;
            }
            self.failures.insert(name.to_string(), 0);
        }
        // A daemonizing server outlives its launcher: kill the real daemon
        // by exact process name (== spec name by convention) so the fresh
        // launch doesn't pile onto a live one fighting over the port + DB.
        if spec.daemonizes {
            let killed = Self::kill_by_name(&spec.name, telemetry);
            telemetry
                .builder()
                .level(telemetry::Level::Info)
                .kind("monitor.daemon_killed")
                .category("monitor")
                .message(format!("killed {killed} '{}(s)' before relaunch", spec.name))
                .emit();
        }
        self.running.remove(name);
        self.children.remove(name);
        let child = restart_program(&spec, telemetry).await?;
        // Re-register the new PID from the owned handle (exact, no ps scan).
        let child_pid = child.id().ok_or_else(|| {
            shared::SupervisorError::Monitor(format!("no pid for restarted {name}"))
        })?;
        let watched = watch_process(child_pid, name, telemetry)?;
        self.running.insert(name.to_string(), watched);
        self.children.insert(name.to_string(), child);
        Ok(())
    }

    /// Probe 127.0.0.1:port with a short budget (daemon liveness).
    async fn probe_port(port: u16) -> bool {
        tokio::time::timeout(
            Duration::from_secs(2),
            tokio::net::TcpStream::connect(("127.0.0.1", port)),
        )
        .await
        .is_ok_and(|r| r.is_ok())
    }

    /// Kill all processes with exactly `process_name` (sysinfo exact match —
    /// no shell-pattern footguns). Returns the kill count. Used for
    /// self-daemonizing servers whose launcher PID is already gone.
    fn kill_by_name(process_name: &str, telemetry: &telemetry::Telemetry) -> usize {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut n = 0usize;
        for proc in sys.processes().values() {
            if proc.name() == process_name && proc.kill() {
                n += 1;
            }
        }
        let _ = telemetry;
        n
    }

    /// Spawn the loop as a long-running task. Returns when `cancel` fires.
    pub async fn run(
        arc: Arc<Mutex<Self>>,
        interval_secs: u64,
        telemetry: telemetry::Telemetry,
        mut cancel: tokio::sync::watch::Receiver<bool>,
    ) {
        let mut ticker = interval(Duration::from_secs(interval_secs));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let mut guard = arc.lock().await;
                    if let Err(e) = guard.tick(&telemetry).await {
                        eprintln!("monitor tick error: {e}");
                    }
                }
                _ = cancel.changed() => {
                    if *cancel.borrow() {
                        break;
                    }
                }
            }
        }
    }
}
