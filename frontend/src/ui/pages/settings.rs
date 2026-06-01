use crate::config::Theme;
use crate::context::Context;
use crate::logs::LogLevel;
use egui::{Button, Grid};
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct SettingsPage {
    log_level: LogLevel,
    theme: Theme,
}

impl SettingsPage {
    pub fn new(context: &Context) -> Self {
        Self {
            log_level: context.settings.log_level,
            theme: context.settings.theme,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button(egui_phosphor::regular::KEY_RETURN).clicked() {
                context.ui_state.switch_to_main();
            }
            if ui.button(egui_phosphor::regular::FLOPPY_DISK).clicked() {
                self.save(context);
            }
        });

        Grid::new("SETTINGS_GRID").num_columns(3).show(ui, |ui| {
            self.log_level(ui, context);
            self.theme(ui, context);
        });
    }

    fn log_level(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let current_value = &mut self.log_level;
        let runtime_value = &mut context.settings.log_level;
        let is_settings_synchronized = current_value == runtime_value;

        ui.label("Log Level:");
        egui::ComboBox::from_id_salt("LOG_LEVEL_SETTING")
            .selected_text(current_value.to_string())
            .show_ui(ui, |ui| {
                for log_level in LogLevel::iter() {
                    ui.selectable_value(current_value, log_level, log_level.to_string());
                }
            });

        if ui
            .add_enabled(
                !is_settings_synchronized,
                Button::new(egui_phosphor::regular::KEY_RETURN),
            )
            .clicked()
        {
            *current_value = *runtime_value;
        }
        ui.end_row();
    }

    fn theme(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let current_value = &mut self.theme;
        let runtime_value = &mut context.settings.theme;
        let is_settings_synchronized = current_value == runtime_value;

        ui.label("Theme:");
        egui::ComboBox::from_id_salt("THEME_SETTING")
            .selected_text(current_value.to_string())
            .show_ui(ui, |ui| {
                for theme in Theme::iter() {
                    ui.selectable_value(current_value, theme, theme.to_string());
                }
            });

        if ui
            .add_enabled(
                !is_settings_synchronized,
                Button::new(egui_phosphor::regular::KEY_RETURN),
            )
            .clicked()
        {
            *current_value = *runtime_value;
        }
        ui.end_row();
    }

    fn save(&self, context: &mut Context) {
        context.settings.log_level = self.log_level;
        context.settings.theme = self.theme;
        context.save_settings();
    }
}
