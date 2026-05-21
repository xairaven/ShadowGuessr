use crate::api::Location;

// Defines all possible messages sent from backend to frontend
pub enum BackendMessage {
    // Contains the true coordinates fetched from Google API
    PlayerLocation(Location),
    // Contains the opponent's unconfirmed pin position
    OpponentPin(Location),
    // Contains map boundaries to synchronize the viewport zoom
    MapSync {
        north: f64,
        east: f64,
        south: f64,
        west: f64,
    },
}
