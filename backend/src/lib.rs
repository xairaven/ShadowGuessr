use crate::api::ApiClient;
use crate::errors::BackendError;
use crate::message::BackendMessage;
use crate::protocol::{GameEvent, GameEventWrapper};
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
}

impl DataProcessor {
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
            let event_wrapper = match serde_json::from_str::<GameEventWrapper>(&payload) {
                Ok(event) => event,
                Err(_) => continue,
            };
            let event = match event_wrapper {
                GameEventWrapper::Known(value) => value,
                GameEventWrapper::Unknown(value) => {
                    log::warn!("DETECTED Unknown event type: {}", value);
                    continue;
                },
            };

            self.process_current_panorama(&event)?;
        }

        Ok(())
    }

    fn process_current_panorama(&self, event: &GameEvent) -> Result<(), BackendError> {
        match event.get_player_panorama()? {
            None => Ok(()),
            Some(panorama_id) => {
                let location = self.api_client.fetch_coordinates(&panorama_id)?;
                let message = BackendMessage::PlayerLocation(location);
                let _ = self.data_sender.send(message);
                Ok(())
            },
        }
    }
}
