//! Monitor handle: program registry access for plugins.

/// Plugin-facing handle over [`monitor::ProgramRegistry`].
#[derive(Debug, Default)]
pub struct MonitorApi {
    pub registry: monitor::ProgramRegistry,
    pub check_interval_secs: u64,
}

impl MonitorApi {
    pub fn new(check_interval_secs: u64) -> Self {
        Self {
            registry: monitor::ProgramRegistry::new(),
            check_interval_secs,
        }
    }

    /// Current status snapshot of every registered program.
    pub fn status(&self) -> Vec<monitor::ProgramStatus> {
        monitor::report::status::current_status(&self.registry)
    }
}
