//! Memory tool handlers: typed args proxied to the wrapper.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

/// Which memory operation this handler performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOp {
    Save,
    Get,
    List,
    Search,
    Delete,
}

impl MemoryOp {
    pub fn method(self) -> &'static str {
        match self {
            MemoryOp::Save => "memory.save",
            MemoryOp::Get => "memory.get",
            MemoryOp::List => "memory.list",
            MemoryOp::Search => "memory.search",
            MemoryOp::Delete => "memory.delete",
        }
    }

    /// Wrapper request type for this op (writes vs reads).
    pub fn request_type(self) -> &'static str {
        match self {
            MemoryOp::Save | MemoryOp::Delete => "db_write",
            MemoryOp::Get | MemoryOp::List | MemoryOp::Search => "db_read",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveMemoryArgs {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub importance: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GetMemoryArgs {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMemoryArgs {
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMemoryArgs {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMemoryArgs {
    pub key: String,
}

/// Dispatches a memory tool call to the wrapper.
pub struct MemoryToolHandler {
    pub op: MemoryOp,
    pub proxy: SharedProxy,
}

impl MemoryToolHandler {
    pub fn new(op: MemoryOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    /// Validate the incoming params as typed args for this op.
    pub fn check_args(op: MemoryOp, params: Value) -> anyhow::Result<()> {
        match op {
            MemoryOp::Save => {
                serde_json::from_value::<SaveMemoryArgs>(params).map(|_| ())
            }
            MemoryOp::Get => {
                serde_json::from_value::<GetMemoryArgs>(params).map(|_| ())
            }
            MemoryOp::List => {
                serde_json::from_value::<ListMemoryArgs>(params).map(|_| ())
            }
            MemoryOp::Search => {
                serde_json::from_value::<SearchMemoryArgs>(params).map(|_| ())
            }
            MemoryOp::Delete => {
                serde_json::from_value::<DeleteMemoryArgs>(params).map(|_| ())
            }
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for MemoryToolHandler {
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