use crate::config::{Config, Theme};
use crate::errors::ClientError;
use crate::logs::LogLevel;
use crate::ui::pages::Page;
use backend::message::BackendMessage;
use crossbeam::channel::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct Context {
    pub config: Config,
    pub settings: RuntimeSettings,

    pub ui_state: UiState,

    // Channels for local UI errors
    pub errors_tx: Sender<ClientError>,
    pub errors_rx: Receiver<ClientError>,

    // Backend communication
    pub data_rx: Receiver<BackendMessage>,
    pub data_tx: Sender<BackendMessage>,
    pub exit_flag: Arc<AtomicBool>,
}

impl Context {
    pub fn new(config: Config) -> Self {
        let settings = RuntimeSettings::from(&config);
        let ui_state = UiState::default();

        let (errors_tx, errors_rx) = crossbeam::channel::unbounded();
        let (data_tx, data_rx) = crossbeam::channel::unbounded();
        let exit_flag = Arc::new(AtomicBool::new(false));

        Self {
            config,
            settings,
            ui_state,
            errors_tx,
            errors_rx,
            data_rx,
            data_tx,
            exit_flag,
        }
    }

    pub fn save_settings(&mut self) {
        self.config = Config::from(&self.settings);

        if let Err(error) = self.config.save_to_file() {
            let _ = self.errors_tx.try_send(error);
        }
    }
}

#[derive(Debug)]
pub struct RuntimeSettings {
    pub map_api_key: String,
    pub interface: String,
    pub keylog_path: String,

    pub log_level: LogLevel,
    pub theme: Theme,
}

impl From<&Config> for RuntimeSettings {
    fn from(config: &Config) -> Self {
        Self {
            map_api_key: config.map_api_key.clone(),
            interface: config.interface.clone(),
            keylog_path: config.keylog_path.clone(),

            log_level: config.log_level,
            theme: config.theme,
        }
    }
}

impl From<&RuntimeSettings> for Config {
    fn from(settings: &RuntimeSettings) -> Self {
        Self {
            map_api_key: settings.map_api_key.clone(),
            interface: settings.interface.clone(),
            keylog_path: settings.keylog_path.clone(),

            log_level: settings.log_level,
            theme: settings.theme,
        }
    }
}

#[derive(Debug, Default)]
pub struct UiState {
    pub page: Page,
}

impl UiState {
    pub fn switch_to_main(&mut self) {
        self.page = Page::Main;
    }

    pub fn switch_to_settings(&mut self) {
        self.page = Page::Settings;
    }

    pub fn switch_to_info(&mut self) {
        self.page = Page::Info;
    }
}
