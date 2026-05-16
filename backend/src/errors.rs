use crate::api::ApiError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Google Maps API. {0}")]
    Api(#[from] ApiError),
}
