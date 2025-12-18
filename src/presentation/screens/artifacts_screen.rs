use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, BorderType, Clear},
    Frame,
};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;
use crate::domain::models::{ArtifactDoc, VersionsArtifact};
use crate::application::artifacts_docs_service::ArtifactsDocsService;
use crate::infrastructure::persistence::{CountryJsonRepository, StageJsonRepository};

enum GenerationState {
    Idle,
    InProgress {
        current_step: usize,
        total_steps: usize,
        message: String,
        spinner_frame: usize,
        start_time: Instant,
    },
    Completed,
}

pub struct ArtifactsScreen {
    submenu_focused: bool,
    artifacts: Vec<ArtifactDoc>,
    generation_state: GenerationState,
    progress_receiver: Option<mpsc::Receiver<(usize, usize, String)>>,
    selected_index: usize,
    scroll_offset: usize,
    show_options_popup: bool,
    selected_option: usize,
}

impl ArtifactsScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            submenu_focused: true,
            artifacts: Vec::new(),
            generation_state: GenerationState::Idle,
            progress_receiver: None,
            selected_index: 0,
            scroll_offset: 0,
            show_options_popup: false,
            selected_option: 0,
        };
        screen.load_or_generate();
        screen
    }

    fn load_or_generate(&mut self) {
        let service = ArtifactsDocsService::new("/Users/u631568/Documents/Development/Teco/Others/configurations-region");
        
        match service.load_from_json() {
            Ok(artifacts) => {
                self.artifacts = artifacts;
            }
            Err(_) => {
                self.start_generation();
            }
        }
    }

    fn start_generation(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.progress_receiver = Some(rx);
        self.generation_state = GenerationState::InProgress {
            current_step: 0,
            total_steps: 1,
            message: "Iniciando...".to_string(),
            spinner_frame: 0,
            start_time: Instant::now(),
        };

        thread::spawn(move || {
            let versions_result = std::fs::read_to_string("data/artifact_version.json")
                .and_then(|content| {
                    serde_json::from_str::<Vec<VersionsArtifact>>(&content)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                });

            let versions = match versions_result {
                Ok(v) => v,
                Err(_) => return,
            };

            let country_repo = CountryJsonRepository::new();
            let stage_repo = StageJsonRepository::new();

            let countries = match country_repo.get_all_countries() {
                Ok(c) => c,
                Err(_) => return,
            };

            let stages = match stage_repo.get_all_stages() {
                Ok(s) => s,
                Err(_) => return,
            };

            let service = ArtifactsDocsService::new("/Users/u631568/Documents/Development/Teco/Others/configurations-region");
            
            let _ = service.generate_artifacts_docs_with_progress(&versions, &countries, &stages, move |current, total, msg| {
                let _ = tx.send((current, total, msg));
            });
        });
    }

    fn update_progress(&mut self) {
        let mut should_complete = false;
        
        if let Some(ref rx) = self.progress_receiver {
            while let Ok((current, total, message)) = rx.try_recv() {
                if let GenerationState::InProgress { spinner_frame, start_time, .. } = self.generation_state {
                    self.generation_state = GenerationState::InProgress {
                        current_step: current,
                        total_steps: total,
                        message,
                        spinner_frame,
                        start_time,
                    };

                    if current >= total {
                        should_complete = true;
                    }
                }
            }
        }

        if should_complete {
            self.generation_state = GenerationState::Completed;
            self.progress_receiver = None;
            self.load_artifacts();
        }

        if let GenerationState::InProgress { spinner_frame, .. } = self.generation_state {
            if let GenerationState::InProgress { current_step, total_steps, message, start_time, .. } = self.generation_state.clone() {
                self.generation_state = GenerationState::InProgress {
                    current_step,
                    total_steps,
                    message,
                    spinner_frame: spinner_frame + 1,
                    start_time,
                };
            }
        }
    }

    fn load_artifacts(&mut self) {
        let service = ArtifactsDocsService::new("/Users/u631568/Documents/Development/Teco/Others/configurations-region");
        if let Ok(artifacts) = service.load_from_json() {
            self.artifacts = artifacts;
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
}

impl Clone for GenerationState {
    fn clone(&self) -> Self {
        match self {
            GenerationState::Idle => GenerationState::Idle,
            GenerationState::InProgress { current_step, total_steps, message, spinner_frame, start_time } => {
                GenerationState::InProgress {
                    current_step: *current_step,
                    total_steps: *total_steps,
                    message: message.clone(),
                    spinner_frame: *spinner_frame,
                    start_time: *start_time,
                }
            }
            GenerationState::Completed => GenerationState::Completed,
        }
    }
}

impl Screen for ArtifactsScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        self.update_progress();

        // Manejo del popup de opciones
        if self.show_options_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_options_popup = false;
                    self.selected_option = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    if self.selected_option < 3 {
                        self.selected_option += 1;
                    } else {
                        self.selected_option = 0;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    if self.selected_option > 0 {
                        self.selected_option -= 1;
                    } else {
                        self.selected_option = 3;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if let Some(artifact) = self.artifacts.get(self.selected_index) {
                        let artifact_name = artifact.name.clone();
                        let option = self.selected_option;
                        
                        self.show_options_popup = false;
                        self.selected_option = 0;
                        
                        let new_state = match option {
                            0 => AppState::ViewingArtifactDescription(artifact_name),
                            1 => AppState::ViewingArtifactHpaCpu(artifact_name),
                            2 => AppState::ViewingArtifactDependencies(artifact_name),
                            3 => AppState::ViewingArtifactEndpoints(artifact_name),
                            _ => return Ok(ScreenOutcome::Continue),
                        };
                        
                        return Ok(ScreenOutcome::ChangeState(new_state));
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    Ok(ScreenOutcome::ChangeState(AppState::Home))
                }
                KeyCode::Char('j') => {
                    if !self.artifacts.is_empty() {
                        if self.selected_index < self.artifacts.len() - 1 {
                            self.selected_index += 1;
                        } else {
                            // Navegación circular: ir al primero
                            self.selected_index = 0;
                            self.scroll_offset = 0;
                        }
                        
                        // Ajustar scroll si es necesario
                        let visible_height = 10;
                        if self.selected_index >= self.scroll_offset + visible_height {
                            self.scroll_offset = self.selected_index - visible_height + 1;
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    if !self.artifacts.is_empty() {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        } else {
                            // Navegación circular: ir al último
                            self.selected_index = self.artifacts.len() - 1;
                            let visible_height = 10;
                            if self.artifacts.len() > visible_height {
                                self.scroll_offset = self.artifacts.len() - visible_height;
                            }
                        }
                        
                        // Ajustar scroll si es necesario
                        if self.selected_index < self.scroll_offset {
                            self.scroll_offset = self.selected_index;
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.artifacts.is_empty() {
                        self.show_options_popup = true;
                        self.selected_option = 0;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    if matches!(self.generation_state, GenerationState::Idle | GenerationState::Completed) {
                        self.start_generation();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        self.update_progress();

        let size = f.size();
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(size);

        let title_block = Line::from(vec![
            Span::styled("📦 ", Style::default().fg(Color::LightBlue)),
            Span::styled("Artifacts", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title_block);

        let inner_area = main_block.inner(main_layout[0]);
        f.render_widget(main_block, main_layout[0]);

        if self.artifacts.is_empty() {
            let empty_text = Paragraph::new("No hay artifacts. Presiona L para cargar.")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(empty_text, inner_area);
        } else {
            // Calcular cuántos artifacts caben en el área visible
            let available_height = inner_area.height as usize / 3; // Cada artifact ocupa 3 líneas
            let visible_artifacts = self.artifacts
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .take(available_height);

            for (i, artifact) in visible_artifacts {
                let y_offset = ((i - self.scroll_offset) * 3) as u16;
                if y_offset + 3 > inner_area.height {
                    break;
                }

                let artifact_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + y_offset,
                    width: inner_area.width,
                    height: 3,
                };

                let is_selected = i == self.selected_index;
                let border_color = if is_selected {
                    Color::Yellow
                } else {
                    Color::Gray
                };

                let artifact_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color));

                let inner = artifact_block.inner(artifact_area);
                f.render_widget(artifact_block, artifact_area);

                // Contenido: nombre a la izquierda, namespace a la derecha
                let content = Line::from(vec![
                    Span::styled(
                        format!("{}", artifact.name),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:>width$}", artifact.namespace, width = (inner.width as usize).saturating_sub(artifact.name.len() + 1)),
                        Style::default().fg(Color::Cyan)
                    ),
                ]);

                let paragraph = Paragraph::new(content);
                f.render_widget(paragraph, inner);
            }
        }

        // Popup de progreso
        if let GenerationState::InProgress { current_step, total_steps, message, spinner_frame, .. } = &self.generation_state {
            let popup_area = Self::centered_rect(60, 30, size);
            f.render_widget(Clear, popup_area);

            let spinner_chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner = spinner_chars[*spinner_frame % spinner_chars.len()];

            let percentage = if *total_steps > 0 {
                (*current_step as f64 / *total_steps as f64 * 100.0) as u16
            } else {
                0
            };

            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(spinner, Style::default().fg(Color::Cyan))),
                Line::from(""),
                Line::from(Span::styled(format!("{}%", percentage), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled(message.clone(), Style::default().fg(Color::White))),
                Line::from(""),
                Line::from(Span::styled(format!("{} / {}", current_step, total_steps), Style::default().fg(Color::Gray))),
            ];

            let popup_block = Block::default()
                .title(Span::styled(" Procesando ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Blue))
                .style(Style::default().bg(Color::Black));

            let popup_paragraph = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .block(popup_block);

            f.render_widget(popup_paragraph, popup_area);
        }

        // Popup de opciones
        if self.show_options_popup {
            let popup_area = Self::centered_rect(20, 10, size);
            f.render_widget(Clear, popup_area);

            let options = vec!["description", "hpa and cpu", "dependencies", "endpoints"];
            let items: Vec<Line> = options
                .iter()
                .enumerate()
                .map(|(i, option)| {
                    let style = if i == self.selected_option {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(Span::styled(format!("  {}", option), style))
                })
                .collect();

            let popup_block = Block::default()
                .title(Line::from(vec![
                    Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
                    Span::styled("Options", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black));

            let popup_paragraph = Paragraph::new(items)
                .block(popup_block);

            f.render_widget(popup_paragraph, popup_area);
        }

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
