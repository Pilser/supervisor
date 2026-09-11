//! Wrapper-backed [`DataProxy`]: forwards canonical tool envelopes to the
//! wrapper over the proxy crate's TCP wire, stamping this VM's tenant id.

use std::sync::Arc;

use serde_json::Value;

use super::{DataProxy, ProxyFuture};

/// `DataProxy` that sends every envelope to the wrapper, stamping this VM's
/// `tenant_id` and embedding the supervisor bearer token into `payload`
/// (the TCP wire has no headers — this mirrors `ProxyClient::call`).
pub struct WrapperProxy {
    client: proxy::ProxyClient,
    tenant_id: String,
    token: String,
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
            token: token.to_string(),
        }))
    }
}

impl WrapperProxy {
    /// Ask the wrapper which dotted ops this tenant may list and call
    /// (`tools.granted` envelope). Returns `None` when unrestricted —
    /// the caller must then list everything.
    pub async fn granted_tools(&self) -> anyhow::Result<Option<Vec<String>>> {
        let envelope = serde_json::json!({
            "type": "tools.granted",
            "tenant_id": self.tenant_id,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
            // The TCP wire has no headers: bearer goes in payload like send().
            "payload": {"token": self.token},
        });
        let resp = self.client.send_json(envelope).await?;
        let payload = resp.get("payload");
        let restricted = payload
            .and_then(|p| p.get("restricted"))
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        if !restricted {
            return Ok(None);
        }
        Ok(Some(
            payload
                .and_then(|p| p.get("tools"))
                .and_then(|t| t.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        ))
    }
}

impl DataProxy for WrapperProxy {
    fn send(&self, mut json: Value) -> ProxyFuture<'static> {
        let needs_stamp = json
            .get("tenant_id")
            .and_then(Value::as_str)
            .map(|t| t.is_empty() || t == super::LOCAL_TENANT_ID)
            .unwrap_or(true);
        if let Some(obj) = json.as_object_mut() {
            if needs_stamp {
                obj.insert(
                    "tenant_id".to_string(),
                    Value::String(self.tenant_id.clone()),
                );
            }
            if let Some(payload) = obj.get_mut("payload").and_then(|p| p.as_object_mut()) {
                payload
                    .entry("token")
                    .or_insert_with(|| Value::String(self.token.clone()));
            }
        }
        let client = self.client.clone();
        Box::pin(async move { client.send_json(json).await })
    }
}
