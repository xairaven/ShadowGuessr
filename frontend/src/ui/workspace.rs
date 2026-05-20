use crate::context::Context;

pub struct Workspace;

impl Workspace {
    pub fn new(_context: &Context) -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, context: &mut Context) {}
}
