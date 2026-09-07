//! Agent memory entries stored via the wrapper.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    #[default]
    Preference,
    Fact,
    Goal,
    Progress,
    Skill,
    Relationship,
    Session,
    Other,
}

/// A single memory key/value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub key: String,
    pub value: String,
    pub category: MemoryCategory,
    pub importance: f64,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for querying memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub category: Option<MemoryCategory>,
    pub limit: Option<u32>,
    pub query: Option<String>,
}