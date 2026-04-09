use std::fmt;

/// Unified error type for PM adapters.
#[derive(Debug)]
pub enum PmError {
    /// HTTP or network error.
    Http(String),
    /// API returned an error response.
    Api { status: u16, message: String },
    /// Serialization / deserialization error.
    Parse(String),
    /// Operation not supported by this adapter.
    UnsupportedOperation(String),
}

impl fmt::Display for PmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP error: {msg}"),
            Self::Api { status, message } => {
                write!(f, "API error ({status}): {message}")
            }
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::UnsupportedOperation(op) => {
                write!(f, "Unsupported operation: {op}")
            }
        }
    }
}

impl std::error::Error for PmError {}

impl From<reqwest::Error> for PmError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

pub type PmResult<T> = Result<T, PmError>;
