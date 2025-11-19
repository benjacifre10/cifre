// src/presentation/screens/checks_screen.rs
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent}; // Importamos KeyModifiers
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;

pub struct ChecksScreen {
    selected_item: usize, // 0 for Todo, 1 for Releases
    items: Vec<&'static str>,
}

impl ChecksScreen {
    pub fn new() -> Self {
        Self {
            selected_item: 0,
            items: vec!["Todo", "Releases"],
        }
    }
}

impl Screen for ChecksScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        match key.code {
            KeyCode::Esc => Ok(ScreenOutcome::ChangeState(AppState::Home)), // Volver a Home
            KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit), // Salir de la app
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_item < self.items.len() - 1 {
                    self.selected_item += 1;
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Enter => {
                match self.selected_item {
                    0 => Ok(ScreenOutcome::ChangeState(AppState::ViewingTodo)),
                    1 => Ok(ScreenOutcome::ChangeState(AppState::ViewingReleases)),
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        // Recuadro principal de Checks
        let main_block_title = Line::from(vec![
            Span::styled("✅ ", Style::default().fg(Color::Cyan)),
            Span::styled("Checks", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)) // Siempre resaltado
            .title(main_block_title);
        f.render_widget(main_block.clone(), size); // Usa clone para obtener el inner_area

        let inner_area = main_block.inner(size);

        // Layout para los ítems
        let item_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); self.items.len()]) // Una línea por ítem
            .horizontal_margin(2) // Margen para que no toque los bordes
            .vertical_margin(2)
            .split(inner_area);

        for (i, item) in self.items.iter().enumerate() {
            let is_selected = i == self.selected_item;
            let item_style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let item_text = format!("  {} {}", if is_selected { "▶" } else { " " }, item);
            let paragraph = Paragraph::new(item_text).style(item_style);
            f.render_widget(paragraph, item_chunks[i]);
        }

        // Menú inferior
        let menu_block_title = Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
            Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let menu_text = Paragraph::new("Q: Quit | ESC: Back | ↑↓/JK: Navigate | Enter: Select")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .title(menu_block_title)
            );

        // Posicionar el menú en la parte inferior
        let menu_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.size())[1];
        f.render_widget(menu_text, menu_area);
    }
}
