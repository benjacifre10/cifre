use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::screen::{Screen, ScreenContext, ScreenOutcome};

pub struct BillsScreen;

impl BillsScreen {
    pub fn new() -> Self {
        BillsScreen
    }
}

impl Screen for BillsScreen {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<ScreenOutcome> {
        match key.code {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                Ok(ScreenOutcome::Quit)
            }
            crossterm::event::KeyCode::Char('b') | crossterm::event::KeyCode::Char('B') => {
                Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
            }
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let content_area = main_layout[0];
        let menu_area = main_layout[1];

        // Main content block
        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" 💳 Bills ", Style::default().fg(Color::Blue)));

        let content = Paragraph::new("Bills management coming soon...")
            .alignment(Alignment::Center)
            .block(content_block);

        f.render_widget(content, content_area);

        // Menu block
        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Menu ", Style::default().fg(Color::Green)));

        let menu_paragraph = Paragraph::new("B: Back | Q: Quit")
            .alignment(Alignment::Center)
            .block(menu_block);

        f.render_widget(menu_paragraph, menu_area);
    }
}
