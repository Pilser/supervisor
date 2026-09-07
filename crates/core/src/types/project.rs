//! Projects — long-lived work items for a tenant.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    #[default]
    Active,
    Paused,
    Completed,
    Archived,
}

/// Free-form project domain label (e.g. `robotics`).
pub type ProjectDomain = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub status: ProjectStatus,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}