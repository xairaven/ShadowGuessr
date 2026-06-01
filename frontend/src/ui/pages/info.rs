use crate::context::Context;

#[derive(Debug)]
pub struct InfoPage {
    version: semver::Version,
}

impl Default for InfoPage {
    fn default() -> Self {
        Self {
            version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(
                semver::Version {
                    major: 0,
                    minor: 0,
                    patch: 1,
                    pre: Default::default(),
                    build: Default::default(),
                },
            ),
        }
    }
}

impl InfoPage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button(egui_phosphor::regular::KEY_RETURN).clicked() {
                context.ui_state.switch_to_main();
            }
        });

        ui.add_space(50.0);
        ui.vertical_centered_justified(|ui| {
            ui.add(egui::Label::new(
                egui::RichText::new(format!("ShadowGuessr v{}", self.version))
                    .size(25.0)
                    .color(egui::Color32::GREEN),
            ));
            ui.label("GeoGuessr Protocol Parser.");

            ui.add_space(20.0);

            ui.label("Developer: Alex Kovalov");

            ui.add_space(20.0);

            ui.hyperlink_to(
                "Check out the code on GitHub!",
                "https://github.com/xairaven/ShadowGuessr",
            );
            ui.hyperlink_to(
                "*Latest release*",
                "https://github.com/xairaven/ShadowGuessr/releases",
            );
        });
    }
}
