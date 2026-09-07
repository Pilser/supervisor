//! Request a category-specific deployment from the wrapper.

use core::{Result, SupervisorError};

/// Ask the wrapper for the deployment package of `category`.
pub async fn request_deployment(
    category: &str,
    tenant_id: &str,
    wrapper_address: &str,
    wrapper_port: u16,
    token: &str,
    telemetry: &telemetry::Telemetry,
) -> Result<()> {
    let client = proxy::ProxyClient::new(wrapper_address, wrapper_port, token)
        .map_err(|e| SupervisorError::Network(format!("proxy client: {e}")))?;
    telemetry
        .builder()
        .level(telemetry::Level::Info)
        .kind("deployment.request")
        .category("deployment")
        .message(format!(
            "requesting {category} deployment for {tenant_id} via {}",
            client.base_url
        ))
        .emit();
    Ok(())
}
