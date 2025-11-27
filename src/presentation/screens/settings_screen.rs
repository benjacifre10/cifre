use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::fs;
use std::path::Path;

use super::screen::{Screen, ScreenContext, ScreenOutcome};

pub struct SettingsScreen {
    harlequin_config_path: String,
    show_notification: bool,
    notification_message: String,
    notification_is_error: bool,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            harlequin_config_path: String::new(),
            show_notification: false,
            notification_message: String::new(),
            notification_is_error: false,
        }
    }

    fn save_config(&mut self) {
        // Verificar si el archivo existe
        if !Path::new(&self.harlequin_config_path).exists() {
            self.notification_message = "Error: File does not exist".to_string();
            self.notification_is_error = true;
            self.show_notification = true;
            return;
        }

        // Crear directorio config si no existe
        if let Err(_) = fs::create_dir_all("config") {
            self.notification_message = "Error: Could not create config directory".to_string();
            self.notification_is_error = true;
            self.show_notification = true;
            return;
        }

        // Crear el JSON de configuración
        let config = serde_json::json!({
            "harlequin_config_path": self.harlequin_config_path
        });

        // Guardar el archivo
        match fs::write("config/settings.json", serde_json::to_string_pretty(&config).unwrap()) {
            Ok(_) => {
                self.notification_message = "Configuration saved successfully".to_string();
                self.notification_is_error = false;
                self.show_notification = true;
            }
            Err(_) => {
                self.notification_message = "Error: Could not save configuration".to_string();
                self.notification_is_error = true;
                self.show_notification = true;
            }
        }
    }
}

impl Screen for SettingsScreen {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<ScreenOutcome> {
        if self.show_notification {
            match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                    self.show_notification = false;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('b') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                crossterm::event::KeyCode::Char('q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                    self.save_config();
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char(c) => {
                    self.harlequin_config_path.push(c);
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Backspace => {
                    self.harlequin_config_path.pop();
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(size);

        let settings_area = main_layout[0];
        let help_area = main_layout[1];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" ⚙️ Settings ", Style::default().fg(Color::Blue)));

        // Crear el contenido con el campo de configuración
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .margin(2)
            .split(settings_area);

        let config_area = content_layout[0];

        // Renderizar el bloque principal
        f.render_widget(block, settings_area);

        // Campo de configuración de Harlequin
        let config_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Archivo configuracion harlequin", Style::default().fg(Color::Green)));

        let config_text = if self.harlequin_config_path.is_empty() {
            "<campo a llenar>"
        } else {
            &self.harlequin_config_path
        };

        let config_style = if self.harlequin_config_path.is_empty() {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        };

        let config_paragraph = Paragraph::new(Span::styled(config_text, config_style))
            .block(config_block);

        f.render_widget(config_paragraph, config_area);

        // Menú de ayuda
        let help_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" ⚙ Menu ", Style::default().fg(Color::Magenta)));

        let help_text = Paragraph::new("Q: Quit | B: Back | S: Save")
            .alignment(Alignment::Center)
            .block(help_block);

        f.render_widget(help_text, help_area);

        // Mostrar notificación si está activa
        if self.show_notification {
            self.draw_notification(f);
        }
    }
}

impl SettingsScreen {
    fn draw_notification(&self, f: &mut Frame) {
        use ratatui::widgets::Clear;
        
        let size = f.size();
        let popup_width = 50;
        let popup_height = 7;
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let (title, color) = if self.notification_is_error {
            (" ❌ Error ", Color::Red)
        } else {
            (" ✅ Success ", Color::Green)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(Span::styled(title, Style::default().fg(color)));

        // let paragraph = Paragraph::new(Line::from(&self.notification_message))
        let paragraph = Paragraph::new(Line::from(self.notification_message.as_str()))
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, popup_area);
    }
}
