//! Registry of programs the monitor watches.

use std::collections::HashMap;

use shared::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// How to launch and supervise one program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramSpec {
    pub name: String,
    pub binary: String,
    pub args: Vec<String>,
    pub restart_on_crash: bool,
}

/// Named set of programs under supervision.
#[derive(Debug, Default)]
pub struct ProgramRegistry {
    programs: HashMap<String, ProgramSpec>,
}

impl ProgramRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, spec: ProgramSpec, telemetry: &telemetry::Telemetry) -> Result<()> {
        if spec.binary.is_empty() {
            return Err(SupervisorError::Monitor(format!(
                "program {} has empty binary",
                spec.name
            )));
        }
        telemetry
            .builder()
            .level(telemetry::Level::Info)
            .kind("monitor.register")
            .category("monitor")
            .message(format!("registered program {}", spec.name))
            .emit();
        self.programs.insert(spec.name.clone(), spec);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&ProgramSpec> {
        self.programs
            .get(name)
            .ok_or_else(|| SupervisorError::NotFound(format!("program {name}")))
    }

    pub fn list(&self) -> Vec<&ProgramSpec> {
        self.programs.values().collect()
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.programs
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| SupervisorError::NotFound(format!("program {name}")))
    }
}
