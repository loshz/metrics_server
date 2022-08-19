use std::error::Error;
use std::fmt;

/// The error type for MetricsServer operations.
#[derive(Debug)]
pub enum ServerError {
    /// Represents an error while creating a new server.
    Create(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::Create(s) => write!(f, "error creating metrics server: {}", s),
        }
    }
}

impl Error for ServerError {}
