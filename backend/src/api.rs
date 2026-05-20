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

    pub fn fetch_coordinates(&self, pano_id: &str) -> Result<Location, ApiError> {
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

        if response.status != "OK" {
            return Err(ApiError::BadStatus(response.status));
        }

        response
            .location
            .ok_or_else(|| ApiError::BadStatus("Missing location data".to_string()))
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
