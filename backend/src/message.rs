use crate::api::Location;
use crate::protocol::{DuelState, MapBoundaries};

// Defines all possible messages sent from backend to frontend
pub enum BackendMessage {
    // Contains the true coordinates fetched from Google API
    PlayerLocation(Location),
    // Contains the opponent's unconfirmed pin position
    OpponentPin(Location),
    // Contains map boundaries to synchronize the viewport zoom
    MapSync(MapBoundaries),
    // Game State
    GameStateUpdate(Box<DuelState>),
}
