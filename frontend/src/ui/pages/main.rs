use crate::context::Context;
use crate::errors::ClientError;
use crate::ui::map::pin::{MapPin, MapPins};
use backend::api::Location;
use backend::errors::BackendError;
use backend::message::BackendMessage;
use backend::protocol::{DuelState, MapBoundaries};
use crossbeam::channel::{Receiver, Sender};
use egui::{Color32, DragPanButtons, Grid, RichText};
use walkers::sources::OpenStreetMap;
use walkers::{HttpTiles, MapMemory};

pub struct MainPage {
    is_running: bool,

    // Map State
    tiles: HttpTiles,
    map_memory: MapMemory,

    // Backend data
    player_location: Option<Location>,
    opponent_pin: Option<Location>,
    map_bounds: Option<MapBoundaries>,
    game_state: Option<Box<DuelState>>,

    // Error channels
    backend_error_tx: Sender<BackendError>,
    backend_error_rx: Receiver<BackendError>,
}

impl MainPage {
    pub fn new(egui_context: egui::Context) -> Self {
        let (backend_error_tx, backend_error_rx) = crossbeam::channel::unbounded();

        Self {
            is_running: false,

            tiles: HttpTiles::new(OpenStreetMap, egui_context),
            map_memory: MapMemory::default(),

            player_location: None,
            opponent_pin: None,
            map_bounds: None,
            game_state: None,

            backend_error_tx,
            backend_error_rx,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        self.poll_backend(ui, context);

        egui::Panel::left("HUD_PANEL")
            .exact_size(200.0)
            .show_inside(ui, |ui| {
                self.show_hud(ui, context);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.show_map(ui);
        });

        while let Ok(err) = self.backend_error_rx.try_recv() {
            let _ = context.errors_tx.try_send(ClientError::Backend(err));
        }
    }

    fn show_hud(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("ShadowGuessr Intel");
        });

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(10.0);

                ui.vertical_centered_justified(|ui| match self.is_running {
                    false => {
                        if ui.button("START SNIFFER").clicked() {
                            backend::DataProcessorBuilder::new(
                                context.data_tx.clone(),
                                context.exit_flag.clone(),
                            )
                            .with_error_sender(self.backend_error_tx.clone())
                            .with_interface(context.settings.interface.clone())
                            .with_keylog_path(context.settings.keylog_path.clone())
                            .with_map_api_key(context.settings.map_api_key.clone())
                            .build()
                            .run();

                            self.is_running = true;
                        }
                    },
                    true => {
                        if ui.button("STOP SNIFFER").clicked() {
                            context
                                .exit_flag
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                            self.is_running = false;
                        }
                    },
                });

                ui.add_space(20.0);

                // Coordinates
                if let Some(location) = &self.player_location {
                    Grid::new("PLAYER_LOCATION_HUD")
                        .num_columns(2)
                        .show(ui, |ui| {
                            RichText::new("Panorama:").color(Color32::WHITE).size(14.0);
                            ui.end_row();

                            ui.label(RichText::new("Latitude:").color(Color32::WHITE));
                            ui.label(format!("{:.3}", location.latitude));
                            ui.end_row();
                            ui.label(RichText::new("Longitude:").color(Color32::WHITE));
                            ui.label(format!("{:.3}", location.longitude));
                            ui.end_row();
                        });
                    ui.add_space(10.0);
                }

                if let Some(location) = &self.opponent_pin {
                    Grid::new("OPPONENT_PIN_HUD").num_columns(2).show(ui, |ui| {
                        RichText::new("Opponent Pin:")
                            .color(Color32::WHITE)
                            .size(14.0);
                        ui.end_row();

                        ui.label(RichText::new("Latitude:").color(Color32::WHITE));
                        ui.label(format!("{:.3}", location.latitude));
                        ui.end_row();
                        ui.label(RichText::new("Longitude:").color(Color32::WHITE));
                        ui.label(format!("{:.3}", location.longitude));
                        ui.end_row();
                    });
                    ui.add_space(10.0);
                }

                // Match stats
                if let Some(state) = &self.game_state {
                    ui.label(
                        RichText::new(format!("Round: {}", state.current_round_number))
                            .color(Color32::WHITE)
                            .size(14.0),
                    );
                    ui.add_space(10.0);

                    for team in &state.teams {
                        Grid::new(format!("TEAM_{}_HUD", team.name))
                            .num_columns(2)
                            .show(ui, |ui| {
                                let team_name =
                                    format!("Team {}:", team.name.to_uppercase());
                                ui.label(RichText::new(team_name).color(Color32::WHITE));

                                let team_hp = format!("{} HP", team.health);
                                ui.label(team_hp);

                                ui.end_row();

                                ui.label(
                                    RichText::new("Multiplier: ").color(Color32::WHITE),
                                );
                                let multiplier = format!("{}x", team.current_multiplier);
                                ui.label(multiplier);
                                ui.end_row();
                            });

                        ui.add_space(5.0);
                    }
                } else {
                    ui.label("Waiting for duel to start...");
                }

                ui.add_space(20.0);

                // Navigation
                ui.vertical_centered_justified(|ui| {
                    if ui.button("Settings").clicked() {
                        context.ui_state.switch_to_settings();
                    }
                    if ui.button("Info").clicked() {
                        context.ui_state.switch_to_info();
                    }
                });
            });
    }

    const KYIV_LATITUDE: f64 = 50.4501;
    const KYIV_LONGITUDE: f64 = 30.5234;
    fn show_map(&mut self, ui: &mut egui::Ui) {
        let mut pins = MapPins::default();

        if let Some(location) = &self.player_location {
            pins.add(MapPin::player_pin(location));
        }
        if let Some(location) = &self.opponent_pin {
            pins.add(MapPin::opponent_pin(location));
        }

        let map = walkers::Map::new(
            Some(&mut self.tiles),
            &mut self.map_memory,
            walkers::lat_lon(Self::KYIV_LATITUDE, Self::KYIV_LONGITUDE),
        )
        .with_plugin(pins)
        .zoom_with_ctrl(false)
        .drag_pan_buttons(DragPanButtons::PRIMARY | DragPanButtons::SECONDARY);

        ui.add(map);
    }

    fn poll_backend(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let mut needs_repaint = false;

        while let Ok(message) = context.data_rx.try_recv() {
            match message {
                BackendMessage::PlayerLocation(location) => {
                    self.player_location = Some(location);
                    log::info!("Got player location: {:?}", self.player_location);
                },
                BackendMessage::OpponentPin(location) => {
                    self.opponent_pin = Some(location);
                    log::info!("Got opponent ping: {:?}", self.opponent_pin);
                },
                BackendMessage::MapSync(bounds) => {
                    // Center of rectangle
                    let center_lat = (bounds.north + bounds.south) / 2.0;
                    let center_lng = (bounds.east + bounds.west) / 2.0;

                    let center = walkers::lat_lon(center_lat, center_lng);
                    self.map_memory.center_at(center);

                    self.map_bounds = Some(bounds);
                },
                BackendMessage::GameStateUpdate(duel) => {
                    if duel.status == "Finished" || duel.status == "Canceled" {
                        self.game_state = None;
                        self.player_location = None;
                        self.opponent_pin = None;
                        self.map_bounds = None;
                        ui.ctx().request_repaint();
                        continue;
                    }

                    let is_new_round = match &self.game_state {
                        Some(old_state) => {
                            old_state.current_round_number != duel.current_round_number
                        },
                        None => true,
                    };

                    if is_new_round {
                        self.player_location = None;
                        self.opponent_pin = None;
                    }

                    self.game_state = Some(duel);
                },
            }

            needs_repaint = true;
        }

        if needs_repaint {
            ui.ctx().request_repaint();
        }
    }
}
