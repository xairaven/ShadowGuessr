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

    pub fn fetch_coordinates(&self, pano_id: &str) -> Result<Option<Location>, ApiError> {
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

        if response.status == ApiStatus::ZeroResults.to_string() {
            log::warn!(
                "Failed to fetch coordinates. Zero Results. Response: {:?}. Panorama ID: {}",
                response,
                pano_id
            );
            return Ok(None);
        } else if response.status != ApiStatus::Ok.to_string() {
            return Err(ApiError::BadStatus(response.status));
        }

        Ok(response.location)
    }
}

#[derive(Debug)]
pub enum ApiStatus {
    Ok,
    ZeroResults,
}

impl std::fmt::Display for ApiStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let text = match self {
            ApiStatus::Ok => "OK",
            ApiStatus::ZeroResults => "ZERO_RESULTS",
        };

        write!(f, "{}", text)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub copyright: Option<String>,
    pub date: Option<String>,
    pub pano_id: Option<String>,
    pub location: Option<Location>,
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
    #[error("Google Maps API returned error status: {0}")]
    BadStatus(String),

    #[error("Request send failed. {0}")]
    RequestSend(reqwest::Error),

    #[error("Invalid JSON. {0}")]
    Deserialize(#[from] reqwest::Error),
}
