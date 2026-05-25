use crate::context::Context;
use backend::api::Location;
use backend::message::BackendMessage;
use backend::protocol::MapBoundaries;
use walkers::sources::OpenStreetMap;
use walkers::{HttpTiles, MapMemory};

pub struct Workspace {
    is_running: bool,

    // Map State
    tiles: HttpTiles,
    map_memory: MapMemory,

    // Backend data
    player_location: Option<Location>,
    opponent_pin: Option<Location>,
    map_bounds: Option<MapBoundaries>,
}

impl Workspace {
    pub fn new(_app_context: &Context, ui_context: egui::Context) -> Self {
        Self {
            is_running: false,

            tiles: HttpTiles::new(OpenStreetMap, ui_context),
            map_memory: MapMemory::default(),

            player_location: None,
            opponent_pin: None,
            map_bounds: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        self.show_main_content(ui);
    }

    fn show_main_content(&mut self, ui: &mut egui::Ui) {
        let map = walkers::Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            walkers::Position::new(0.0, 0.0),
        );

        // TODO: Pins

        ui.add(map);
    }

    fn poll_backend(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let mut needs_repaint = false;

        while let Ok(message) = context.data_rx.try_recv() {
            match message {
                BackendMessage::PlayerLocation(location) => {
                    self.player_location = Some(location);
                    needs_repaint = true;
                },
                BackendMessage::OpponentPin(loc) => {
                    self.opponent_pin = Some(loc);
                    needs_repaint = true;
                },
                BackendMessage::MapSync(boundaries) => {
                    self.map_bounds = Some(boundaries);
                    // TODO: Map Sync
                    needs_repaint = true;
                },
                BackendMessage::GameStateUpdate(duel) => {
                    // TODO: ...
                    needs_repaint = true;
                },
            }
        }

        if needs_repaint {
            ui.ctx().request_repaint();
        }
    }
}
