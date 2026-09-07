//! Error handling for wrapper responses and transport failures.

use serde_json::Value;

/// Error classes raised while talking to the wrapper.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ProxyError {
    /// A network-level failure talking to the wrapper.
    Network(String),
    /// The wrapper endpoint could not be reached at all.
    Unreachable(String),
    /// The request timed out.
    Timeout(String),
    /// A transport-level error (bad HTTP status, malformed body, ...).
    Transport(String),
    /// The supervisor was not recognised/authorised by the wrapper.
    Unauthorized(String),
    /// The wrapper response could not be parsed.
    Parse(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Network(m) => write!(f, "proxy network error: {m}"),
            ProxyError::Unreachable(m) => write!(f, "wrapper unreachable: {m}"),
            ProxyError::Timeout(m) => write!(f, "proxy timeout: {m}"),
            ProxyError::Transport(m) => write!(f, "proxy transport error: {m}"),
            ProxyError::Unauthorized(m) => write!(f, "proxy unauthorized: {m}"),
            ProxyError::Parse(m) => write!(f, "proxy parse error: {m}"),
        }
    }
}

impl std::error::Error for ProxyError {}

impl ProxyError {
    /// Stable classification string for telemetry.
    pub fn kind(&self) -> &'static str {
        match self {
            ProxyError::Network(_) => "network",
            ProxyError::Unreachable(_) => "unreachable",
            ProxyError::Timeout(_) => "timeout",
            ProxyError::Transport(_) => "transport",
            ProxyError::Unauthorized(_) => "unauthorized",
            ProxyError::Parse(_) => "parse",
        }
    }

    /// Whether retrying this error is likely to help (used to decide offline
    /// queueing).
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ProxyError::Network(_) | ProxyError::Unreachable(_) | ProxyError::Timeout(_)
        )
    }
}

impl From<reqwest::Error> for ProxyError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            ProxyError::Timeout(e.to_string())
        } else if e.is_connect() {
            ProxyError::Unreachable(e.to_string())
        } else if e.is_request() || e.is_redirect() {
            ProxyError::Network(e.to_string())
        } else {
            ProxyError::Transport(e.to_string())
        }
    }
}

/// Extract an error message from a wrapper error envelope, if any.
pub fn parse_error(value: &Value) -> Option<String> {
    value
        .get("error")
        .or_else(|| value.get("message"))
        .and_then(Value::as_str)
        .map(String::from)
}