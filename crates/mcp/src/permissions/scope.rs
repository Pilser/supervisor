//! What an agent/client session is allowed to access.

/// The access scope granted to a connected agent.
///
/// A `None` domain/tool list means unrestricted within the tenant; an empty
/// vec means nothing.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    /// The tenant this session belongs to, if restricted to one.
    pub tenant_id: Option<String>,
    /// The category this session is scoped to, if any.
    pub category_id: Option<String>,
    /// Domains the agent may touch. Empty = unrestricted.
    pub allowed_domains: Vec<String>,
    /// Tool names the agent may call. Empty = unrestricted.
    pub allowed_tools: Vec<String>,
}

impl Scope {
    /// A fully open scope (no domain/tool restriction).
    pub fn open() -> Self {
        Self::default()
    }

    /// Restrict the scope to a single tenant.
    pub fn for_tenant(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: Some(tenant_id.into()),
            ..Self::default()
        }
    }

    /// Permit the tool `name` (in addition to any configured allow-list).
    pub fn allow_tool(mut self, name: impl Into<String>) -> Self {
        self.allowed_tools.push(name.into());
        self
    }

    /// Permit the tool `name` (in addition to any configured allow-list).
    pub fn allow_domain(mut self, domain: impl Into<String>) -> Self {
        self.allowed_domains.push(domain.into());
        self
    }

    /// True if the given tool name is permitted.
    pub fn permits_tool(&self, tool: &str) -> bool {
        self.allowed_tools.is_empty() || self.allowed_tools.iter().any(|t| t == tool)
    }

    /// True if the given domain is permitted.
    pub fn permits_domain(&self, domain: &str) -> bool {
        self.allowed_domains.is_empty() || self.allowed_domains.iter().any(|d| d == domain)
    }
}