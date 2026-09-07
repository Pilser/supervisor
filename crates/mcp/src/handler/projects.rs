//! Projects tool handlers.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectOp {
    Create,
    Update,
    List,
    Get,
}

impl ProjectOp {
    pub fn method(self) -> &'static str {
        match self {
            ProjectOp::Create => "projects.create",
            ProjectOp::Update => "projects.update",
            ProjectOp::List => "projects.list",
            ProjectOp::Get => "projects.get",
        }
    }

    /// Wrapper request type for this op (writes vs reads).
    pub fn request_type(self) -> &'static str {
        match self {
            ProjectOp::Create | ProjectOp::Update => "db_write",
            ProjectOp::List | ProjectOp::Get => "db_read",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectArgs {
    pub name: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectArgs {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsArgs {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetProjectArgs {
    pub id: String,
}

pub struct ProjectToolHandler {
    pub op: ProjectOp,
    pub proxy: SharedProxy,
}

impl ProjectToolHandler {
    pub fn new(op: ProjectOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }

    fn check_args(op: ProjectOp, params: Value) -> anyhow::Result<()> {
        match op {
            ProjectOp::Create => {
                serde_json::from_value::<CreateProjectArgs>(params).map(|_| ())
            }
            ProjectOp::Update => {
                serde_json::from_value::<UpdateProjectArgs>(params).map(|_| ())
            }
            ProjectOp::List => serde_json::from_value::<ListProjectsArgs>(params).map(|_| ()),
            ProjectOp::Get => serde_json::from_value::<GetProjectArgs>(params).map(|_| ()),
        }
        .map_err(Into::into)
    }
}

impl ToolHandler for ProjectToolHandler {
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