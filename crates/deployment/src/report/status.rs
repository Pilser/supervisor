//! Report deployment status to the wrapper.

use shared::Result;

use crate::artifacts::manifest::DeploymentManifest;

/// Build the status payload for a finished deployment.
pub fn deployment_status(
    manifest: &DeploymentManifest,
    success: bool,
    message: &str,
) -> serde_json::Value {
    serde_json::json!({
        "tenant_id": manifest.tenant_id,
        "category": manifest.category,
        "success": success,
        "message": message,
        "artifact_count": manifest.artifacts.len(),
    })
}

/// Forward the status payload to the wrapper. Stub: emits telemetry.
pub async fn report_to_wrapper(status: &serde_json::Value, telemetry: &telemetry::Telemetry) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.report")
        .category("deployment")
        .message("deployment status reported")
        .data("status", status.clone())
        .emit();
    Ok(())
}
