use std::error::Error;
use std::fmt;
use std::result;

/// Custom result type for Clipper operations
pub type Result<T> = result::Result<T, ClipperError>;

/// Represents all possible errors that can occur in Clipper operations
#[derive(Debug)]
pub enum ClipperError {
    /// Error when trying to process open paths in an operation that requires closed paths
    OpenPathsNotSupported,
    /// Error when encountering invalid polygon data
    InvalidPolygon(String),
    /// Error when a required operation cannot be completed
    OperationFailure(String),
    /// Error when parameters are invalid
    InvalidParameter(String),
    /// Coordinate outside allowed range
    CoordinateOutOfRange,
    /// Error when trying to execute an operation while another is in progress
    ExecutionInProgress,
    /// Miscellaneous errors
    Other(String),
}

impl fmt::Display for ClipperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipperError::OpenPathsNotSupported => 
                write!(f, "Open paths are not supported for this operation"),
            ClipperError::InvalidPolygon(msg) => 
                write!(f, "Invalid polygon: {}", msg),
            ClipperError::OperationFailure(msg) => 
                write!(f, "Operation failed: {}", msg),
            ClipperError::InvalidParameter(msg) => 
                write!(f, "Invalid parameter: {}", msg),
            ClipperError::CoordinateOutOfRange => 
                write!(f, "Coordinate value outside allowed range"),
            ClipperError::ExecutionInProgress => 
                write!(f, "Cannot execute while another operation is in progress"),
            ClipperError::Other(msg) => 
                write!(f, "{}", msg),
        }
    }
}

impl Error for ClipperError {}

/// Helper function to create an InvalidPolygon error
pub fn invalid_polygon<T>(msg: &str) -> Result<T> {
    Err(ClipperError::InvalidPolygon(msg.to_string()))
}

/// Helper function to create an OperationFailure error
pub fn operation_failed<T>(msg: &str) -> Result<T> {
    Err(ClipperError::OperationFailure(msg.to_string()))
}

/// Helper function to create an InvalidParameter error
pub fn invalid_parameter<T>(msg: &str) -> Result<T> {
    Err(ClipperError::InvalidParameter(msg.to_string()))
}
