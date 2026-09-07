//! Learner skill tracking.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillLevel {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Free-form skill domain label.
pub type SkillDomain = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub level: SkillLevel,
    pub confidence: f64,
    pub last_practiced: Option<String>,
    pub created_at: String,
}