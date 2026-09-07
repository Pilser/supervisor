//! Configs tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigOp {
    Get,
    Set,
}

impl ConfigOp {
    pub fn method(self) -> &'static str {
        match self {
            ConfigOp::Get => "configs.get",
            ConfigOp::Set => "configs.set",
        }
    }

    /// Wrapper request type for this op (writes vs reads).
    pub fn request_type(self) -> &'static str {
        match self {
            ConfigOp::Get => "db_read",
            ConfigOp::Set => "db_write",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetConfigArgs {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct SetConfigArgs {
    pub key: String,
    pub value: Value,
}

pub struct ConfigToolHandler {
    pub op: ConfigOp,
    pub proxy: SharedProxy,
}

impl ConfigToolHandler {
    pub fn new(op: ConfigOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: ConfigOp, params: Value) -> anyhow::Result<()> {
        match op {
            ConfigOp::Get => serde_json::from_value::<GetConfigArgs>(params).map(|_| ()),
            ConfigOp::Set => serde_json::from_value::<SetConfigArgs>(params).map(|_| ()),
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for ConfigToolHandler {
    fn call(&self, params: Value) -> BoxFuture<'static, anyhow::Result<Value>> {
        let op = self.op;
        let method = self.op.method().to_string();
        let proxy = Arc::clone(&self.proxy);
        Box::pin(async move {
            Self::check_args(op, params.clone())?;
            let tenant_id = super::tenant_from_params(&params);
            super::proxy_call(proxy.as_ref(), op.request_type(), &tenant_id, &method, params)
                .await
        })
    }
}