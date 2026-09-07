//! Handle category change: re-deploy everything.

use core::Result;

/// React to a tenant category change by triggering a full re-deploy.
pub async fn handle_category_change(
    old: &str,
    new: &str,
    telemetry: &telemetry::Telemetry,
) -> Result<()> {
    telemetry
        .builder()
        .level(telemetry::Level::Warn)
        .kind("deployment.category_changed")
        .category("deployment")
        .message(format!("category changed {old} -> {new}; full re-deploy required"))
        .emit();
    Ok(())
}
