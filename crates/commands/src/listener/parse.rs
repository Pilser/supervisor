//! Parse incoming commands.

use core::{Result, SupervisorError};
use serde::{Deserialize, Serialize};

/// A parsed wrapper command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingCommand {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

/// Parse a JSON command envelope.
pub fn parse_command(raw: &str) -> Result<IncomingCommand> {
    serde_json::from_str(raw).map_err(|e| SupervisorError::Command(format!("parse: {e}")))
}
