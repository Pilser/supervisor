//! Enforcement: apply a `Scope` when dispatching tools.

use anyhow::{anyhow, bail};

use super::Scope;

/// A simple access decision outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    Denied,
}

/// Non-fallible check of whether `tool` may be invoked under `scope`.
pub fn can_call(scope: &Scope, tool: &str) -> bool {
    scope.permits_tool(tool)
}

/// Fallible check returning a `Decision`, erroring when denied.
pub fn enforce(scope: &Scope, tool: &str) -> anyhow::Result<Decision> {
    if can_call(scope, tool) {
        Ok(Decision::Allowed)
    } else {
        bail!("permission denied: tool `{tool}` is not in the agent scope");
    }
}

/// Fallible enforcement that maps to `anyhow::Result<()>`, suitable for dispatch.
pub fn enforce_result(scope: &Scope, tool: &str) -> anyhow::Result<()> {
    enforce(scope, tool).map(|_| ())
}

/// Assert a tenant match, failing when the scope pins a different tenant.
pub fn enforce_tenant(scope: &Scope, tenant_id: &str) -> anyhow::Result<()> {
    match &scope.tenant_id {
        Some(pinned) if pinned != tenant_id => {
            Err(anyhow!("permission denied: tenant `{tenant_id}` not in scope"))
        }
        _ => Ok(()),
    }
}