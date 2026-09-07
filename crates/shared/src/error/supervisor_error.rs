//! The `SupervisorError` type for fallible supervisor operations.

/// Coarse classification used by telemetry and plugin hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisorErrorKind {
    Config,
    Database,
    Network,
    Mcp,
    Monitor,
    Heartbeat,
    Command,
    Deployment,
    Transport,
    Plugin,
    NotFound,
    InvalidInput,
    Timeout,
    Unauthorized,
    Internal,
}

/// The primary error type of the supervisor.
#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("config error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("mcp error: {0}")]
    Mcp(String),

    #[error("monitor error: {0}")]
    Monitor(String),

    #[error("heartbeat error: {0}")]
    Heartbeat(String),

    #[error("command error: {0}")]
    Command(String),

    #[error("deployment error: {0}")]
    Deployment(String),

    #[error("transport error: {0}")]
    Transport(String),

    #[error("plugin error: {0}")]
    Plugin(String),

    #[error("{0} was not found")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("operation timed out: {0}")]
    Timeout(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

impl SupervisorError {
    pub fn kind(&self) -> SupervisorErrorKind {
        match self {
            SupervisorError::Config(_) => SupervisorErrorKind::Config,
            SupervisorError::Database(_) => SupervisorErrorKind::Database,
            SupervisorError::Network(_) => SupervisorErrorKind::Network,
            SupervisorError::Mcp(_) => SupervisorErrorKind::Mcp,
            SupervisorError::Monitor(_) => SupervisorErrorKind::Monitor,
            SupervisorError::Heartbeat(_) => SupervisorErrorKind::Heartbeat,
            SupervisorError::Command(_) => SupervisorErrorKind::Command,
            SupervisorError::Deployment(_) => SupervisorErrorKind::Deployment,
            SupervisorError::Transport(_) => SupervisorErrorKind::Transport,
            SupervisorError::Plugin(_) => SupervisorErrorKind::Plugin,
            SupervisorError::NotFound(_) => SupervisorErrorKind::NotFound,
            SupervisorError::InvalidInput(_) => SupervisorErrorKind::InvalidInput,
            SupervisorError::Timeout(_) => SupervisorErrorKind::Timeout,
            SupervisorError::Unauthorized(_) => SupervisorErrorKind::Unauthorized,
            SupervisorError::Io(_) | SupervisorError::Json(_) | SupervisorError::Internal(_) => {
                SupervisorErrorKind::Internal
            }
        }
    }
}

impl From<anyhow::Error> for SupervisorError {
    fn from(e: anyhow::Error) -> Self {
        SupervisorError::Internal(e.to_string())
    }
}

impl From<SupervisorErrorKind> for SupervisorError {
    fn from(k: SupervisorErrorKind) -> Self {
        SupervisorError::Internal(format!("{k:?}"))
    }
}

pub type Result<T> = std::result::Result<T, SupervisorError>;