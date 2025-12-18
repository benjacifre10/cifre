use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;

pub struct FlowsScreen {
    submenu_focused: bool,
}

impl FlowsScreen {
    pub fn new() -> Self {
        Self {
            submenu_focused: true,
        }
    }
}

impl Screen for FlowsScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
            KeyCode::Char('b') | KeyCode::Char('B') => {
                Ok(ScreenOutcome::ChangeState(AppState::Home))
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                // TODO: Load functionality
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // TODO: Add functionality
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                // TODO: Print functionality
                Ok(ScreenOutcome::Continue)
            }
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Bloque principal
        let title_block = Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(Color::LightBlue)),
            Span::styled("Flows", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title_block);

        let content = Paragraph::new("Flows content will go here")
            .block(main_block)
            .style(Style::default());

        f.render_widget(content, main_layout[0]);

        // Submenú (debajo del contenido, arriba del menú)
        let submenu_style = if self.submenu_focused {
            Style::default().fg(Color::LightCyan)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let submenu_text = Paragraph::new("L: Load | A: Add | P: Print")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(submenu_style)
                    .title(Span::styled(" ⚙ Submenu ", submenu_style))
            );
        f.render_widget(submenu_text, main_layout[1]);

        // Menú inferior
        let menu_block_title = Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
            Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        
        let menu_text = Paragraph::new("B: Back | Q: Quit")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .title(menu_block_title)
            );
        
        f.render_widget(menu_text, main_layout[2]);
    }
}
