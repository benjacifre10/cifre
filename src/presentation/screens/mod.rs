// src/presentation/screens/mod.rs
pub mod screen;
pub mod home_screen;
pub mod versions_screen;
pub mod todo_screen;
pub mod releases_screen;
pub mod settings_screen;

use anyhow::Result; // Asegúrate de que Result esté importado
use crossterm::event::KeyEvent; // ¡Importar KeyEvent!
use ratatui::Frame; // ¡Importar Frame!

// Exportar el trait y el enum para fácil acceso
pub use screen::{Screen, ScreenContext, ScreenOutcome};

// Nuevo enum para manejar todas las pantallas
pub enum AllScreens {
    Home(home_screen::HomeScreen),
    Versions(versions_screen::VersionsScreen),
    Todo(todo_screen::TodoScreen),
    Releases(releases_screen::ReleasesScreen),
    Settings(settings_screen::SettingsScreen),
}

impl Screen for AllScreens {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        match self {
            AllScreens::Home(s) => s.handle_key_event(key),
            AllScreens::Versions(s) => s.handle_key_event(key),
            AllScreens::Todo(s) => s.handle_key_event(key),
            AllScreens::Releases(s) => s.handle_key_event(key),
            AllScreens::Settings(s) => s.handle_key_event(key),
        }
    }

    fn draw(&mut self, f: &mut Frame, context: &ScreenContext) {
        match self {
            AllScreens::Home(s) => s.draw(f, context),
            AllScreens::Versions(s) => s.draw(f, context),
            AllScreens::Todo(s) => s.draw(f, context),
            AllScreens::Releases(s) => s.draw(f, context),
            AllScreens::Settings(s) => s.draw(f, context),
        }
    }
}
