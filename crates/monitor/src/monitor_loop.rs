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
    /// Give up and escalate after this many restart failures.
    pub max_restarts: u32,
}

impl MonitorLoop {
    pub fn new(registry: ProgramRegistry) -> Self {
        Self {
            registry,
            running: HashMap::new(),
            children: HashMap::new(),
            agent_state_path: None,
            failures: HashMap::new(),
            max_restarts: 5,
        }
    }

    /// Set the path the agent writes its health state to.
    pub fn with_agent_state_path(mut self, path: PathBuf) -> Self {
        self.agent_state_path = Some(path);
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

    /// Run one health-check tick: probe every program, restart the dead ones.
    pub async fn tick(&mut self, telemetry: &telemetry::Telemetry) -> Result<()> {
        let mut crashed: Vec<String> = Vec::new();

        // Owned handles first: try_wait() reaps exited children (no zombies)
        // and reports the exit, race-free.
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
            if !crashed.contains(&name) {
                crashed.push(name);
            }
        }

        for (name, proc) in &self.running {
            // Already flagged via try_wait() above: pushing twice would
            // restart the program twice and duplicate the daemon.
            if crashed.contains(name) {
                continue;
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
            escalate_to_wrapper(
                name,
                &format!("exceeded {} restart attempts", self.max_restarts),
                telemetry,
            )?;
            self.failures.insert(name.to_string(), 0);
            return Ok(());
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
