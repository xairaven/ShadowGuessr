use crate::config::Theme;
use crate::context::Context;
use crate::logs::LogLevel;
use egui::{Button, Grid};
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct SettingsPage {
    interface: String,
    keylog_path: String,
    log_level: LogLevel,
    map_api_key: String,
    theme: Theme,

    api_key_shown: bool,
}

impl SettingsPage {
    pub fn new(context: &Context) -> Self {
        Self {
            interface: context.settings.interface.clone(),
            keylog_path: context.settings.keylog_path.clone(),
            log_level: context.settings.log_level,
            map_api_key: context.settings.map_api_key.clone(),
            theme: context.settings.theme,

            api_key_shown: false,
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

        Grid::new("SETTINGS_GRID").num_columns(4).show(ui, |ui| {
            self.map_api_key(ui, context);
            self.interface(ui, context);
            self.keylog_path(ui, context);
            self.log_level(ui, context);
            self.theme(ui, context);
        });
    }

    fn map_api_key(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let current_value = &mut self.map_api_key;
        let runtime_value = &mut context.settings.map_api_key;
        let is_settings_synchronized = current_value == runtime_value;

        ui.label("Google Map API Key:");
        ui.add(
            egui::TextEdit::singleline(&mut *current_value).password(!self.api_key_shown),
        );
        if ui
            .button(if self.api_key_shown {
                egui_phosphor::regular::EYE_CLOSED
            } else {
                egui_phosphor::regular::EYE
            })
            .clicked()
        {
            self.api_key_shown = !self.api_key_shown;
        }

        if ui
            .add_enabled(
                !is_settings_synchronized,
                Button::new(egui_phosphor::regular::KEY_RETURN),
            )
            .clicked()
        {
            *current_value = runtime_value.clone();
        }
        ui.end_row();
    }

    fn interface(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let current_value = &mut self.interface;
        let runtime_value = &mut context.settings.interface;
        let is_settings_synchronized = current_value == runtime_value;

        ui.label("Interface:");
        ui.text_edit_singleline(current_value);

        if ui
            .add_enabled(
                !is_settings_synchronized,
                Button::new(egui_phosphor::regular::KEY_RETURN),
            )
            .clicked()
        {
            *current_value = runtime_value.clone();
        }
        ui.end_row();
    }

    fn keylog_path(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        let current_value = &mut self.keylog_path;
        let runtime_value = &mut context.settings.keylog_path;
        let is_settings_synchronized = current_value == runtime_value;

        ui.label("Key Log Path:");
        ui.text_edit_singleline(current_value);

        if ui
            .add_enabled(
                !is_settings_synchronized,
                Button::new(egui_phosphor::regular::KEY_RETURN),
            )
            .clicked()
        {
            *current_value = runtime_value.clone();
        }
        ui.end_row();
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
        context.settings.interface = self.interface.clone();
        context.settings.map_api_key = self.map_api_key.clone();
        context.settings.keylog_path = self.keylog_path.clone();
        context.settings.log_level = self.log_level;
        context.settings.theme = self.theme;
        context.save_settings();
    }
}
