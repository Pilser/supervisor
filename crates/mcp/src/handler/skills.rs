//! Skills tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillOp {
    Add,
    Update,
    List,
    Get,
}

impl SkillOp {
    pub fn method(self) -> &'static str {
        match self {
            SkillOp::Add => "skills.add",
            SkillOp::Update => "skills.update",
            SkillOp::List => "skills.list",
            SkillOp::Get => "skills.get",
        }
    }

    /// Wrapper request type for this op (writes vs reads).
    pub fn request_type(self) -> &'static str {
        match self {
            SkillOp::Add | SkillOp::Update => "db_write",
            SkillOp::List | SkillOp::Get => "db_read",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddSkillArgs {
    pub name: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSkillArgs {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ListSkillsArgs {
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetSkillArgs {
    pub id: String,
}

pub struct SkillToolHandler {
    pub op: SkillOp,
    pub proxy: SharedProxy,
}

impl SkillToolHandler {
    pub fn new(op: SkillOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: SkillOp, params: Value) -> anyhow::Result<()> {
        match op {
            SkillOp::Add => serde_json::from_value::<AddSkillArgs>(params).map(|_| ()),
            SkillOp::Update => serde_json::from_value::<UpdateSkillArgs>(params).map(|_| ()),
            SkillOp::List => serde_json::from_value::<ListSkillsArgs>(params).map(|_| ()),
            SkillOp::Get => serde_json::from_value::<GetSkillArgs>(params).map(|_| ()),
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for SkillToolHandler {
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