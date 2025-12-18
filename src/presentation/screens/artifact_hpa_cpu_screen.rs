use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;
use crate::application::hpa_docs_service::HpaDocsService;
use crate::domain::models::ArtifactHpaDoc;

#[derive(Debug, Clone)]
pub enum GenerationState {
    Idle,
    InProgress,
    Completed,
}

pub struct ArtifactHpaCpuScreen {
    artifact_name: String,
    hpa_data: Option<ArtifactHpaDoc>,
    generation_state: GenerationState,
    progress_receiver: Option<mpsc::Receiver<String>>,
    current_progress: String,
    last_update: Instant,
}

impl ArtifactHpaCpuScreen {
    pub fn new(artifact_name: String) -> Self {
        let mut screen = Self { 
            artifact_name: artifact_name.clone(),
            hpa_data: None,
            generation_state: GenerationState::Idle,
            progress_receiver: None,
            current_progress: String::new(),
            last_update: Instant::now(),
        };
        screen.load_hpa_data();
        screen
    }
    
    fn load_hpa_data(&mut self) {
        if let Ok(docs) = HpaDocsService::load_hpa_docs() {
            self.hpa_data = docs.into_iter().find(|doc| doc.name == self.artifact_name);
        }
    }
    
    fn start_generation(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.progress_receiver = Some(rx);
        self.generation_state = GenerationState::InProgress;
        self.current_progress = "Starting HPA generation...".to_string();
        self.last_update = Instant::now();
        
        let artifact_name = self.artifact_name.clone();
        thread::spawn(move || {
            let _ = tx.send("Scanning configurations-region...".to_string());
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            let _ = tx.send(format!("Processing artifact: {}", artifact_name));
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            let _ = tx.send("Extracting HPA data from YAML files...".to_string());
            std::thread::sleep(std::time::Duration::from_millis(1000));
            
            let _ = tx.send("Processing deployment configurations...".to_string());
            std::thread::sleep(std::time::Duration::from_millis(800));
            
            let _ = tx.send("Saving HPA data...".to_string());
            let _ = HpaDocsService::generate_hpa_docs_for_artifact(&artifact_name);
            std::thread::sleep(std::time::Duration::from_millis(300));
            
            let _ = tx.send("COMPLETED".to_string());
        });
    }
    
    fn update_progress(&mut self) {
        if let Some(receiver) = &self.progress_receiver {
            if let Ok(message) = receiver.try_recv() {
                if message == "COMPLETED" {
                    self.generation_state = GenerationState::Completed;
                    self.progress_receiver = None;
                    self.current_progress.clear();
                    self.load_hpa_data();
                } else {
                    self.current_progress = message;
                }
                self.last_update = Instant::now();
            }
        }
    }
}

impl Screen for ArtifactHpaCpuScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        // Always update progress first
        self.update_progress();
        
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
            KeyCode::Char('b') | KeyCode::Char('B') => {
                Ok(ScreenOutcome::ChangeState(AppState::ViewingArtifacts))
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                if matches!(self.generation_state, GenerationState::Idle | GenerationState::Completed) {
                    self.start_generation();
                }
                Ok(ScreenOutcome::Continue)
            }
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        // Update progress on every draw
        self.update_progress();
        
        // Force redraw if generating and enough time has passed
        if matches!(self.generation_state, GenerationState::InProgress) {
            if self.last_update.elapsed() > Duration::from_millis(100) {
                self.last_update = Instant::now();
            }
        }
        
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.size());

        let title_block = Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                format!("{} - hpa and cpu", self.artifact_name),
                Style::default().add_modifier(Modifier::BOLD)
            ),
        ]);
        
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title_block);

        let content = if let Some(hpa_data) = &self.hpa_data {
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("Artifact: {}", hpa_data.name),
                    Style::default().add_modifier(Modifier::BOLD)
                )),
                Line::from(""),
            ];
            
            for country_data in &hpa_data.hpa {
                lines.push(Line::from(Span::styled(
                    format!("Country: {}", country_data.country),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                )));
                
                for stage_data in &country_data.stages {
                    lines.push(Line::from(format!(
                        "  {} - Min: {}, Max: {}, Deploy: {}",
                        stage_data.stage,
                        stage_data.min_replicas,
                        stage_data.max_replicas,
                        stage_data.deploy_replicas
                    )));
                }
                lines.push(Line::from(""));
            }
            
            Text::from(lines)
        } else {
            Text::from("No HPA data available. Press G to generate.")
        };

        let paragraph = Paragraph::new(content).block(main_block);
        f.render_widget(paragraph, main_layout[0]);

        let menu_block_title = Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
            Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        
        let menu_text = match self.generation_state {
            GenerationState::InProgress => "B: Back | Q: Quit | Generating...",
            _ => if self.hpa_data.is_some() {
                "B: Back | Q: Quit | G: Regenerate"
            } else {
                "B: Back | Q: Quit | G: Generate"
            }
        };
        
        let menu_paragraph = Paragraph::new(menu_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .title(menu_block_title)
            );
        
        f.render_widget(menu_paragraph, main_layout[1]);

        // Draw progress popup if generating
        if matches!(self.generation_state, GenerationState::InProgress) && self.progress_receiver.is_some() {
            self.draw_progress_popup(f);
        }
    }
}

impl ArtifactHpaCpuScreen {
    fn draw_progress_popup(&self, f: &mut Frame) {
        let size = f.size();
        let popup_area = Rect {
            x: size.width / 4,
            y: size.height / 3,
            width: size.width / 2,
            height: 8,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" 🔄 Generating HPA Data ", Style::default().fg(Color::Green)));

        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let spinner_index = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() / 100) % spinner_chars.len() as u128;
        let spinner = spinner_chars[spinner_index as usize];

        let content = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{} ", spinner), Style::default().fg(Color::Green)),
                Span::raw(&self.current_progress),
            ]),
            Line::from(""),
            Line::from(Span::styled("Please wait...", Style::default().fg(Color::Gray))),
        ]);

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, popup_area);
    }
}
