use std::fmt;
use structsy::StructsyError;
use thiserror::Error; // Importing the `Error` trait and derive macro from the `thiserror` crate

// Define a wrapper struct around StructsyError so that we can create our own PartialEq trait.
// PartialEq trait is important for our application, as it is required for testing purposes.
#[derive(Debug)]
pub struct StructsyErrWrapper(pub StructsyError);

impl std::error::Error for StructsyErrWrapper {}

impl PartialEq for StructsyErrWrapper {
    fn eq(&self, other: &Self) -> bool {
        // Implement PartialEq based on our testing requirements
        // For simplicity, let's assume equality if the error messages are the same.
        self.0.to_string() == other.0.to_string()
    }
}

impl fmt::Display for StructsyErrWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Defines application logic error types.
#[derive(Debug, PartialEq, Error)]
pub enum AppError {
    #[error("Structsy error: {0}")]
    StructsyError(#[from] StructsyErrWrapper),

    #[error("Invalid string format: {0}")]
    InvalidInputString(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Defines configuration setting error types.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// returns an error if, `data direcotry` is not available.
    #[error("Failed to get the data directory.")]
    LocalDataDirUnavailable,

    /// returns an error if, `local data directory` could not be created.
    #[error("Failed to create directory: {0}")]
    LocalAppDataDirCreationFailure(String),
}
