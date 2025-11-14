use std::fmt;

/// Result type for polymarket-rs operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for polymarket-rs
#[derive(Debug)]
pub enum Error {
    /// HTTP request failed
    Http(reqwest::Error),

    /// JSON serialization/deserialization failed
    Json(serde_json::Error),

    /// Invalid configuration
    Config(String),

    /// Authentication required but not provided
    AuthRequired(String),

    /// Signing operation failed
    Signing(String),

    /// Invalid parameter
    InvalidParameter(String),

    /// API error response
    Api { status: u16, message: String },

    /// Decimal conversion error
    Decimal(rust_decimal::Error),

    /// Invalid order configuration
    InvalidOrder(String),

    /// Missing required field
    MissingField(String),

    /// WebSocket connection error
    WebSocket(String),

    /// WebSocket connection closed
    ConnectionClosed,

    /// Reconnection failed after multiple attempts
    ReconnectFailed {
        attempts: u32,
        last_error: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::AuthRequired(msg) => write!(f, "Authentication required: {}", msg),
            Error::Signing(msg) => write!(f, "Signing error: {}", msg),
            Error::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Error::Api { status, message } => {
                write!(f, "API error (status {}): {}", status, message)
            }
            Error::Decimal(e) => write!(f, "Decimal error: {}", e),
            Error::InvalidOrder(msg) => write!(f, "Invalid order: {}", msg),
            Error::MissingField(field) => write!(f, "Missing required field: {}", field),
            Error::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            Error::ConnectionClosed => write!(f, "WebSocket connection closed"),
            Error::ReconnectFailed {
                attempts,
                last_error,
            } => write!(
                f,
                "Reconnection failed after {} attempts: {}",
                attempts, last_error
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::Decimal(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<rust_decimal::Error> for Error {
    fn from(err: rust_decimal::Error) -> Self {
        Error::Decimal(err)
    }
}

impl From<alloy_signer::Error> for Error {
    fn from(err: alloy_signer::Error) -> Self {
        Error::Signing(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(err.to_string())
    }
}
