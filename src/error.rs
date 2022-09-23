use std::error::Error;
use std::fmt;

/// The error type for MetricsServer operations.
#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    /// Represents an error encountered while creating a new server.
    Create(String),
    /// Represents an error encountered while stopping the server.
    Stop(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::Create(s) => write!(f, "error creating metrics server: {}", s),
            ServerError::Stop(s) => write!(f, "error stopping metrics server: {}", s),
        }
    }
}

impl Error for ServerError {}
