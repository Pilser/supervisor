//! Command priorities.

use serde::{Deserialize, Serialize};

/// Queue priority. High commands jump to the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
}
