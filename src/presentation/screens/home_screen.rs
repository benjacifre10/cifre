use anyhow::Result;
use std::fs;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::domain::models::{Release, Task};

pub struct HomeScreen {
    checks_focused: bool,
    notifications_focused: bool,
    tools_focused: bool,
    documentation_focused: bool,
    selected_item: usize,
    selected_tool: usize,
    selected_doc: usize,
    show_notifications_popup: bool,
    show_options_popup: bool,
    show_harlequin_popup: bool,
    show_helix_popup: bool,
    notification_count: usize,
    notifications: Vec<String>, // Lista de notificaciones del día
    harlequin_databases: Vec<String>, // Lista de bases de datos de Harlequin
    selected_harlequin_db: usize, // Índice de la base de datos seleccionada
    helix_file_entries: Vec<HelixFileEntry>,
    helix_selected_entry: usize,
    helix_scroll_offset: usize,
    helix_current_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
struct HelixFileEntry {
    path: std::path::PathBuf,
    name: String,
    is_dir: bool,
}

impl HomeScreen {
    pub fn new() -> Result<Self> {
        let mut screen = HomeScreen {
            checks_focused: false,
            notifications_focused: false,
            tools_focused: false,
            documentation_focused: false,
            selected_item: 0,
            selected_tool: 0,
            selected_doc: 0,
            show_notifications_popup: false,
            show_options_popup: false,
            show_harlequin_popup: false,
            show_helix_popup: false,
            notification_count: 0,
            notifications: Vec::new(),
            harlequin_databases: Vec::new(),
            selected_harlequin_db: 0,
            helix_file_entries: Vec::new(),
            helix_selected_entry: 0,
            helix_scroll_offset: 0,
            helix_current_dir: std::path::PathBuf::new(),
        };
        screen.load_notifications();
        Ok(screen)
    }

    pub fn new_with_focus() -> Result<Self> {
        let mut screen = HomeScreen {
            checks_focused: true,
            notifications_focused: false,
            tools_focused: false,
            documentation_focused: false,
            selected_item: 0,
            selected_tool: 0,
            selected_doc: 0,
            show_notifications_popup: false,
            show_options_popup: false,
            show_harlequin_popup: false,
            show_helix_popup: false,
            notification_count: 0,
            notifications: Vec::new(),
            harlequin_databases: Vec::new(),
            selected_harlequin_db: 0,
            helix_file_entries: Vec::new(),
            helix_selected_entry: 0,
            helix_scroll_offset: 0,
            helix_current_dir: std::path::PathBuf::new(),
        };
        screen.load_notifications();
        Ok(screen)
    }

    fn load_notifications(&mut self) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut notifications = Vec::new();

        // Cargar notificaciones de releases
        if let Ok(content) = fs::read_to_string("data/release.json") {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(releases_array) = json_value.get("releases").and_then(|v| v.as_array()) {
                    for release_value in releases_array {
                        if let Ok(release) = serde_json::from_value::<Release>(release_value.clone()) {
                            if release.date_qa == today {
                                notifications.push(format!("{} • QA Date", release.name));
                            }
                            if release.date_finish == today {
                                notifications.push(format!("{} • Finish Date", release.name));
                            }
                        }
                    }
                }
            }
        }

        // Cargar notificaciones de tasks
        if let Ok(content) = fs::read_to_string("data/task.json") {
            if let Ok(tasks) = serde_json::from_str::<Vec<Task>>(&content) {
                for task in tasks {
                    if task.finish_date == today && task.alert {
                        notifications.push(format!("{} • {} • {}", task.name, task.priority, task.state));
                    }
                }
            }
        }

        self.notification_count = notifications.len();
        self.notifications = notifications;
    }

    fn load_harlequin_databases(&mut self) {
        self.harlequin_databases.clear();
        self.selected_harlequin_db = 0;
        
        // Leer el path de configuración
        let config_path = if let Ok(config_str) = fs::read_to_string("config/settings.json") {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                config["harlequin_config_path"].as_str().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(path) = config_path {
            // Leer el archivo TOML
            if let Ok(toml_content) = fs::read_to_string(&path) {
                if let Ok(toml_value) = toml_content.parse::<toml::Value>() {
                    // Buscar los perfiles en el TOML
                    if let Some(profiles) = toml_value.get("profiles").and_then(|v| v.as_table()) {
                        for (name, _) in profiles {
                            self.harlequin_databases.push(name.clone());
                        }
                    }
                }
            }
        }
        
        if self.harlequin_databases.is_empty() {
            self.harlequin_databases.push("No profiles configured".to_string());
        }
    }

    fn load_helix_directory(&mut self) {
        use std::fs;
        
        self.helix_file_entries.clear();
        
        // Agregar ".." para subir de nivel si no estamos en la raíz
        if let Some(parent) = self.helix_current_dir.parent() {
            self.helix_file_entries.push(HelixFileEntry {
                path: parent.to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
            });
        }
        
        if let Ok(entries) = fs::read_dir(&self.helix_current_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let path = entry.path();
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    self.helix_file_entries.push(HelixFileEntry {
                        path: path.clone(),
                        name,
                        is_dir: metadata.is_dir(),
                    });
                }
            }
        }
        
        // Ordenar solo las entradas después de ".."
        let has_parent = self.helix_file_entries.first().map(|e| e.name == "..").unwrap_or(false);
        let start_idx = if has_parent { 1 } else { 0 };
        
        if start_idx < self.helix_file_entries.len() {
            self.helix_file_entries[start_idx..].sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
        }
        
        self.helix_selected_entry = 0;
        self.helix_scroll_offset = 0;
    }

    fn draw_options_popup(&self, f: &mut Frame) {
        use ratatui::widgets::Clear;
        
        let size = f.size();
        let popup_width = 30;
        let popup_height = 10;
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(" ⚙ Options ", Style::default().fg(Color::Cyan)));

        let content = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("✅ C: Checks", Style::default().fg(Color::Cyan))),
            Line::from(""),
            Line::from(Span::styled("🔔 N: Notify", Style::default().fg(Color::Magenta))),
            Line::from(""),
            Line::from(Span::styled("🔧 T: Tools", Style::default().fg(Color::Green))),
            Line::from(""),
        ]);

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, popup_area);
    }

    fn draw_notifications_popup(&self, f: &mut Frame) {
        use ratatui::widgets::Clear;
        
        let size = f.size();
        let popup_width = 50;
        let popup_height = 15;
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(" 🔔 Notifications ", Style::default().fg(Color::Yellow)));

        let content = if self.notifications.is_empty() {
            Text::from(Line::from(Span::styled("empty notifications", Style::default().fg(Color::Gray).add_modifier(ratatui::style::Modifier::ITALIC))))
        } else {
            let lines: Vec<Line> = self.notifications.iter()
                .map(|notification| Line::from(format!("• {}", notification)))
                .collect();
            Text::from(lines)
        };

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Left)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(paragraph, popup_area);
    }

    fn draw_harlequin_popup(&self, f: &mut Frame) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let size = f.size();
        let popup_width = 60;
        let popup_height = (self.harlequin_databases.len() as u16 + 6).max(10);
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(" 🗄  Harlequin Profiles ", Style::default().fg(Color::Cyan)));

        if self.harlequin_databases.is_empty() || self.harlequin_databases[0] == "No profiles configured" {
            let content = Text::from(Line::from(Span::styled("No configuration found", Style::default().fg(Color::Gray).add_modifier(ratatui::style::Modifier::ITALIC))));
            let paragraph = Paragraph::new(content)
                .alignment(Alignment::Center)
                .block(block);
            f.render_widget(paragraph, popup_area);
        } else {
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);
            
            // Dividir en área de lista y área de recordatorio
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(2)])
                .split(inner);
            
            let items: Vec<ListItem> = self.harlequin_databases.iter()
                .enumerate()
                .map(|(i, db)| {
                    let text = format!("• {}", db);
                    if i == self.selected_harlequin_db {
                        ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                    } else {
                        ListItem::new(text)
                    }
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, chunks[0]);
            
            // Recordatorio de VPN
            let reminder = Paragraph::new("⚠️  Asegúrate de estar conectado a Cisco AnyConnect")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            f.render_widget(reminder, chunks[1]);
        }
    }

    fn draw_helix_popup(&self, f: &mut Frame) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let size = f.size();
        let popup_width = 80;
        let popup_height = 25;
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let current_dir_str = self.helix_current_dir.to_string_lossy();
        let title = format!(" 📁 Helix: {} ", current_dir_str);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(title, Style::default().fg(Color::Cyan)));

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);
        
        // Dividir en área de lista y área de ayuda
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner);
        
        let visible_height = chunks[0].height as usize;
        let items: Vec<ListItem> = self.helix_file_entries.iter()
            .skip(self.helix_scroll_offset)
            .take(visible_height)
            .enumerate()
            .map(|(display_idx, entry)| {
                let actual_idx = display_idx + self.helix_scroll_offset;
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let text = format!("{} {}", icon, entry.name);
                if actual_idx == self.helix_selected_entry {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, chunks[0]);
        
        // Área de ayuda
        let help_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        
        let help_text = Paragraph::new("j/k: Navigate | Enter: Open folder | Tab: Launch Helix here | Esc: Cancel")
            .block(help_block)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(help_text, chunks[1]);
    }

    fn draw_notifications_block(&self, f: &mut Frame, area: Rect) {
        let (title_style, border_style) = if self.notifications_focused {
            (Style::default().fg(Color::Blue), Style::default().fg(Color::Blue))
        } else {
            (Style::default().fg(Color::Magenta), Style::default())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(" 🔔 Notify ", title_style));

        let paragraph = Paragraph::new(self.notification_count.to_string())
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, area);
    }

    fn draw_date_block(&self, f: &mut Frame, area: Rect, context: &ScreenContext) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" 📅 Date ", Style::default().fg(Color::Green)));

        let date_str = context.current_datetime.format("%Y-%m-%d").to_string();
        let time_str = context.current_datetime.format("%H:%M:%S").to_string();

        let paragraph = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(date_str, Style::default().fg(Color::White))),
            Line::from(Span::styled(time_str, Style::default().fg(Color::Yellow))),
        ]))
        .alignment(Alignment::Center)
        .block(block);

        f.render_widget(paragraph, area);
    }

    fn draw_checks_block(&self, f: &mut Frame, area: Rect) {
        let (title_style, border_style) = if self.checks_focused {
            (Style::default().fg(Color::Blue), Style::default().fg(Color::Blue))
        } else {
            (Style::default().fg(Color::Cyan), Style::default())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(" ✅ Checks ", title_style));

        let todo_style = if self.checks_focused && self.selected_item == 0 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let releases_style = if self.checks_focused && self.selected_item == 1 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let versions_style = if self.checks_focused && self.selected_item == 2 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let content = Text::from(vec![
            Line::from(Span::styled("Todo", todo_style)),
            Line::from(Span::styled("Releases", releases_style)),
            Line::from(Span::styled("Versions", versions_style)),
        ]);

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, area);
    }

    fn draw_tools_block(&self, f: &mut Frame, area: Rect) {
        let (title_style, border_style) = if self.tools_focused {
            (Style::default().fg(Color::Blue), Style::default().fg(Color::Blue))
        } else {
            (Style::default().fg(Color::Green), Style::default())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(" 🔧 Tools ", title_style));

        let posting_style = if self.tools_focused && self.selected_tool == 0 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let glances_style = if self.tools_focused && self.selected_tool == 1 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let harlequin_style = if self.tools_focused && self.selected_tool == 2 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let helix_style = if self.tools_focused && self.selected_tool == 3 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        let content = Text::from(vec![
            // Line::from(""),
            Line::from(Span::styled("posting", posting_style)),
            Line::from(Span::styled("glances", glances_style)),
            Line::from(Span::styled("harlequin", harlequin_style)),
            Line::from(Span::styled("helix", helix_style)),
        ]);

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, area);
    }

    fn draw_documentation_block(&self, f: &mut Frame, area: Rect) {
        let (title_style, border_style) = if self.documentation_focused {
            (Style::default().fg(Color::Blue), Style::default().fg(Color::Blue))
        } else {
            (Style::default().fg(Color::Cyan), Style::default())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(" 📚 Docs ", title_style))
            .padding(ratatui::widgets::Padding::horizontal(1));

        let artifacts_style = if self.documentation_focused && self.selected_doc == 0 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };

        let diagrams_style = if self.documentation_focused && self.selected_doc == 1 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };

        let miscellany_style = if self.documentation_focused && self.selected_doc == 2 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };

        let flows_style = if self.documentation_focused && self.selected_doc == 3 {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().fg(Color::White)
        };

        let content = Text::from(vec![
            Line::from(Span::styled(" Artifacts", artifacts_style)),
            Line::from(Span::styled(" Diagrams", diagrams_style)),
            Line::from(Span::styled(" Miscellany", miscellany_style)),
            Line::from(Span::styled(" Flows", flows_style)),
        ]);

        let paragraph = Paragraph::new(content)
            .block(block);

        f.render_widget(paragraph, area);
    }

    fn draw_menu_block(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" ⚙ Menu ", Style::default().fg(Color::Magenta)));

        let paragraph = Paragraph::new("Q: Quit | O: Options | S: Settings")
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, area);
    }
}

impl Screen for HomeScreen {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<ScreenOutcome> {
        if self.show_notifications_popup {
            match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                    self.show_notifications_popup = false;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_helix_popup {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_helix_popup = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if self.helix_selected_entry < self.helix_file_entries.len().saturating_sub(1) {
                        self.helix_selected_entry += 1;
                        let visible_height = 20;
                        if self.helix_selected_entry >= self.helix_scroll_offset + visible_height {
                            self.helix_scroll_offset = self.helix_selected_entry - visible_height + 1;
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.helix_selected_entry > 0 {
                        self.helix_selected_entry -= 1;
                        if self.helix_selected_entry < self.helix_scroll_offset {
                            self.helix_scroll_offset = self.helix_selected_entry;
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if self.helix_selected_entry < self.helix_file_entries.len() {
                        let entry = &self.helix_file_entries[self.helix_selected_entry];
                        if entry.is_dir {
                            self.helix_current_dir = entry.path.clone();
                            self.load_helix_directory();
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Tab => {
                    if self.helix_selected_entry < self.helix_file_entries.len() {
                        let entry = &self.helix_file_entries[self.helix_selected_entry];
                        let target_dir = if entry.is_dir {
                            entry.path.clone()
                        } else {
                            entry.path.parent().unwrap_or(&self.helix_current_dir).to_path_buf()
                        };
                        Ok(ScreenOutcome::LaunchHelix(target_dir))
                    } else {
                        Ok(ScreenOutcome::Continue)
                    }
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_harlequin_popup {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_harlequin_popup = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if self.selected_harlequin_db < self.harlequin_databases.len().saturating_sub(1) {
                        self.selected_harlequin_db += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.selected_harlequin_db > 0 {
                        self.selected_harlequin_db -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if self.selected_harlequin_db < self.harlequin_databases.len() {
                        let profile = self.harlequin_databases[self.selected_harlequin_db].clone();
                        if profile != "No profiles configured" {
                            // Guardar el perfil para ejecutarlo después de salir
                            std::env::set_var("CIFRE_HARLEQUIN_PROFILE", &profile);
                            // Salir de la aplicación para ejecutar harlequin
                            return Ok(ScreenOutcome::Quit);
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_options_popup {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_options_popup = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                    self.show_options_popup = false;
                    self.checks_focused = true;
                    self.notifications_focused = false;
                    self.tools_focused = false;
                    self.documentation_focused = false;
                    self.selected_item = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                    self.show_options_popup = false;
                    self.notifications_focused = true;
                    self.checks_focused = false;
                    self.tools_focused = false;
                    self.documentation_focused = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('t') | crossterm::event::KeyCode::Char('T') => {
                    self.show_options_popup = false;
                    self.tools_focused = true;
                    self.checks_focused = false;
                    self.notifications_focused = false;
                    self.documentation_focused = false;
                    self.selected_tool = 0;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Char('q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('o') | crossterm::event::KeyCode::Char('O') => {
                    self.show_options_popup = true;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                    self.checks_focused = true;
                    self.notifications_focused = false;
                    self.tools_focused = false;
                    self.documentation_focused = false;
                    self.selected_item = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                    self.notifications_focused = true;
                    self.checks_focused = false;
                    self.tools_focused = false;
                    self.documentation_focused = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('t') | crossterm::event::KeyCode::Char('T') => {
                    self.tools_focused = true;
                    self.checks_focused = false;
                    self.notifications_focused = false;
                    self.documentation_focused = false;
                    self.selected_tool = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                    self.documentation_focused = true;
                    self.checks_focused = false;
                    self.notifications_focused = false;
                    self.tools_focused = false;
                    self.selected_doc = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if self.checks_focused && self.selected_item < 2 {
                        self.selected_item += 1;
                    } else if self.tools_focused && self.selected_tool < 3 {
                        self.selected_tool += 1;
                    } else if self.documentation_focused && self.selected_doc < 3 {
                        self.selected_doc += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.checks_focused && self.selected_item > 0 {
                        self.selected_item -= 1;
                    } else if self.tools_focused && self.selected_tool > 0 {
                        self.selected_tool -= 1;
                    } else if self.documentation_focused && self.selected_doc > 0 {
                        self.selected_doc -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if self.checks_focused {
                        match self.selected_item {
                            0 => Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::ViewingTodo)),
                            1 => Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::ViewingReleases)),
                            2 => Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::ViewingVersions)),
                            _ => Ok(ScreenOutcome::Continue),
                        }
                    } else if self.notifications_focused {
                        self.show_notifications_popup = true;
                        Ok(ScreenOutcome::Continue)
                    } else if self.tools_focused {
                        match self.selected_tool {
                            0 | 1 => {
                                let tool_name = match self.selected_tool {
                                    0 => "posting",
                                    1 => "glances",
                                    _ => return Ok(ScreenOutcome::Continue),
                                };
                                
                                // Ejecutar la aplicación
                                if let Err(_) = std::process::Command::new(tool_name).status() {
                                    // Si falla, no hacer nada
                                }
                            }
                            2 => {
                                // Mostrar popup de harlequin
                                self.load_harlequin_databases();
                                self.show_harlequin_popup = true;
                            }
                            3 => {
                                // Mostrar popup de helix con filetree
                                if let Ok(config_str) = std::fs::read_to_string("config/settings.json") {
                                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                                        if let Some(repos_path) = config["repos_path"].as_str() {
                                            self.helix_current_dir = std::path::PathBuf::from(repos_path);
                                            self.load_helix_directory();
                                            self.show_helix_popup = true;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        
                        Ok(ScreenOutcome::Continue)
                    } else {
                        Ok(ScreenOutcome::Continue)
                    }
                }
                crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::ViewingSettings))
                }
                crossterm::event::KeyCode::Esc => {
                    if self.checks_focused || self.notifications_focused || self.tools_focused || self.documentation_focused {
                        self.checks_focused = false;
                        self.notifications_focused = false;
                        self.tools_focused = false;
                        self.documentation_focused = false;
                        Ok(ScreenOutcome::Continue)
                    } else {
                        Ok(ScreenOutcome::Quit)
                    }
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        }
    }

    fn draw(&mut self, f: &mut Frame, context: &ScreenContext) {
        let size = f.size();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let home_area = main_layout[0];
        let menu_area = main_layout[1];

        let home_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" 🏠 Home ", Style::default().fg(Color::Blue)));
        
        f.render_widget(home_block.clone(), home_area);
        let home_inner = home_block.inner(home_area);

        let home_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12),
                Constraint::Length(2), // Margen entre Checks y Tools
                Constraint::Length(14), // Tools
                Constraint::Length(2), // Margen entre Tools y Documentation
                Constraint::Length(14), // Documentation
                Constraint::Min(0),
                Constraint::Length(17),
            ])
            .margin(1)
            .split(home_inner);

        let checks_area = home_layout[0];
        let tools_area = home_layout[2];
        let documentation_area = home_layout[4];
        let date_area = home_layout[6];

        let checks_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5)])
            .split(checks_area);

        let tools_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6)])
            .split(tools_area);

        let documentation_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6)])
            .split(documentation_area);

        let date_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Length(4)])
            .split(date_area);

        self.draw_checks_block(f, checks_vertical[0]);
        self.draw_tools_block(f, tools_vertical[0]);
        self.draw_documentation_block(f, documentation_vertical[0]);
        self.draw_date_block(f, date_vertical[0], context);
        self.draw_notifications_block(f, date_vertical[1]);
        self.draw_menu_block(f, menu_area);

        // Mostrar popup de notificaciones si está activo
        if self.show_notifications_popup {
            self.draw_notifications_popup(f);
        }

        // Mostrar popup de harlequin si está activo
        if self.show_harlequin_popup {
            self.draw_harlequin_popup(f);
        }

        // Mostrar popup de helix si está activo
        if self.show_helix_popup {
            self.draw_helix_popup(f);
        }

        // Mostrar popup de opciones si está activo
        if self.show_options_popup {
            self.draw_options_popup(f);
        }
    }
}
