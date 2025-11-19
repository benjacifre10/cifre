// src/presentation/screens/screen.rs
use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::presentation::tui::AppState;

// Contexto que se pasa a las pantallas
pub struct ScreenContext {
    pub current_datetime: chrono::DateTime<chrono::Local>,
}

// Resultado de manejar un evento
pub enum ScreenOutcome {
    Continue, // La pantalla manejó el evento y no hay cambio de estado principal
    ChangeState(AppState), // La pantalla solicita un cambio de AppState
    Quit, // La pantalla solicita salir de la aplicación
}

pub trait Screen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome>;
    fn draw(&mut self, f: &mut Frame, context: &ScreenContext);
}
