//! Shared domain types for the supervisor.

pub mod agent;
pub mod category;
pub mod deployment;
pub mod history;
pub mod memory;
pub mod project;
pub mod report;
pub mod skill;
pub mod tenant;

pub use agent::AgentInfo;
pub use category::CategoryInfo;
pub use deployment::{DeploymentArtifact, DeploymentManifest};
pub use history::{HistoryAction, HistoryEntry};
pub use memory::{MemoryCategory, MemoryEntry, MemoryQuery};
pub use project::{Project, ProjectDomain, ProjectStatus};
pub use report::{Report, ReportPeriod};
pub use skill::{Skill, SkillDomain, SkillLevel};
pub use tenant::TenantContext;