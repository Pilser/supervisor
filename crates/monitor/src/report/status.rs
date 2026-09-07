//! Current status of all monitored programs.

use serde::{Deserialize, Serialize};

use crate::programs::ProgramRegistry;

/// Point-in-time status of one program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramStatus {
    pub name: String,
    pub running: bool,
    pub restarts: u64,
}

/// Snapshot status for every registered program.
pub fn current_status(registry: &ProgramRegistry) -> Vec<ProgramStatus> {
    registry
        .list()
        .into_iter()
        .map(|spec| ProgramStatus {
            name: spec.name.clone(),
            running: false,
            restarts: 0,
        })
        .collect()
}
