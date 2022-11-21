//! Error definition.

use thiserror::Error;

/// Possible errors
#[derive(Debug, Error)]
pub enum SoracomHarvestClientError {
    /// Failed to authenticate with auth key ID and auth key secret given.
    #[error("Failed to authenticate with auth key ID and auth key secret given")]
    Auth,

    /// Invalid limit is provided. It should be from 1 to 1000.
    #[error("Invalid limit is provided. It should be from 1 to 1000")]
    InvalidLimit,

    /// Transparent error from [`reqwest`](https://docs.rs/reqwest/latest/reqwest/) crate.
    #[error(transparent)]
    Request(#[from] reqwest::Error),

    /// Transparent error from [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) crate.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<SoracomHarvestClientError> for String {
    fn from(s: SoracomHarvestClientError) -> Self {
        s.to_string()
    }
}
