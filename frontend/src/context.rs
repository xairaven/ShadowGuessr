use crate::config::Config;
use crate::errors::ClientError;
use crossbeam::channel::{Receiver, Sender};

#[derive(Debug)]
pub struct Context {
    pub _config: Config,

    // Channels for local UI errors
    pub errors_tx: Sender<ClientError>,
    pub errors_rx: Receiver<ClientError>,
}

impl Context {
    pub fn new(_config: Config) -> Self {
        let (errors_tx, errors_rx) = crossbeam::channel::unbounded();

        Self {
            _config,
            errors_tx,
            errors_rx,
        }
    }
}
