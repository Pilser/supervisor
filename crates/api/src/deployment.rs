//! Deployment handle: artifact and category hooks for plugins.

/// Plugin-facing handle for category deployments.
#[derive(Debug, Default)]
pub struct DeploymentApi {
    pub history: deployment::report::history::DeploymentHistory,
}

impl DeploymentApi {
    pub fn new() -> Self {
        Self {
            history: deployment::report::history::DeploymentHistory::new(100),
        }
    }

    /// Parse a deployment manifest without depending on receiver internals.
    pub fn parse(&self, raw: &str) -> shared::Result<deployment::DeploymentManifest> {
        deployment::parse_manifest(raw)
    }
}
