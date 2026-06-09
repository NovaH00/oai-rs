//! Unified error type for the crate.
//!
//! Wraps HTTP, serialization, and API-level error conditions into a
//! single [`Error`] enum that all public methods return.

use thiserror::Error;

/// Errors that can occur during API interaction.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to construct a valid HTTP header from the API key.
    #[error("invalid header: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    /// JSON serialization or deserialization failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP/transport-level failure.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned a response with no choices array.
    #[error("response contained no choices")]
    NoChoices,

    /// Response was missing the message content field.
    #[error("response contained no content")]
    NoContent,

    /// API returned an error message.
    #[error("api error: {0}")]
    Api(String),
}
