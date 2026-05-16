use reqwest::blocking::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug)]
pub struct ApiClient {
    key: String,
    client: Client,
}

impl ApiClient {
    pub fn new(key: String) -> Self {
        Self {
            key,
            client: Client::new(),
        }
    }

    fn decode_pano(hex_pano: &str) -> Result<String, ApiError> {
        let bytes = hex::decode(hex_pano).map_err(ApiError::InvalidHexPano)?;
        String::from_utf8(bytes).map_err(ApiError::InvalidUtf8)
    }

    fn fetch_coordinates(&self, pano_id: &str) -> Result<Location, ApiError> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/streetview/metadata?pano={}&key={}",
            pano_id, self.key
        );

        let response: ApiResponse = self
            .client
            .get(&url)
            .send()
            .map_err(ApiError::RequestSend)?
            .json()
            .map_err(ApiError::Deserialize)?;

        Ok(response.location)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub copyright: String,
    pub date: String,
    pub pano_id: String,
    pub location: Location,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lng")]
    pub longitude: f64,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid HEX Pano ID. {0}")]
    InvalidHexPano(hex::FromHexError),

    #[error("Invalid UTF-8 Pano ID. {0}")]
    InvalidUtf8(std::string::FromUtf8Error),

    #[error("Request send failed. {0}")]
    RequestSend(reqwest::Error),

    #[error("Invalid JSON. {0}")]
    Deserialize(#[from] reqwest::Error),
}
