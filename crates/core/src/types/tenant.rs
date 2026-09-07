//! Tenant context — which tenant this VM serves.

use serde::{Deserialize, Serialize};

/// Identity of the tenant this supervisor instance belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub vm_id: String,
    pub category_id: String,
    pub username: String,
}

impl TenantContext {
    pub fn new(
        tenant_id: impl Into<String>,
        vm_id: impl Into<String>,
        category_id: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            vm_id: vm_id.into(),
            category_id: category_id.into(),
            username: username.into(),
        }
    }
}