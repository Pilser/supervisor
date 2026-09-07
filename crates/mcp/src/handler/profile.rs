//! Profile tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileOp {
    Get,
    Update,
}

impl ProfileOp {
    pub fn method(self) -> &'static str {
        match self {
            ProfileOp::Get => "profile.get",
            ProfileOp::Update => "profile.update",
        }
    }

    /// Wrapper request type for this op (writes vs reads).
    pub fn request_type(self) -> &'static str {
        match self {
            ProfileOp::Get => "db_read",
            ProfileOp::Update => "db_write",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetProfileArgs {
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileArgs {
    #[serde(default)]
    pub username: Option<String>,
    pub updates: serde_json::Map<String, Value>,
}

pub struct ProfileToolHandler {
    pub op: ProfileOp,
    pub proxy: SharedProxy,
}

impl ProfileToolHandler {
    pub fn new(op: ProfileOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: ProfileOp, params: Value) -> anyhow::Result<()> {
        match op {
            ProfileOp::Get => serde_json::from_value::<GetProfileArgs>(params).map(|_| ()),
            ProfileOp::Update => serde_json::from_value::<UpdateProfileArgs>(params).map(|_| ()),
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for ProfileToolHandler {
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