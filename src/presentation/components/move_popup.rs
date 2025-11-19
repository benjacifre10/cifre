use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct MovePopup {
    pub states: Vec<String>,
    pub list_state: ListState,
}

impl MovePopup {
    pub fn new() -> Self {
        let states = std::fs::read_to_string("data/task_state.json")
            .and_then(|content| serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            .unwrap_or_else(|_| vec!["pending".to_string(), "progress".to_string(), "block".to_string(), "done".to_string()]);
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            states,
            list_state,
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.states.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.states.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn get_selected_state(&self) -> Option<String> {
        self.list_state.selected().map(|i| self.states[i].clone())
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.size();
        let popup_area = centered_rect(30, 40, size);
        
        f.render_widget(Clear, popup_area);
        
        let block = Block::default()
            .title("Move Task To")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(inner);

        let items: Vec<ListItem> = self.states
            .iter()
            .map(|state| ListItem::new(state.as_str()))
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("► ");

        f.render_stateful_widget(list, chunks[0], &mut self.list_state);

        let instructions = ratatui::widgets::Paragraph::new("j/k: Navigate | Enter: Move | Esc: Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(instructions, chunks[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
