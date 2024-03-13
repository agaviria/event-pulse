use dirs;
use log::{debug, info};
use std::{fs, path::PathBuf};
use tracing::{error, span, Level};

use crate::error::ConfigError;

/// Checks and/or creates the local application data directory.
///
/// This asynchronous function checks if the `$XDG_DATA_HOME` or `$HOME/.local/share/$dir_name`
/// directory exists. If it exists, it returns the path to the directory. If not, it attempts
/// to create the directory and returns the path if successful.
///
/// # Arguments
///
/// * `dir_name` - A string slice containing the name of the directory to check or create.
///
/// # Returns
///
/// Returns a `Result` containing a `PathBuf` representing the path to the local application
/// data directory on success, or a `ConfigError` on failure.
///
/// # Errors
///
/// Returns an `AppError` if there are issues obtaining the local data directory, checking
/// or creating the specified directory.
///
/// # Examples
///
/// ```
/// use event_pulse::config;
///
/// async fn example() {
///     match config::local_app_data_dir("test.db").await {
///         Ok(path) => println!("Local app data directory: {:?}", path),
///         Err(e) => eprintln!("Error obtaining local app data directory: {}", e),
///     }
/// }
/// ```
pub async fn local_app_data_dir(dir_name: &str) -> Result<PathBuf, ConfigError> {
    let local_data_dir = dirs::data_local_dir().ok_or(ConfigError::LocalDataDirUnavailable)?;
    let local_app_data_path = local_data_dir.join(dir_name);

    // Check if directory exists
    if local_app_data_path.exists() {
        let _span = span!(Level::INFO, "LocalDataDirCheck").entered();
        info!(
            "Local application data directory '{}' already exists.",
            local_app_data_path.display()
        );
        return Ok(local_app_data_path);
    }

    // Attempt to create directory
    if let Err(err) = fs::create_dir(&local_app_data_path) {
        return Err(ConfigError::LocalAppDataDirCreationFailure(err.to_string()));
    }

    let _span = span!(Level::INFO, "LocalAppDataDirCreation").entered();
    info!(
        "Local application data directory '{}' created successfully.",
        local_app_data_path.display()
    );

    Ok(local_app_data_path)
}

/// Initializes the database data file.
///
/// This function retrieves the local application data directory, constructs the
/// database file path, and returns a PathBuf of the local data filepath.
///
/// # Arguments
///
/// * `db_filename` - A string slice containing the name of the database file.
///
/// # Returns
///
/// Returns a `Result` containing a `Structsy` instance representing the database
/// on success, or an `AppError` on failure.
///
/// # Errors
///
/// Returns an `AppError` if there are issues obtaining the local application data
/// directory, constructing the database file path, or opening/creating the database
/// file.
///
/// # Examples
///
/// ```
/// use event_pulse::config;
///
/// async fn example() {
///     match config::init_db_datafilepath("test.db").await {
///         Ok(db) => println!("Database initialized successfully: {:?}", db),
///         Err(e) => eprintln!("Error initializing database: {}", e),
///     }
/// }
/// ```
pub async fn init_db_datafilepath(db_filename: &str) -> Result<PathBuf, ConfigError> {
    // Start a new span for initialization
    let _span = span!(Level::INFO, "InitDatabaseFile").entered();

    // Retrieve package name from Cargo manifest
    let pkg_name = env!("CARGO_PKG_NAME");

    // Obtain local app data directory
    let data_dir = crate::config::local_app_data_dir(pkg_name)
        .await
        .map_err(|e| {
            error!("Failed to obtain local app data directory: {}", e);
            e
        })?;

    // Log data directory
    debug!(
        "Local app data directory obtained: '{}'",
        data_dir.to_string_lossy()
    );

    // Construct database file path
    let db_path = data_dir.join(db_filename);

    Ok(db_path)
}
