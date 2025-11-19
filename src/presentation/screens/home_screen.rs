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
    selected_item: usize,
    selected_tool: usize,
    show_notifications_popup: bool,
    show_options_popup: bool,
    show_harlequin_popup: bool,
    notification_count: usize,
    notifications: Vec<String>, // Lista de notificaciones del día
}

impl HomeScreen {
    pub fn new() -> Result<Self> {
        let mut screen = HomeScreen {
            checks_focused: false,
            notifications_focused: false,
            tools_focused: false,
            selected_item: 0,
            selected_tool: 0,
            show_notifications_popup: false,
            show_options_popup: false,
            show_harlequin_popup: false,
            notification_count: 0,
            notifications: Vec::new(),
        };
        screen.load_notifications();
        Ok(screen)
    }

    pub fn new_with_focus() -> Result<Self> {
        let mut screen = HomeScreen {
            checks_focused: true,
            notifications_focused: false,
            tools_focused: false,
            selected_item: 0,
            selected_tool: 0,
            show_notifications_popup: false,
            show_options_popup: false,
            show_harlequin_popup: false,
            notification_count: 0,
            notifications: Vec::new(),
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
        use ratatui::widgets::Clear;
        
        let size = f.size();
        let popup_width = 40;
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
            .title(Span::styled(" 🗄  Harlequin Connections ", Style::default().fg(Color::Cyan)));

        let paragraph = Paragraph::new("")
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, popup_area);
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

        let content = Text::from(vec![
            Line::from(Span::styled("Todo", todo_style)),
            Line::from(Span::styled("Releases", releases_style)),
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
            Line::from(""),
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
        } else if self.show_harlequin_popup {
            match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                    self.show_harlequin_popup = false;
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
                    self.selected_item = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                    self.show_options_popup = false;
                    self.notifications_focused = true;
                    self.checks_focused = false;
                    self.tools_focused = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('t') | crossterm::event::KeyCode::Char('T') => {
                    self.show_options_popup = false;
                    self.tools_focused = true;
                    self.checks_focused = false;
                    self.notifications_focused = false;
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
                    self.selected_item = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                    self.notifications_focused = true;
                    self.checks_focused = false;
                    self.tools_focused = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('t') | crossterm::event::KeyCode::Char('T') => {
                    self.tools_focused = true;
                    self.checks_focused = false;
                    self.notifications_focused = false;
                    self.selected_tool = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if self.checks_focused && self.selected_item < 1 {
                        self.selected_item += 1;
                    } else if self.tools_focused && self.selected_tool < 3 {
                        self.selected_tool += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.checks_focused && self.selected_item > 0 {
                        self.selected_item -= 1;
                    } else if self.tools_focused && self.selected_tool > 0 {
                        self.selected_tool -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if self.checks_focused {
                        match self.selected_item {
                            0 => Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::ViewingTodo)),
                            1 => Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::ViewingReleases)),
                            _ => Ok(ScreenOutcome::Continue),
                        }
                    } else if self.notifications_focused {
                        self.show_notifications_popup = true;
                        Ok(ScreenOutcome::Continue)
                    } else if self.tools_focused {
                        match self.selected_tool {
                            0 | 1 | 3 => {
                                let tool_name = match self.selected_tool {
                                    0 => "posting",
                                    1 => "glances",
                                    3 => "helix",
                                    _ => return Ok(ScreenOutcome::Continue),
                                };
                                
                                // Ejecutar la aplicación
                                if let Err(_) = std::process::Command::new(tool_name).status() {
                                    // Si falla, no hacer nada
                                }
                            }
                            2 => {
                                // Mostrar popup de harlequin
                                self.show_harlequin_popup = true;
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
                    if self.checks_focused || self.notifications_focused || self.tools_focused {
                        self.checks_focused = false;
                        self.notifications_focused = false;
                        self.tools_focused = false;
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
                Constraint::Length(14), // Aumentado para mejor espaciado
                Constraint::Min(0),
                Constraint::Length(17),
            ])
            .margin(1)
            .split(home_inner);

        let checks_area = home_layout[0];
        let tools_area = home_layout[2];
        let date_area = home_layout[4];

        let checks_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4)])
            .split(checks_area);

        let tools_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8)])
            .split(tools_area);

        let date_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Length(4)])
            .split(date_area);

        self.draw_checks_block(f, checks_vertical[0]);
        self.draw_tools_block(f, tools_vertical[0]);
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

        // Mostrar popup de opciones si está activo
        if self.show_options_popup {
            self.draw_options_popup(f);
        }
    }
}
