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

pub struct ArtifactDependenciesScreen {
    artifact_name: String,
}

impl ArtifactDependenciesScreen {
    pub fn new(artifact_name: String) -> Self {
        Self { artifact_name }
    }
}

impl Screen for ArtifactDependenciesScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
            KeyCode::Char('b') | KeyCode::Char('B') => {
                Ok(ScreenOutcome::ChangeState(AppState::ViewingArtifacts))
            }
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.size());

        let title_block = Line::from(vec![
            Span::styled("🔗 ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                format!("{} - dependencies", self.artifact_name),
                Style::default().add_modifier(Modifier::BOLD)
            ),
        ]);
        
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title_block);

        let content = Paragraph::new("Dependencies content will go here")
            .block(main_block);

        f.render_widget(content, main_layout[0]);

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
        
        f.render_widget(menu_text, main_layout[1]);
    }
}
