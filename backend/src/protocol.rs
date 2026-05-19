use serde::Deserialize;
use strum_macros::Display;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct GameEvent {
    pub code: GameCode,
    pub id: String,
    pub duel: Duel,
}

#[derive(Debug, Display, Deserialize, PartialEq)]
pub enum GameCode {
    DuelStarted,
    DuelNewRound,
}

impl GameCode {
    pub fn include_coordinates() -> Vec<GameCode> {
        vec![GameCode::DuelStarted, GameCode::DuelNewRound]
    }
}

#[derive(Debug, Deserialize)]
pub struct Duel {
    pub rounds: Vec<Round>,
}

#[derive(Debug, Deserialize)]
pub struct Round {
    #[serde(rename = "roundNumber")]
    pub round_number: u8,
    pub panorama: Panorama,
}

#[derive(Debug, Deserialize)]
pub struct Panorama {
    #[serde(rename = "panoId")]
    pub id: String, // HEX
    #[serde(rename = "lat")]
    pub latitude: f64, // Fake?
    #[serde(rename = "lng")]
    pub longitude: f64, // Fake?
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub heading: f64,
    pub pitch: f64,
    pub zoom: f64,
}

impl Panorama {
    pub fn decode_id(&self) -> Result<String, ProtocolError> {
        let bytes = hex::decode(&self.id).map_err(ProtocolError::InvalidHexPano)?;
        String::from_utf8(bytes).map_err(ProtocolError::InvalidUtf8)
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid HEX Pano ID. {0}")]
    InvalidHexPano(hex::FromHexError),

    #[error("Invalid UTF-8 Pano ID. {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
}
