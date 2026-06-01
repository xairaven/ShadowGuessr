use crate::context::Context;
use crate::ui::pages::Page;
use crate::ui::pages::info::InfoPage;
use crate::ui::pages::main::MainPage;
use crate::ui::pages::settings::SettingsPage;

pub struct Workspace {
    main: MainPage,
    settings: SettingsPage,
    info: InfoPage,
}

impl Workspace {
    pub fn new(app_context: &Context, egui_context: egui::Context) -> Self {
        Self {
            main: MainPage::new(egui_context),
            settings: SettingsPage::new(app_context),
            info: InfoPage::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, context: &mut Context) {
        match &context.ui_state.page {
            Page::Main => self.main.show(ui, context),
            Page::Settings => self.settings.show(ui, context),
            Page::Info => self.info.show(ui, context),
        }
    }
}
