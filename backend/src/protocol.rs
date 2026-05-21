use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(tag = "code")]
pub enum GameEvent {
    #[serde(rename_all = "camelCase")]
    SubscribeToLobby {
        game_id: String,
        player_id: String,
    },
    #[serde(rename_all = "camelCase")]
    SubscribeToLiveStream {
        game_id: String,
        player_id: String,
    },
    #[serde(rename_all = "camelCase")]
    DuelStarted {
        game_id: String,
        duel: Duel,
        timestamp: String,
    },
    #[serde(rename_all = "camelCase")]
    DuelNewRound {
        game_id: String,
        duel: Duel,
        timestamp: String,
    },
    #[serde(rename_all = "camelCase")]
    DuelPlayerGuessed {
        game_id: String,
        duel: Duel,
        timestamp: String,
    },
    #[serde(rename_all = "camelCase")]
    DuelPinPlaced {
        game_id: String,
        duel: Duel,
        timestamp: String,
    },
    #[serde(rename_all = "camelCase")]
    DuelRoundTimedOut {
        game_id: String,
        duel: Duel,
        timestamp: String,
    },
    #[serde(rename_all = "camelCase")]
    DuelFinished {
        game_id: String,
        duel: Duel,
        timestamp: String,
    },
    #[serde(rename_all = "camelCase")]
    LiveStreamSamples {
        player_id: String,
        payload: Vec<LiveStreamData>,
    },
    HeartBeat,

    // For logging purposes if something unknown met
    #[serde(untagged)]
    UnknownEvent(serde_json::Value),
}

impl GameEvent {
    pub fn get_current_panorama(&self) -> Result<Option<String>, ProtocolError> {
        let duel = match &self {
            Self::DuelStarted { duel, .. } => duel,
            Self::DuelNewRound { duel, .. } => duel,
            _ => return Ok(None),
        };

        let last_round = duel.state.rounds.last();
        let pano_id = match last_round {
            None => return Ok(None),
            Some(round) => round.panorama.decode_id()?,
        };

        Ok(Some(pano_id))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Duel {
    pub state: DuelState,
    pub pin: Option<Coordinates>, // TODO: Unknown Type
    pub from_pano_id: Option<serde_json::Value>, // TODO: Unknown Type
    pub location: Option<serde_json::Value>, // TODO: Unknown Type
    pub player_id: Option<serde_json::Value>, // TODO: Unknown Type
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuelState {
    pub game_id: String,
    pub game_server_node_id: String,
    pub teams: Vec<Team>,
    pub rounds: Vec<Round>,
    pub current_round_number: u8,
    pub status: String,
    pub version: i32,
    pub options: StateOptions,
    pub context: Option<serde_json::Value>, // TODO: Unknown Type
    pub movement_options: MovementOptions,
    pub map_bounds: MapBounds,
    pub initial_health: u32,
    pub max_number_of_rounds: u8,
    pub result: Option<GameResult>,
    pub is_paused: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub players: Vec<Player>,
    pub round_results: Vec<RoundResult>,
    pub current_multiplier: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: String,
    pub guesses: Vec<Guess>,
    pub rating: u32,
    pub country_code: String,
    pub progress_change: Option<ProgressChange>,
    pub pin: Option<Coordinates>,
    pub help_requested: bool,
    pub is_steam: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    pub round_number: u8,
    pub panorama: Panorama,
    pub has_processed_round_timeout: bool,
    pub is_healing_round: bool,
    pub multiplier: f64,
    pub damage_multiplier: f64,
    pub start_time: String,
    pub end_time: Option<String>,
    pub timer_start_time: Option<String>,
    pub skipped_by_player_id: Option<serde_json::Value>, // TODO: Unknown Type
}
#[derive(Debug, Deserialize)]
pub struct Panorama {
    #[serde(rename = "panoId")]
    pub id: String, // HEX
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lng")]
    pub longitude: f64,
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StateOptions {
    pub initial_health: u32,
    pub individual_initial_health: bool,
    pub initial_health_team_one: u32,
    pub initial_health_team_two: u32,
    pub round_time: u32,
    pub max_round_time: u32,
    pub grace_period_time: i32,
    pub max_number_of_rounds: u32,
    pub healing_rounds: Vec<i32>,
    pub movement_options: MovementOptions,
    pub map_slug: String,
    pub is_rated: bool,
    pub map: Map,
    pub rounds_without_damage_multiplier: i32,
    pub multiplier_increment: i32,
    pub round_win_multiplier_increment: i32,
    pub disable_healing: bool,
    pub is_team_duels: bool,
    pub game_context: Option<serde_json::Value>, // TODO: Unknown Type
    pub round_starting_behavior: String,
    pub competitive_game_mode: String,
    pub count_all_guesses: bool,
    pub master_control_auto_start_rounds: bool,
    pub guess_map_type: String,
    pub progression_system: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MovementOptions {
    pub forbid_moving: bool,
    pub forbid_zooming: bool,
    pub forbid_rotating: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub name: String,
    pub slug: String,
    pub bounds: MapBounds,
    pub max_error_distance: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapBounds {
    pub min: Coordinates,
    pub max: Coordinates,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Coordinates {
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lng")]
    pub longitude: f64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoundResult {
    pub round_number: u8,
    pub score: i32,
    pub health_before: i32,
    pub health_after: i32,
    pub best_guess: Guess,
    pub damage_dealt: i32,
    pub multiplier: f64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Guess {
    pub round_number: i32,
    pub lat: f64,
    pub lng: f64,
    pub distance: f64,
    pub created: String,
    pub is_teams_best_guess_on_round: bool,
    pub score: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AwardedXp {
    pub total_awarded_xp: i32,
    pub xp_awards: Vec<serde_json::Value>, // TODO: Unknown Type
}

#[derive(Deserialize, Debug, Clone)]
pub struct RatingDelta {
    #[serde(rename = "ratingBefore")]
    pub before: i32,
    #[serde(rename = "ratingAfter")]
    pub after: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressChange {
    pub xp_progressions: Vec<serde_json::Value>, // TODO: Unknown Type
    pub awarded_xp: AwardedXp,
    pub medal: String,
    pub competitive_progress: Option<serde_json::Value>, // TODO: Unknown Type
    pub ranked_system_progress: Option<serde_json::Value>, // TODO: Unknown Type
    pub ranked_team_duels_progress: Option<serde_json::Value>, // TODO: Unknown Type
    pub quickplay_duels_progress: RatingDelta,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameResult {
    pub is_draw: bool,
    pub winning_team_id: String,
    pub winner_style: String,
}

#[derive(Debug, Deserialize)]
pub struct LiveStreamData {
    pub time: u64,
    #[serde(flatten)]
    pub event: LiveStreamEvent,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum LiveStreamEvent {
    GuessWithLatLng {
        #[serde(rename = "lat")]
        latitude: f64,
        #[serde(rename = "lng")]
        longitude: f64,
    },
    MapBoundingBox {
        north: f64,
        east: f64,
        south: f64,
        west: f64,
    },
    MapDisplay {
        #[serde(rename = "isActive")]
        is_active: bool,
        #[serde(rename = "isSticky")]
        is_sticky: bool,
        size: i32,
    },
    PanoPosition {
        #[serde(rename = "lat")]
        latitude: f64,
        #[serde(rename = "lng")]
        longitude: f64,
        #[serde(rename = "panoId")]
        pano_id: String,
    },
    PanoPov {
        heading: f64,
        pitch: f64,
    },
    PanoZoom {
        zoom: f64,
    },
    PinPosition {
        #[serde(rename = "lat")]
        latitude: f64,
        #[serde(rename = "lng")]
        longitude: f64,
    },
    Timer {
        time: f64,
    },

    // For logging purposes if something unknown met
    #[serde(untagged)]
    UnknownTelemetry(serde_json::Value),
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid HEX Pano ID. {0}")]
    InvalidHexPano(hex::FromHexError),

    #[error("Invalid UTF-8 Pano ID. {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
}
