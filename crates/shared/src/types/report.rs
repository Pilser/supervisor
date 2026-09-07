//! Learner progress reports.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportPeriod {
    #[default]
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub content: String,
    pub highlights: Option<String>,
    pub areas_to_improve: Option<String>,
    pub generated_at: String,
}