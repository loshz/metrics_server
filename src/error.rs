use std::error::Error;
use std::fmt;

/// The error type for MetricsServer operations.
#[derive(Debug)]
pub struct ServerError {
    details: String,
}

impl ServerError {
    /// Creates a new ServerError with additional details.
    pub fn new(msg: &str) -> ServerError {
        ServerError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ServerError {}
