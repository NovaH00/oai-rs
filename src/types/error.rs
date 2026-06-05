use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid header: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("response contained no choices")]
    NoChoices,

    #[error("api error: {0}")]
    Api(String),
}
