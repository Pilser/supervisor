//! Reports tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportOp {
    Get,
    List,
}

impl ReportOp {
    pub fn method(self) -> &'static str {
        match self {
            ReportOp::Get => "reports.get",
            ReportOp::List => "reports.list",
        }
    }

    /// Wrapper request type for this op (reads only).
    pub fn request_type(self) -> &'static str {
        match self {
            ReportOp::Get | ReportOp::List => "db_read",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetReportArgs {
    pub period: String,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListReportsArgs {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

pub struct ReportToolHandler {
    pub op: ReportOp,
    pub proxy: SharedProxy,
}

impl ReportToolHandler {
    pub fn new(op: ReportOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: ReportOp, params: Value) -> anyhow::Result<()> {
        match op {
            ReportOp::Get => serde_json::from_value::<GetReportArgs>(params).map(|_| ()),
            ReportOp::List => serde_json::from_value::<ListReportsArgs>(params).map(|_| ()),
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for ReportToolHandler {
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