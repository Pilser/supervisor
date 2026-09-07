//! Tenant tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantOp {
    GetInfo,
    GetCategory,
}

impl TenantOp {
    pub fn method(self) -> &'static str {
        match self {
            TenantOp::GetInfo => "tenant.info",
            TenantOp::GetCategory => "tenant.category",
        }
    }

    /// Wrapper request type for this op (reads only).
    pub fn request_type(self) -> &'static str {
        match self {
            TenantOp::GetInfo | TenantOp::GetCategory => "db_read",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetCategoryArgs {
    pub category_id: String,
}

pub struct TenantToolHandler {
    pub op: TenantOp,
    pub proxy: SharedProxy,
}

impl TenantToolHandler {
    pub fn new(op: TenantOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: TenantOp, params: Value) -> anyhow::Result<()> {
        match op {
            TenantOp::GetInfo => Ok(()),
            TenantOp::GetCategory => {
                serde_json::from_value::<GetCategoryArgs>(params).map(|_| ())
            }
            .map_err(Into::into)
        }
    }
}

impl ToolHandler for TenantToolHandler {
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