pub mod info;
pub mod main;
pub mod settings;

#[derive(Debug, Default)]
pub enum Page {
    #[default]
    Main,
    Settings,
    Info,
}
