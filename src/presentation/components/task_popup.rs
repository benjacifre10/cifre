use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct TaskPopup {
    pub name: String,
    pub description: String,
    pub state: usize,
    pub priority: usize,
    pub tag: usize,
    pub finish_date: String,
    pub alert: bool,
    pub current_field: usize,
    pub states: Vec<String>,
    pub priorities: Vec<String>,
    pub tags: Vec<String>,
}

impl TaskPopup {
    pub fn new() -> Self {
        let states = std::fs::read_to_string("data/task_state.json")
            .and_then(|content| serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            .unwrap_or_else(|_| vec!["pending".to_string(), "progress".to_string(), "block".to_string(), "done".to_string()]);
        
        let priorities = std::fs::read_to_string("data/priority.json")
            .and_then(|content| serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            .unwrap_or_else(|_| vec!["low".to_string(), "medium".to_string(), "high".to_string(), "critical".to_string()]);

        let tags = std::fs::read_to_string("data/task_tag.json")
            .and_then(|content| serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            .unwrap_or_else(|_| vec!["personal".to_string(), "work".to_string()]);

        Self {
            name: String::new(),
            description: String::new(),
            state: 0,
            priority: 0,
            tag: 0,
            finish_date: String::new(),
            alert: false,
            current_field: 0,
            states,
            priorities,
            tags,
        }
    }

    pub fn draw(&self, f: &mut Frame, is_editing: bool) {
        let size = f.size();
        let popup_area = centered_rect(60, 60, size);
        
        f.render_widget(Clear, popup_area);
        
        let title = if is_editing { "Edit Task" } else { "Create New Task" };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);
        
        let constraints = if is_editing {
            vec![
                Constraint::Length(3), // Name
                Constraint::Length(4), // Description
                Constraint::Length(3), // Priority
                Constraint::Length(3), // Tag
                Constraint::Length(3), // Finish Date
                Constraint::Length(3), // Alert
                Constraint::Length(3), // Instructions
            ]
        } else {
            vec![
                Constraint::Length(3), // Name
                Constraint::Length(4), // Description
                Constraint::Length(3), // State
                Constraint::Length(3), // Priority
                Constraint::Length(3), // Tag
                Constraint::Length(3), // Finish Date
                Constraint::Length(3), // Alert
                Constraint::Length(3), // Instructions
            ]
        };
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut chunk_index = 0;

        // Name field
        let name_style = if self.current_field == 0 { Style::default().fg(Color::Yellow) } else { Style::default() };
        let name_widget = Paragraph::new(self.name.as_str())
            .block(Block::default().borders(Borders::ALL).title("Name").border_style(name_style));
        f.render_widget(name_widget, chunks[chunk_index]);
        chunk_index += 1;

        // Description field - usar wrap automático de ratatui con ancho ajustado
        let desc_style = if self.current_field == 1 { Style::default().fg(Color::Yellow) } else { Style::default() };
        
        // Calcular ancho disponible (restar 2 por los bordes)
        let available_width = chunks[chunk_index].width.saturating_sub(2) as usize;
        
        // Dividir descripción manualmente para controlar el ancho exacto
        let mut desc_lines = Vec::new();
        let chars: Vec<char> = self.description.chars().collect();
        for chunk in chars.chunks(available_width) {
            desc_lines.push(chunk.iter().collect::<String>());
        }
        let desc_display = desc_lines.join("\n");
        
        let desc_widget = Paragraph::new(desc_display)
            .block(Block::default().borders(Borders::ALL).title(format!("Description ({}/96)", self.description.len())).border_style(desc_style));
        f.render_widget(desc_widget, chunks[chunk_index]);
        chunk_index += 1;

        // State field (solo en modo creación)
        if !is_editing {
            let state_style = if self.current_field == 2 { Style::default().fg(Color::Yellow) } else { Style::default() };
            let state_text = format!("< {} >", self.states[self.state]);
            let state_widget = Paragraph::new(state_text)
                .block(Block::default().borders(Borders::ALL).title("State").border_style(state_style));
            f.render_widget(state_widget, chunks[chunk_index]);
            chunk_index += 1;
        }

        // Priority field
        let priority_field_index = if is_editing { 2 } else { 3 };
        let priority_style = if self.current_field == priority_field_index { Style::default().fg(Color::Yellow) } else { Style::default() };
        let priority_text = format!("< {} >", self.priorities[self.priority]);
        let priority_widget = Paragraph::new(priority_text)
            .block(Block::default().borders(Borders::ALL).title("Priority").border_style(priority_style));
        f.render_widget(priority_widget, chunks[chunk_index]);
        chunk_index += 1;

        // Tag field
        let tag_field_index = if is_editing { 3 } else { 4 };
        let tag_style = if self.current_field == tag_field_index { Style::default().fg(Color::Yellow) } else { Style::default() };
        let tag_text = format!("< {} >", self.tags[self.tag]);
        let tag_widget = Paragraph::new(tag_text)
            .block(Block::default().borders(Borders::ALL).title("Tag").border_style(tag_style));
        f.render_widget(tag_widget, chunks[chunk_index]);
        chunk_index += 1;

        // Finish Date field
        let date_field_index = if is_editing { 4 } else { 5 };
        let date_style = if self.current_field == date_field_index { Style::default().fg(Color::Yellow) } else { Style::default() };
        let date_widget = Paragraph::new(self.finish_date.as_str())
            .block(Block::default().borders(Borders::ALL).title("Finish Date (YYYY-MM-DD)").border_style(date_style));
        f.render_widget(date_widget, chunks[chunk_index]);
        chunk_index += 1;

        // Alert field
        let alert_field_index = if is_editing { 5 } else { 6 };
        let alert_style = if self.current_field == alert_field_index { Style::default().fg(Color::Yellow) } else { Style::default() };
        let alert_text = if self.alert { "Yes" } else { "No" };
        let alert_widget = Paragraph::new(format!("< {} >", alert_text))
            .block(Block::default().borders(Borders::ALL).title("Alert").border_style(alert_style));
        f.render_widget(alert_widget, chunks[chunk_index]);
        chunk_index += 1;

        // Instructions
        let instructions = Paragraph::new("Tab: Next | Shift+Tab: Previous\nh/l: Change Options | Enter: Save | Esc: Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(instructions, chunks[chunk_index]);
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
