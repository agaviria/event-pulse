use structsy::StructsyError;
use thiserror::Error; // Importing the `Error` trait and derive macro from the `thiserror` crate

/// Defines application logic error types.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Structsy error: {0}")]
    StructsyError(#[from] StructsyError),

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
