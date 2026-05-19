use crate::api::{ApiClient, Location};
use crate::errors::BackendError;
use crate::protocol::{GameCode, GameEvent};
use crate::sniffer::Sniffer;
use crossbeam::channel::Sender;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub mod api;
pub mod errors;
pub mod protocol;
pub mod sniffer;

#[derive(Debug)]
pub struct DataReceiverBuilder {
    error_sender: Sender<BackendError>,
    data_sender: Sender<Location>,
    interface: String,
    keylog_path: String,
    map_api_key: String,

    exit_flag: Arc<AtomicBool>,
}

impl DataReceiverBuilder {
    pub fn new(data_sender: Sender<Location>, exit_flag: Arc<AtomicBool>) -> Self {
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

    pub fn build(self) -> DataReceiver {
        DataReceiver {
            data_sender: self.data_sender,
            error_sender: self.error_sender,
            api_client: ApiClient::new(self.map_api_key),
            interface: self.interface,
            keylog_path: self.keylog_path,
            exit_flag: self.exit_flag,
        }
    }
}

pub struct DataReceiver {
    data_sender: Sender<Location>,
    error_sender: Sender<BackendError>,
    api_client: ApiClient,
    interface: String,
    keylog_path: String,

    exit_flag: Arc<AtomicBool>,
}

impl DataReceiver {
    pub fn run(self) {
        thread::spawn(move || {
            let mut sniffer = match Sniffer::start(&self.interface, &self.keylog_path) {
                Ok(value) => value,
                Err(error) => {
                    let _ = self.error_sender.send(BackendError::Sniffer(error));
                    return;
                },
            };

            let work_result = self.worker_loop(&mut sniffer);
            if let Err(error) = work_result {
                let _ = self.error_sender.send(error);
            }

            sniffer.wait_child();
        });
    }

    fn worker_loop(&self, sniffer: &mut Sniffer) -> Result<(), BackendError> {
        loop {
            if self.exit_flag.load(Ordering::Relaxed) {
                break;
            }
            let payload = sniffer.read().map_err(BackendError::Sniffer)?;
            let event: GameEvent = match serde_json::from_str(&payload) {
                Ok(event) => event,
                Err(error) => {
                    log::info!("Failed to parse payload: {}, error: {}", payload, error);
                    continue;
                },
            };
            if !GameCode::include_coordinates().contains(&event.code) {
                continue;
            }
            let last_round = event.duel.rounds.last();
            let pano_id = match last_round {
                None => continue,
                Some(round) => round.panorama.decode_id()?,
            };
            let location = self.api_client.fetch_coordinates(&pano_id)?;
            let _ = self.data_sender.send(location);
        }

        Ok(())
    }
}
