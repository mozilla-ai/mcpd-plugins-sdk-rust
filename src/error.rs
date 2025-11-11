use thiserror::Error;
use tonic::{Code, Status};

/// Error types for plugin operations.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Configuration error occurred during plugin initialization.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Server initialization or runtime error.
    #[error("Server error: {0}")]
    Server(String),

    /// Invalid input provided to the plugin.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Internal plugin error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// gRPC transport error.
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}

impl From<PluginError> for Status {
    fn from(err: PluginError) -> Self {
        match err {
            PluginError::Configuration(msg) => Status::new(Code::InvalidArgument, msg),
            PluginError::Server(msg) => Status::new(Code::Internal, msg),
            PluginError::InvalidInput(msg) => Status::new(Code::InvalidArgument, msg),
            PluginError::Internal(msg) => Status::new(Code::Internal, msg),
            PluginError::Io(err) => Status::new(Code::Internal, err.to_string()),
            PluginError::Transport(err) => Status::new(Code::Unavailable, err.to_string()),
        }
    }
}

/// Result type for plugin operations.
pub type Result<T> = std::result::Result<T, PluginError>;
