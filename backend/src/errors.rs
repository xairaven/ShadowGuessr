use crate::api::ApiError;
use crate::protocol::ProtocolError;
use crate::sniffer::SnifferError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Google Maps API. {0}")]
    Api(#[from] ApiError),

    #[error("Sniffer. {0}")]
    Sniffer(#[from] SnifferError),

    #[error("GeoGuessr Protocol. {0}")]
    Protocol(#[from] ProtocolError),
}
