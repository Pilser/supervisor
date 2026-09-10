//! The monitor loop: poll all supervised programs, restart on crash.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::interval;

use shared::Result;

use crate::programs::{ProgramRegistry, ProgramSpec};
use crate::recover::{escalate_to_wrapper, kill_process, restart_program};
use crate::watcher::{
    health_check, read_agent_state, watch_process, HealthStatus, WatchedProcess,
};

/// Tracks running programs and their PIDs.
pub struct MonitorLoop {
    pub registry: ProgramRegistry,
    pub running: HashMap<String, WatchedProcess>,
    /// Path to the agent's state file (state/daemon_state.json), if any.
    pub agent_state_path: Option<PathBuf>,
    /// Unix socket the agent daemon must answer on. A live PID is NOT
    /// enough: snapshot restores (and crashes) can leave a running daemon
    /// with a dead listener, which wedges zw's bridge forever.
    pub agent_socket_path: Option<PathBuf>,
    /// Consecutive socket dial failures.
    pub socket_failures: u32,
    /// Consecutive restart failures per program.
    pub failures: HashMap<String, u32>,
    /// Give up and escalate after this many restart failures.
    pub max_restarts: u32,
}

/// Dial failures tolerated before healing the agent link.
const SOCKET_FAILURE_THRESHOLD: u32 = 2;

impl MonitorLoop {
    pub fn new(registry: ProgramRegistry) -> Self {
        Self {
            registry,
            running: HashMap::new(),
            agent_state_path: None,
            agent_socket_path: None,
            socket_failures: 0,
            failures: HashMap::new(),
            max_restarts: 5,
        }
    }

    /// Set the path the agent writes its health state to.
    pub fn with_agent_state_path(mut self, path: PathBuf) -> Self {
        self.agent_state_path = Some(path);
        self
    }

    /// Set the daemon socket that must accept connections. Enables the
    /// socket-health check in `tick()`: on repeated dial failures the
    /// agent AND zerowrapper are restarted as a pair (zw holds dead dial
    /// state, so restarting the agent alone just moves the wedge).
    pub fn with_agent_socket_path(mut self, path: PathBuf) -> Self {
        self.agent_socket_path = Some(path);
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
        // Keep the child handle alive so it doesn't get reaped/dropped.
        let watched = watch_process(pid, &spec.name, telemetry)?;
        self.running.insert(spec.name.clone(), watched);
        // Detach: forget the child handle (the supervisor is the parent and the
        // process table is what we watch).
        std::mem::forget(child);
        Ok(())
    }

    /// Run one health-check tick: probe every program, restart the dead ones.
    pub async fn tick(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        let mut crashed: Vec<String> = Vec::new();

        for (name, proc) in &self.running {
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

        // Agent link health: PID-alive does not mean serving. Dial the
        // daemon socket; a restored-or-wedged daemon stops answering while
        // its PID looks fine, and zw's bridge hangs on the dead socket.
        if self.agent_socket_path.is_some() {
            self.check_agent_socket(telemetry).await?;
        }
        Ok(())
    }

    /// Dial the agent socket once (2s budget).
    async fn probe_agent_socket(&self) -> bool {
        let Some(ref path) = self.agent_socket_path else {
            return true;
        };
        tokio::time::timeout(
            Duration::from_secs(2),
            tokio::net::UnixStream::connect(path),
        )
        .await
        .is_ok_and(|r| r.is_ok())
    }

    /// Heal a dead agent link: kill the wedged pair, drop stale socket
    /// files (a restarted daemon cannot bind otherwise), restart both.
    async fn check_agent_socket(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        if self.probe_agent_socket().await {
            self.socket_failures = 0;
            return Ok(());
        }
        self.socket_failures += 1;
        telemetry
            .builder()
            .level(telemetry::Level::Warn)
            .kind("monitor.socket_unreachable")
            .category("monitor")
            .message(format!(
                "agent socket unreachable ({}/{SOCKET_FAILURE_THRESHOLD})",
                self.socket_failures
            ))
            .emit();
        if self.socket_failures < SOCKET_FAILURE_THRESHOLD {
            return Ok(());
        }
        self.socket_failures = 0;
        telemetry
            .builder()
            .level(telemetry::Level::Warn)
            .kind("monitor.socket_heal")
            .category("monitor")
            .message("restarting agent+zerowrapper pair (dead daemon socket)")
            .emit();
        // Kill the wedged processes first: restart only spawns, and two
        // daemons would fight over the socket lock.
        for name in ["agent", "zerowrapper"] {
            if let Some(proc) = self.running.get(name) {
                let _ = kill_process(proc.pid, telemetry);
            }
        }
        // Drop stale socket files so the fresh daemon can bind.
        if let Some(ref path) = self.agent_socket_path {
            let _ = std::fs::remove_file(path);
            let _ = std::fs::remove_file(path.with_extension("sock.lock"));
            let _ = std::fs::remove_file(
                path.parent()
                    .unwrap_or(std::path::Path::new("/tmp"))
                    .join("daemon.sock.lock"),
            );
        }
        // Restart the pair through the normal crash path (fresh PIDs,
        // failure counting, escalation on repeated failure).
        self.handle_crash("agent", telemetry).await?;
        self.handle_crash("zerowrapper", telemetry).await?;
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
            escalate_to_wrapper(
                name,
                &format!("exceeded {} restart attempts", self.max_restarts),
                telemetry,
            )?;
            self.failures.insert(name.to_string(), 0);
            return Ok(());
        }
        self.running.remove(name);
        restart_program(&spec, telemetry).await?;
        // Re-register the new PID.
        if let Some(child_pid) = self.last_launched_pid(&spec) {
            let watched = watch_process(child_pid, name, telemetry)?;
            self.running.insert(name.to_string(), watched);
        }
        Ok(())
    }

    fn last_launched_pid(&self, spec: &ProgramSpec) -> Option<u32> {
        // Find the PID of the most recently spawned process matching the binary.
        let mut sys = sysinfo::System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let binary = std::path::Path::new(&spec.binary);
        let bin_name = binary.file_name().and_then(|n| n.to_str());
        sys.processes()
            .values()
            .filter(|p| match bin_name {
                Some(name) => p.name().eq_ignore_ascii_case(name),
                None => false,
            })
            .max_by_key(|p| p.start_time())
            .map(|p| p.pid().as_u32())
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
