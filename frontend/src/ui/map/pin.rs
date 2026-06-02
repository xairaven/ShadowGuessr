use backend::api::Location;
use egui::{Response, Ui};
use walkers::{MapMemory, Plugin, Position, Projector};

#[derive(Debug)]
pub struct MapPin {
    position: Position,
    color: egui::Color32,
    radius: f32,
}

impl MapPin {
    pub fn with_location(location: &Location) -> Self {
        let position = walkers::lat_lon(location.latitude, location.longitude);

        Self {
            position,
            color: Default::default(),
            radius: 10.0,
        }
    }

    pub fn with_color(self, color: egui::Color32) -> Self {
        Self { color, ..self }
    }

    pub fn with_radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    pub fn player_pin(location: &Location) -> Self {
        Self::with_location(location)
            .with_radius(10.0)
            .with_color(egui::Color32::GREEN)
    }

    pub fn opponent_pin(location: &Location) -> Self {
        Self::with_location(location)
            .with_radius(10.0)
            .with_color(egui::Color32::RED)
    }
}

#[derive(Debug, Default)]
pub struct MapPins {
    pins: Vec<MapPin>,
}

impl MapPins {
    pub fn add(&mut self, pin: MapPin) {
        self.pins.push(pin);
    }
}

impl Plugin for MapPins {
    fn run(
        self: Box<Self>, ui: &mut Ui, response: &Response, projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        for pin in self.pins {
            // Projecting geographic coordinates into screen pixels
            let screen_position = projector.project(pin.position).to_pos2();
            // Pin radius in screen pixels
            let screen_radius = pin.radius;

            let hovered = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(screen_position) < screen_radius)
                .unwrap_or(false);

            // Bright pins
            let alpha_multiplier = if hovered { 1.0 } else { 0.8 };

            // Painting pin
            ui.painter().circle_filled(
                screen_position,
                screen_radius,
                pin.color.gamma_multiply(alpha_multiplier),
            );

            // Pin Stroke
            ui.painter().circle_stroke(
                screen_position,
                screen_radius,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );
        }
    }
}
