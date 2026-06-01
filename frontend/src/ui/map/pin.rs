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
            let position = pin.position;
            // Compute pixel radius for a 100-meter circle.
            let radius = pin.radius * projector.scale_pixel_per_meter(position);
            // Project it into the position on the screen.
            let position = projector.project(position).to_pos2();
            let hovered = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(position) < radius)
                .unwrap_or(false);
            ui.painter().circle_filled(
                position,
                radius,
                pin.color.gamma_multiply(if hovered { 0.5 } else { 0.2 }),
            );
        }
    }
}
