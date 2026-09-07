//! History tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryOp {
    Log,
    Get,
}

impl HistoryOp {
    pub fn method(self) -> &'static str {
        match self {
            HistoryOp::Log => "history.log",
            HistoryOp::Get => "history.get",
        }
    }

    /// Wrapper request type for this op (writes vs reads).
    pub fn request_type(self) -> &'static str {
        match self {
            HistoryOp::Log => "db_write",
            HistoryOp::Get => "db_read",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LogHistoryArgs {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub duration_seconds: Option<u64>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetHistoryArgs {
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub since: Option<String>,
}

pub struct HistoryToolHandler {
    pub op: HistoryOp,
    pub proxy: SharedProxy,
}

impl HistoryToolHandler {
    pub fn new(op: HistoryOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: HistoryOp, params: Value) -> anyhow::Result<()> {
        match op {
            HistoryOp::Log => serde_json::from_value::<LogHistoryArgs>(params).map(|_| ()),
            HistoryOp::Get => serde_json::from_value::<GetHistoryArgs>(params).map(|_| ()),
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for HistoryToolHandler {
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