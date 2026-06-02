use crate::api::{ApiClient, Location};
use crate::errors::BackendError;
use crate::message::BackendMessage;
use crate::protocol::{
    GameEvent, GameEventWrapper, LiveStreamEvent, LiveStreamEventWrapper, MapBoundaries,
};
use crate::sniffer::Sniffer;
use crossbeam::channel::Sender;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub mod api;
pub mod errors;
pub mod message;
pub mod protocol;
pub mod sniffer;

#[derive(Debug)]
pub struct DataProcessorBuilder {
    error_sender: Sender<BackendError>,
    data_sender: Sender<BackendMessage>,
    interface: String,
    keylog_path: String,
    map_api_key: String,

    exit_flag: Arc<AtomicBool>,
}

impl DataProcessorBuilder {
    pub fn new(data_sender: Sender<BackendMessage>, exit_flag: Arc<AtomicBool>) -> Self {
        let (error_sender, _) = crossbeam::channel::unbounded();

        Self {
            error_sender,
            data_sender,
            interface: "".to_string(),
            keylog_path: "".to_string(),
            map_api_key: "".to_string(),
            exit_flag,
        }
    }

    pub fn with_error_sender(self, sender: Sender<BackendError>) -> Self {
        Self {
            error_sender: sender,
            ..self
        }
    }

    pub fn with_interface(self, interface: String) -> Self {
        Self { interface, ..self }
    }

    pub fn with_keylog_path(self, keylog_path: String) -> Self {
        Self {
            keylog_path,
            ..self
        }
    }

    pub fn with_map_api_key(self, map_api_key: String) -> Self {
        Self {
            map_api_key,
            ..self
        }
    }

    pub fn build(self) -> DataProcessor {
        DataProcessor {
            data_sender: self.data_sender,
            error_sender: self.error_sender,
            api_client: ApiClient::new(self.map_api_key),
            interface: self.interface,
            keylog_path: self.keylog_path,
            exit_flag: self.exit_flag,
            my_player_id: None,
        }
    }
}

pub struct DataProcessor {
    data_sender: Sender<BackendMessage>,
    error_sender: Sender<BackendError>,
    api_client: ApiClient,
    interface: String,
    keylog_path: String,
    exit_flag: Arc<AtomicBool>,

    // Tracks the current user ID to filter incoming telemetry streams
    my_player_id: Option<String>,
}

impl DataProcessor {
    pub fn run(mut self) {
        thread::spawn(move || {
            let mut sniffer = match Sniffer::start(&self.interface, &self.keylog_path) {
                Ok(value) => value,
                Err(error) => {
                    log::error!("Failed to start sniffer: {}", error);
                    let _ = self.error_sender.send(BackendError::Sniffer(error));
                    return;
                },
            };

            let work_result = self.worker_loop(&mut sniffer);
            if let Err(error) = work_result {
                log::error!("{}", error);
                let _ = self.error_sender.send(error);
            }

            sniffer.wait_child();
        });
    }

    fn worker_loop(&mut self, sniffer: &mut Sniffer) -> Result<(), BackendError> {
        loop {
            if self.exit_flag.load(Ordering::Relaxed) {
                break;
            }
            let line = sniffer.read().map_err(BackendError::Sniffer)?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            let text_payload = parts.first().copied().unwrap_or("");
            let raw_payload = parts.get(1).copied().unwrap_or("");
            let masking_key = parts.get(2).copied().unwrap_or("");

            log::info!(
                "Received payload. TEXT: {}\nRAW: {}\nMASKING: {}",
                text_payload,
                raw_payload,
                masking_key
            );

            // Deciding what JSON we will parse
            let json_str = if !text_payload.is_empty() {
                // Incoming traffic (from the server) is already in plain text
                text_payload.to_string()
            } else if !raw_payload.is_empty()
                && !masking_key.is_empty()
                && let Ok(unmasked) =
                    Sniffer::unmask_websocket_payload(raw_payload, masking_key)
            {
                // Outcoming traffic (from the player), decoding
                log::info!("Tried to unmask. Result: {}", unmasked);
                unmasked
            } else {
                log::warn!("Received line with no valid payload: {}", line);
                continue;
            };

            let event_wrapper = match serde_json::from_str::<GameEventWrapper>(&json_str)
            {
                Ok(event) => event,
                Err(error) => {
                    log::debug!("Failed to parse: {}", error);
                    continue;
                },
            };
            let event = match event_wrapper {
                GameEventWrapper::Known(value) => *value,
                GameEventWrapper::Unknown(value) => {
                    log::warn!("DETECTED Unknown event type: {}", value);
                    continue;
                },
            };

            self.process_event(event)?;
        }

        Ok(())
    }

    fn process_event(&mut self, event: GameEvent) -> Result<(), BackendError> {
        // Attempt to extract new coordinates whenever round data updates
        if let Ok(Some(panorama_id)) = event.get_player_panorama()
            && let Some(location) = self.api_client.fetch_coordinates(&panorama_id)?
        {
            let message = BackendMessage::PlayerLocation(location);
            let _ = self.data_sender.send(message);
        }

        match event {
            GameEvent::DuelStarted { duel, .. }
            | GameEvent::DuelNewRound { duel, .. }
            | GameEvent::DuelFinished { duel, .. } => {
                // Pass state update
                let _ = self
                    .data_sender
                    .send(BackendMessage::GameStateUpdate(Box::new(duel.state)));
            },

            GameEvent::SubscribeToLiveStream { player_id, .. } => {
                // Save the identity upon connection to the lobby stream
                self.my_player_id = Some(player_id);
            },
            GameEvent::LiveStreamSamples { player_id, payload } => {
                let is_me = Some(&player_id) == self.my_player_id.as_ref();

                for data in payload {
                    let telemetry = match data.event {
                        LiveStreamEventWrapper::Known(telemetry) => telemetry,
                        LiveStreamEventWrapper::Unknown(value) => {
                            log::warn!("DETECTED Unknown telemetry type: {}", value);
                            continue;
                        },
                    };

                    match telemetry {
                        LiveStreamEvent::MapBoundingBox {
                            north,
                            east,
                            south,
                            west,
                        } if is_me => {
                            // Forward map bounds adjustments initiated by the user
                            let message = BackendMessage::MapSync(MapBoundaries {
                                north,
                                east,
                                south,
                                west,
                            });
                            let _ = self.data_sender.send(message);
                        },
                        LiveStreamEvent::PinPosition {
                            latitude,
                            longitude,
                        } if !is_me => {
                            // Forward marker placements made by the opponent
                            let location = Location {
                                latitude,
                                longitude,
                            };
                            let _ = self
                                .data_sender
                                .send(BackendMessage::OpponentPin(location));
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }

        Ok(())
    }
}
