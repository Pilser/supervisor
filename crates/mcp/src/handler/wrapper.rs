//! Wrapper-backed [`DataProxy`]: forwards canonical tool envelopes to the
//! wrapper over the proxy crate's TCP wire, stamping this VM's tenant id.

use std::sync::Arc;

use serde_json::Value;

use super::{DataProxy, ProxyFuture};

/// `DataProxy` that sends every envelope to the wrapper and stamps
/// `tenant_id` when the envelope carries none (or `"local"`).
pub struct WrapperProxy {
    client: proxy::ProxyClient,
    tenant_id: String,
}

impl WrapperProxy {
    pub fn new(
        address: &str,
        port: u16,
        token: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            client: proxy::ProxyClient::new(address, port, token)?,
            tenant_id: tenant_id.to_string(),
        }))
    }
}

impl DataProxy for WrapperProxy {
    fn send(&self, mut json: Value) -> ProxyFuture<'static> {
        let needs_stamp = json
            .get("tenant_id")
            .and_then(Value::as_str)
            .map(|t| t.is_empty() || t == super::LOCAL_TENANT_ID)
            .unwrap_or(true);
        if needs_stamp {
            if let Some(obj) = json.as_object_mut() {
                obj.insert(
                    "tenant_id".to_string(),
                    Value::String(self.tenant_id.clone()),
                );
            }
        }
        let client = self.client.clone();
        Box::pin(async move { client.send_json(json).await })
    }
}
