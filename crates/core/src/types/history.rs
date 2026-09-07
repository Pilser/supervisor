//! Activity history entries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HistoryAction {
    #[default]
    Created,
    Updated,
    Deleted,
    Completed,
    Started,
    Viewed,
    Other,
}

/// One row in the tenant `history` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub session_id: Option<String>,
    pub action: HistoryAction,
    pub domain: Option<String>,
    pub summary: Option<String>,
    pub duration_seconds: Option<u64>,
    pub created_at: String,
}