use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GameEventWrapper {
    Known(Box<GameEvent>),
    Unknown(serde_json::Value),
}

#[derive(Debug, Deserialize, PartialEq)]
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
    DuelReplaceRoundPanorama {
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
    Subscribe {
        topic: String,
        client: String,
    },
    Unsubscribe {
        topic: String,
        client: String,
    },
    Subscribed {
        topic: String,
        level: String,
    },
    SubscribeToMatchmaking {
        topic: String,
        client: String,
        payload: MatchmakingSubscribe,
    },
    #[serde(rename_all = "camelCase")]
    MatchmakingJoined {
        topic: String,
        client: String,
        timestamp: String,
        access_token: Option<String>,
        payload: String, // WARN: It's MatchmakingJoinInfo. But for some reason, server gives it as shielded line
    },
    #[serde(rename_all = "camelCase")]
    MatchmakingMatched {
        topic: String,
        client: String,
        timestamp: String,
        access_token: Option<String>,
        payload: String, // WARN: It's MatchmakingMatchInfo. But for some reason, server gives it as shielded line
    },
    ChatEmote {
        client: String,
        topic: String,
        payload: String, // Emoji
    },
    #[serde(rename_all = "camelCase")]
    ChatMessage {
        topic: String,
        client: String,
        timestamp: String,
        access_token: Option<String>,
        payload: String, // WARN: It's ChatMessageInfo. But for some reason, server gives it as shielded line
    },
    ChatDisconnect {
        topic: String,
        client: String,
    },
}

impl GameEvent {
    pub fn get_player_panorama(&self) -> Result<Option<String>, ProtocolError> {
        let duel = match &self {
            Self::DuelStarted { duel, .. } => duel,
            Self::DuelNewRound { duel, .. } => duel,
            Self::DuelReplaceRoundPanorama { duel, .. } => duel,
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Duel {
    pub state: DuelState,
    pub pin: Option<Coordinates>, // WARN: Unknown Type
    pub from_pano_id: Option<serde_json::Value>, // WARN: Unknown Type
    pub location: Option<serde_json::Value>, // WARN: Unknown Type
    pub player_id: Option<serde_json::Value>, // WARN: Unknown Type
}

#[derive(Debug, Deserialize, PartialEq)]
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
    pub context: Option<serde_json::Value>, // WARN: Unknown Type
    pub movement_options: MovementOptions,
    pub map_bounds: MapBounds,
    pub initial_health: u32,
    pub max_number_of_rounds: u8,
    pub result: Option<GameResult>,
    pub is_paused: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub players: Vec<Player>,
    pub round_results: Vec<RoundResult>,
    pub current_multiplier: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[serde(rename = "playerId")]
    pub id: String,
    pub guesses: Vec<Guess>,
    pub rating: u32,
    pub country_code: String,
    pub progress_change: Option<ProgressChange>,
    pub pin: Option<Coordinates>,
    pub help_requested: bool,
    pub is_steam: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
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
    pub skipped_by_player_id: Option<serde_json::Value>, // WARN: Unknown Type
}
#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StateOptions {
    // pub allow_spectators: bool, // IDK if this is needed
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
    pub game_context: Option<serde_json::Value>, // WARN: Unknown Type
    pub round_starting_behavior: String,
    pub competitive_game_mode: String,
    pub count_all_guesses: bool,
    pub master_control_auto_start_rounds: bool,
    pub guess_map_type: String,
    pub progression_system: i32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MovementOptions {
    pub forbid_moving: bool,
    pub forbid_zooming: bool,
    pub forbid_rotating: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub name: String,
    pub slug: String,
    pub bounds: MapBounds,
    pub max_error_distance: i32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MapBounds {
    pub min: Coordinates,
    pub max: Coordinates,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Coordinates {
    #[serde(rename = "lat")]
    pub latitude: f64,
    #[serde(rename = "lng")]
    pub longitude: f64,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoundResult {
    pub round_number: u8,
    pub score: i32,
    pub health_before: i32,
    pub health_after: i32,
    pub best_guess: Option<Guess>,
    pub damage_dealt: i32,
    pub multiplier: f64,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AwardedXp {
    pub total_awarded_xp: i32,
    pub xp_awards: Vec<serde_json::Value>, // WARN: Unknown Type
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct RatingDelta {
    #[serde(rename = "ratingBefore")]
    pub before: i32,
    #[serde(rename = "ratingAfter")]
    pub after: i32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressChange {
    pub xp_progressions: Vec<serde_json::Value>, // WARN: Unknown Type
    pub awarded_xp: AwardedXp,
    pub medal: String,
    pub competitive_progress: Option<serde_json::Value>, // WARN: Unknown Type
    pub ranked_system_progress: Option<serde_json::Value>, // WARN: Unknown Type
    pub ranked_team_duels_progress: Option<serde_json::Value>, // WARN: Unknown Type
    pub quickplay_duels_progress: RatingDelta,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameResult {
    pub is_draw: bool,
    pub winning_team_id: String,
    pub winner_style: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LiveStreamData {
    pub time: u64,
    #[serde(flatten)]
    pub event: LiveStreamEventWrapper,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
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
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingSubscribe {
    pub game_modes: Vec<String>,
    pub queue: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingJoinInfo {
    pub team_members_in_matchmaking: Option<serde_json::Value>, // WARN: Unknown Type
    pub game_modes: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingMatchInfo {
    pub game_id: String,
    pub game_server_node_id: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageInfo {
    pub id: String,
    pub payload_type: String,
    pub text_payload: String,
    pub invite_payload: Option<serde_json::Value>, // WARN: Unknown Type
    pub recipient_type: String,
    pub recipient_id: Option<String>,
    pub source_type: String,
    pub source_id: String,
    pub sent_at: String,
    pub room_id: String,
    pub context: String,
    pub channel: Option<serde_json::Value>, // WARN: Unknown Type
    pub club_payload: Option<serde_json::Value>, // WARN: Unknown Type
    pub reaction_payload: Option<serde_json::Value>, // WARN: Unknown Type
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum LiveStreamEventWrapper {
    Known(LiveStreamEvent),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapBoundaries {
    pub north: f64,
    pub east: f64,
    pub south: f64,
    pub west: f64,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid HEX Pano ID. {0}")]
    InvalidHexPano(hex::FromHexError),

    #[error("Invalid UTF-8 Pano ID. {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_pano_pov() {
        let payload = r#"
{"code":"LiveStreamSamples","playerId":"5542d49bf27d472e20a8fb34","payload":[{"time":1779315641354,"type":"PanoPov","payload":{"heading":293.46143,"pitch":-8.943378}}]}
        "#;

        let actual: GameEventWrapper = serde_json::from_str(payload).unwrap();
        let expected = GameEventWrapper::Known(Box::new(GameEvent::LiveStreamSamples {
            player_id: "5542d49bf27d472e20a8fb34".to_string(),
            payload: vec![LiveStreamData {
                time: 1779315641354,
                event: LiveStreamEventWrapper::Known(LiveStreamEvent::PanoPov {
                    heading: 293.46143,
                    pitch: -8.943378,
                }),
            }],
        }));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_live_map() {
        let payload = r#"
        {"code":"LiveStreamSamples","playerId":"5542d49bf27d472e20a8fb34","payload":[{"time":1779315645724,"type":"MapDisplay","payload":{"isActive":true,"isSticky":false,"size":4}},{"time":1779315645752,"type":"MapBoundingBox","payload":{"north":82.515,"east":180,"south":-78.737625,"west":-180}},{"time":1779315645771,"type":"MapBoundingBox","payload":{"north":78.20225,"east":158.95335,"south":-72.295494,"west":-157.80446}},{"time":1779315645794,"type":"MapBoundingBox","payload":{"north":72.06741,"east":123.44553,"south":-63.243137,"west":-122.29664}},{"time":1779315645813,"type":"MapBoundingBox","payload":{"north":75.44777,"east":138.56273,"south":-68.21205,"west":-137.41383}},{"time":1779315645830,"type":"MapBoundingBox","payload":{"north":75.96844,"east":145.3303,"south":-68.98174,"west":-144.18141}},{"time":1779315645847,"type":"MapBoundingBox","payload":{"north":75.96844,"east":146.82443,"south":-68.98174,"west":-145.67554}}]}
        "#;

        let actual: GameEventWrapper = serde_json::from_str(payload).unwrap();
        let expected = GameEventWrapper::Known(Box::new(GameEvent::LiveStreamSamples {
            player_id: "5542d49bf27d472e20a8fb34".to_string(),
            payload: vec![
                LiveStreamData {
                    time: 1779315645724,
                    event: LiveStreamEventWrapper::Known(LiveStreamEvent::MapDisplay {
                        is_active: true,
                        is_sticky: false,
                        size: 4,
                    }),
                },
                LiveStreamData {
                    time: 1779315645752,
                    event: LiveStreamEventWrapper::Known(
                        LiveStreamEvent::MapBoundingBox {
                            north: 82.515,
                            east: 180.0,
                            south: -78.737625,
                            west: -180.0,
                        },
                    ),
                },
                LiveStreamData {
                    time: 1779315645771,
                    event: LiveStreamEventWrapper::Known(
                        LiveStreamEvent::MapBoundingBox {
                            north: 78.20225,
                            east: 158.95335,
                            south: -72.295494,
                            west: -157.80446,
                        },
                    ),
                },
                LiveStreamData {
                    time: 1779315645794,
                    event: LiveStreamEventWrapper::Known(
                        LiveStreamEvent::MapBoundingBox {
                            north: 72.06741,
                            east: 123.44553,
                            south: -63.243137,
                            west: -122.29664,
                        },
                    ),
                },
                LiveStreamData {
                    time: 1779315645813,
                    event: LiveStreamEventWrapper::Known(
                        LiveStreamEvent::MapBoundingBox {
                            north: 75.44777,
                            east: 138.56273,
                            south: -68.21205,
                            west: -137.41383,
                        },
                    ),
                },
                LiveStreamData {
                    time: 1779315645830,
                    event: LiveStreamEventWrapper::Known(
                        LiveStreamEvent::MapBoundingBox {
                            north: 75.96844,
                            east: 145.3303,
                            south: -68.98174,
                            west: -144.18141,
                        },
                    ),
                },
                LiveStreamData {
                    time: 1779315645847,
                    event: LiveStreamEventWrapper::Known(
                        LiveStreamEvent::MapBoundingBox {
                            north: 75.96844,
                            east: 146.82443,
                            south: -68.98174,
                            west: -145.67554,
                        },
                    ),
                },
            ],
        }));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_heartbeat() {
        let payload = r#"{"code":"HeartBeat"}"#;

        let actual: GameEventWrapper = serde_json::from_str(payload).unwrap();
        let expected = GameEventWrapper::Known(Box::new(GameEvent::HeartBeat));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_duel_started() {
        let payload = r#"{"code":"DuelStarted","gameId":"6a0e337ddb568a66ee576488","duel":{"state":{"gameId":"6a0e337ddb568a66ee576488","gameServerNodeId":"0c870f9bace54aa49579be97cdc29e81","teams":[{"id":"140595a3-713c-4700-b5ce-410ec3d09a5a","name":"blue","health":6000,"players":[{"playerId":"67ba281ce13441ba96170b1e","guesses":[],"rating":0,"countryCode":"ua","progressChange":null,"pin":null,"helpRequested":false,"isSteam":false}],"roundResults":[],"currentMultiplier":1},{"id":"8905940e-d676-4d33-965f-44008a9290fe","name":"red","health":6000,"players":[{"playerId":"5542d49bf27d472e20a8fb34","guesses":[],"rating":461,"countryCode":"ru","progressChange":null,"pin":null,"helpRequested":false,"isSteam":false}],"roundResults":[],"currentMultiplier":1}],"rounds":[{"roundNumber":1,"panorama":{"panoId":"4E324A683732617541384A4C706A50306F4A504E3677","lat":37.826655,"lng":-122.42289,"countryCode":"","heading":329.14108534270133,"pitch":-0.18209029651973196,"zoom":0},"hasProcessedRoundTimeout":false,"isHealingRound":false,"multiplier":1,"damageMultiplier":1,"startTime":"2026-05-20T22:19:46.0746169Z","endTime":null,"timerStartTime":null,"skippedByPlayerId":null}],"currentRoundNumber":1,"status":"Ongoing","version":3,"options":{"initialHealth":6000,"individualInitialHealth":false,"initialHealthTeamOne":6000,"initialHealthTeamTwo":6000,"roundTime":15,"maxRoundTime":0,"gracePeriodTime":1,"maxNumberOfRounds":0,"healingRounds":[5],"movementOptions":{"forbidMoving":false,"forbidZooming":false,"forbidRotating":false},"mapSlug":"6983611e411dbe3f3b2a8c5b","isRated":true,"map":{"name":"A Figsy World","slug":"6983611e411dbe3f3b2a8c5b","bounds":{"min":{"lat":-54.88672138251638,"lng":-177.39450713648608},"max":{"lat":78.23591049351379,"lng":178.5434054107903}},"maxErrorDistance":18499075},"roundsWithoutDamageMultiplier":1,"multiplierIncrement":0,"roundWinMultiplierIncrement":5,"disableHealing":true,"isTeamDuels":false,"gameContext":null,"roundStartingBehavior":"Default","competitiveGameMode":"StandardDuels","countAllGuesses":false,"masterControlAutoStartRounds":false,"guessMapType":"roadmap","progressionSystem":5},"context":null,"movementOptions":{"forbidMoving":false,"forbidZooming":false,"forbidRotating":false},"mapBounds":{"min":{"lat":-54.88672138251638,"lng":-177.39450713648608},"max":{"lat":78.23591049351379,"lng":178.5434054107903}},"initialHealth":6000,"maxNumberOfRounds":0,"result":null,"isPaused":false},"pin":null,"fromPanoId":null,"location":null,"playerId":null},"timestamp":"2026-05-20T22:19:42.1772555Z"}"#;

        let actual: GameEventWrapper = serde_json::from_str(payload).unwrap();
        let expected = GameEventWrapper::Known(Box::new(GameEvent::DuelStarted {
            game_id: "6a0e337ddb568a66ee576488".to_string(),
            duel: Duel {
                state: DuelState {
                    game_id: "6a0e337ddb568a66ee576488".to_string(),
                    game_server_node_id: "0c870f9bace54aa49579be97cdc29e81".to_string(),
                    teams: vec![
                        Team {
                            id: "140595a3-713c-4700-b5ce-410ec3d09a5a".to_string(),
                            name: "blue".to_string(),
                            health: 6000,
                            players: vec![Player {
                                id: "67ba281ce13441ba96170b1e".to_string(),
                                guesses: vec![],
                                rating: 0,
                                country_code: "ua".to_string(),
                                progress_change: None,
                                pin: None,
                                help_requested: false,
                                is_steam: false,
                            }],
                            round_results: vec![],
                            current_multiplier: 1.0,
                        },
                        Team {
                            id: "8905940e-d676-4d33-965f-44008a9290fe".to_string(),
                            name: "red".to_string(),
                            health: 6000,
                            players: vec![Player {
                                id: "5542d49bf27d472e20a8fb34".to_string(),
                                guesses: vec![],
                                rating: 461,
                                country_code: "ru".to_string(),
                                progress_change: None,
                                pin: None,
                                help_requested: false,
                                is_steam: false,
                            }],
                            round_results: vec![],
                            current_multiplier: 1.0,
                        },
                    ],
                    rounds: vec![Round {
                        round_number: 1,
                        panorama: Panorama {
                            id: "4E324A683732617541384A4C706A50306F4A504E3677"
                                .to_string(),
                            latitude: 37.826655,
                            longitude: -122.42289,
                            country_code: "".to_string(),
                            heading: 329.14108534270133,
                            pitch: -0.18209029651973196,
                            zoom: 0.0,
                        },
                        has_processed_round_timeout: false,
                        is_healing_round: false,
                        multiplier: 1.0,
                        damage_multiplier: 1.0,
                        start_time: "2026-05-20T22:19:46.0746169Z".to_string(),
                        end_time: None,
                        timer_start_time: None,
                        skipped_by_player_id: None,
                    }],
                    current_round_number: 1,
                    status: "Ongoing".to_string(),
                    version: 3,
                    options: StateOptions {
                        initial_health: 6000,
                        individual_initial_health: false,
                        initial_health_team_one: 6000,
                        initial_health_team_two: 6000,
                        round_time: 15,
                        max_round_time: 0,
                        grace_period_time: 1,
                        max_number_of_rounds: 0,
                        healing_rounds: vec![5],
                        movement_options: MovementOptions {
                            forbid_moving: false,
                            forbid_zooming: false,
                            forbid_rotating: false,
                        },
                        map_slug: "6983611e411dbe3f3b2a8c5b".to_string(),
                        is_rated: true,
                        map: Map {
                            name: "A Figsy World".to_string(),
                            slug: "6983611e411dbe3f3b2a8c5b".to_string(),
                            bounds: MapBounds {
                                min: Coordinates {
                                    latitude: -54.88672138251638,
                                    longitude: -177.39450713648608,
                                },
                                max: Coordinates {
                                    latitude: 78.23591049351379,
                                    longitude: 178.5434054107903,
                                },
                            },
                            max_error_distance: 18499075,
                        },
                        rounds_without_damage_multiplier: 1,
                        multiplier_increment: 0,
                        round_win_multiplier_increment: 5,
                        disable_healing: true,
                        is_team_duels: false,
                        game_context: None,
                        round_starting_behavior: "Default".to_string(),
                        competitive_game_mode: "StandardDuels".to_string(),
                        count_all_guesses: false,
                        master_control_auto_start_rounds: false,
                        guess_map_type: "roadmap".to_string(),
                        progression_system: 5,
                    },
                    context: None,
                    movement_options: MovementOptions {
                        forbid_moving: false,
                        forbid_zooming: false,
                        forbid_rotating: false,
                    },
                    map_bounds: MapBounds {
                        min: Coordinates {
                            latitude: -54.88672138251638,
                            longitude: -177.39450713648608,
                        },
                        max: Coordinates {
                            latitude: 78.23591049351379,
                            longitude: 178.5434054107903,
                        },
                    },
                    initial_health: 6000,
                    max_number_of_rounds: 0,
                    result: None,
                    is_paused: false,
                },
                pin: None,
                from_pano_id: None,
                location: None,
                player_id: None,
            },
            timestamp: "2026-05-20T22:19:42.1772555Z".to_string(),
        }));

        assert_eq!(actual, expected);
    }
}
