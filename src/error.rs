use thiserror::Error;

/// Custom error types for the Bungie API analyzer application
/// Provides comprehensive error handling for all potential failure points
#[derive(Error, Debug)]
pub enum BungieError {
    /// HTTP request errors from reqwest
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Bungie API specific errors
    #[error("Bungie API error: {message} (Error code: {error_code})")]
    ApiError {
        /// Human-readable error message from Bungie API
        message: String,
        /// Bungie API error code
        error_code: i32,
    },

    /// User not found errors
    #[error("User not found: {bungie_id}")]
    UserNotFound {
        /// The Bungie ID that was not found
        bungie_id: String,
    },

    /// Post-game carnage report not found
    #[error("Post-game carnage report not found for activity: {activity_id}")]
    PgcrNotFound {
        /// The activity ID that was not found
        activity_id: String,
    },

    /// Invalid activity type error
    #[error("Invalid activity type: {activity_type}")]
    InvalidActivityType {
        /// The invalid activity type provided
        activity_type: String,
    },

    /// Authentication errors
    #[error("Authentication failed: {message}")]
    AuthenticationError {
        /// Authentication error message
        message: String,
    },

    /// Rate limiting errors from Bungie API
    #[error("Rate limit exceeded. Try again in {retry_after} seconds")]
    RateLimitExceeded {
        /// Seconds to wait before retrying
        retry_after: u64,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigError {
        /// Configuration error message
        message: String,
    },

    /// Data validation errors
    #[error("Data validation failed: {field} - {message}")]
    ValidationError {
        /// Field that failed validation
        field: String,
        /// Validation error message
        message: String,
    },

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// URL parsing errors
    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),
}

/// Result type alias for the application
/// Simplifies error handling throughout the codebase
pub type Result<T> = std::result::Result<T, BungieError>;
