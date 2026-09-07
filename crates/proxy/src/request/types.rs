//! Request types understood by the wrapper protocol.

use serde::{Deserialize, Serialize};

/// The category of a wrapper request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    /// A read against a tenant database.
    DbRead,
    /// A write against a tenant database.
    DbWrite,
    /// A supervision heartbeat.
    Heartbeat,
    /// A deployment artifact/config push.
    Deployment,
    /// A command targeted at the agent VM.
    Command,
}

impl RequestType {
    /// Wire string for this request type.
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::DbRead => "db_read",
            RequestType::DbWrite => "db_write",
            RequestType::Heartbeat => "heartbeat",
            RequestType::Deployment => "deployment",
            RequestType::Command => "command",
        }
    }

    /// Parse a wire string into a `RequestType`.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "db_read" | "dbread" | "read" => Some(RequestType::DbRead),
            "db_write" | "dbwrite" | "write" => Some(RequestType::DbWrite),
            "heartbeat" => Some(RequestType::Heartbeat),
            "deployment" => Some(RequestType::Deployment),
            "command" => Some(RequestType::Command),
            _ => None,
        }
    }

    /// Whether this request performs a read (cacheable).
    pub fn is_read(&self) -> bool {
        matches!(self, RequestType::DbRead)
    }
}