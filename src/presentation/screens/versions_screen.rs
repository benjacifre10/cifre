// presentation/versions_screen.rs
use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, BorderType, Clear}, // <-- Añadir Clear
    Frame,
};
use ratatui::style::Stylize;
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::application::versions_artifact_service::VersionsArtifactService;
use crate::domain::models::VersionsArtifact;

#[allow(dead_code)]
enum GenerationState {
    Idle,
    InProgress {
        current_step: usize,
        total_steps: usize,
        message: String,
        error_message: Option<String>,
        spinner_frame: usize,
        start_time: Instant,
    },
    Completed,
    Failed {
        error: String,
    },
}

enum GenerationMessage {
    Progress(usize, Option<String>, Option<String>),
    TotalSteps(usize),
    Completed(Result<Vec<VersionsArtifact>, String>),
}


pub struct VersionsScreen {
    versions_artifact_data: Vec<VersionsArtifact>,
    error_message: Option<String>,
    base_path_display: String,
    versions_service: VersionsArtifactService,

    generation_state: GenerationState,
    progress_receiver: Option<mpsc::Receiver<GenerationMessage>>,
    last_ui_update: Instant,

    selected_repo: usize, // <-- Nuevo campo para el repositorio seleccionado
}

impl VersionsScreen {
    pub fn new() -> Self {
        let versions_service = VersionsArtifactService::new("/Users/u631568/Documents/Development/Teco/401/");
        let base_path_display = versions_service.get_base_path_display();

        let mut screen_instance = Self {
            versions_artifact_data: Vec::new(),
            error_message: None,
            base_path_display,
            versions_service,
            generation_state: GenerationState::Idle,
            progress_receiver: None,
            last_ui_update: Instant::now(),
            selected_repo: 0, // <-- Inicializar el seleccionado
        };

        match screen_instance.versions_service.read_versions_from_json() {
            Ok(mut data) => { // 'mut data' porque vamos a ordenar
                data.sort_by(|a, b| a.name.cmp(&b.name)); // <--- APLICAR ORDENACIÓN AQUÍ
                screen_instance.versions_artifact_data = data;
                // Asegurarse de que selected_repo esté dentro de los límites si hay datos
                if !screen_instance.versions_artifact_data.is_empty() {
                    screen_instance.selected_repo = 0;
                } else {
                    screen_instance.selected_repo = 0;
                }
            },
            Err(e) => {
                screen_instance.error_message = Some(e);
            }
        }

        screen_instance
    }

    fn start_generation(&mut self) {
        self.error_message = None;
        self.versions_artifact_data.clear();
        self.generation_state = GenerationState::InProgress {
            current_step: 0,
            total_steps: 1, // Inicializar con 1 para evitar división por cero
            message: "Iniciando generación...".to_string(),
            error_message: None,
            spinner_frame: 0,
            start_time: Instant::now(),
        };
        self.selected_repo = 0; // Resetear la selección al iniciar la generación

        let (sender, receiver) = mpsc::channel::<GenerationMessage>();
        self.progress_receiver = Some(receiver);

        let base_path_for_thread = self.versions_service.base_path.clone();
        let output_dir_for_thread = self.versions_service.output_dir.clone();
        let output_file_name_for_thread = self.versions_service.output_file_name.clone();

        thread::spawn(move || {
            let service_in_thread = VersionsArtifactService::new_with_output(
                base_path_for_thread.to_str().unwrap_or_default(),
                output_dir_for_thread.to_str().unwrap_or_default(),
                &output_file_name_for_thread
            );

            let service_in_thread_for_callback = service_in_thread.clone();
            let sender_for_callback = sender.clone();

            let callback = move |current_step: usize, msg: Option<String>, err_msg: Option<String>| {
                let num_repos = fs::read_dir(&service_in_thread_for_callback.base_path).map(|dir| dir.count()).unwrap_or(0);
                let num_branches = 3; // CAMBIO: Ahora son 3 ramas (dev, release, prod)
                let total_steps = num_repos * num_branches;

                if total_steps > 0 {
                     sender_for_callback.send(GenerationMessage::TotalSteps(total_steps)).unwrap();
                }

                sender_for_callback.send(GenerationMessage::Progress(current_step, msg, err_msg)).unwrap();
            };

            let result = service_in_thread.get_versions_artifact_data_with_progress(callback);
            sender.send(GenerationMessage::Completed(result)).unwrap();
        });
    }

    fn check_generation_progress(&mut self) {
        if let Some(receiver) = self.progress_receiver.take() {
            let mut keep_receiver = true;

            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    GenerationMessage::TotalSteps(total) => {
                        if let GenerationState::InProgress { total_steps, .. } = &mut self.generation_state {
                            *total_steps = total;
                        }
                    },
                    GenerationMessage::Progress(step, msg, err) => {
                        if let GenerationState::InProgress { current_step, message, error_message, .. } = &mut self.generation_state {
                            *current_step = step;
                            if let Some(m) = msg {
                                *message = m;
                            }
                            *error_message = err;
                        }
                    },
                    GenerationMessage::Completed(result) => {
                        match result {
                            Ok(mut data) => { // 'mut data' porque vamos a ordenar
                                data.sort_by(|a, b| a.name.cmp(&b.name)); // <--- APLICAR ORDENACIÓN AQUÍ
                                self.versions_artifact_data = data;
                                self.generation_state = GenerationState::Completed;
                                // Asegurarse de que selected_repo esté dentro de los límites después de la generación
                                if !self.versions_artifact_data.is_empty() {
                                    self.selected_repo = 0;
                                } else {
                                    self.selected_repo = 0;
                                }
                            },
                            Err(e) => {
                                self.generation_state = GenerationState::Failed { error: e.clone() };
                                self.error_message = Some(e);
                            },
                        }
                        keep_receiver = false;
                    },
                }
            }

            if keep_receiver {
                self.progress_receiver = Some(receiver);
            }
        }
    }
}

impl Screen for VersionsScreen {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<ScreenOutcome> {
        if matches!(self.generation_state, GenerationState::InProgress { .. }) {
            return Ok(ScreenOutcome::Continue);
        }

        match key.code {
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('b') => {
                Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
            }
            crossterm::event::KeyCode::Char('q') => {
                Ok(ScreenOutcome::Quit)
            }
            crossterm::event::KeyCode::Char('g') | crossterm::event::KeyCode::Char('G') => {
                self.start_generation();
                Ok(ScreenOutcome::Continue)
            }
            crossterm::event::KeyCode::Char('j') => { // Mover selección hacia abajo
                if !self.versions_artifact_data.is_empty() {
                    self.selected_repo = (self.selected_repo + 1) % self.versions_artifact_data.len();
                }
                Ok(ScreenOutcome::Continue)
            }
            crossterm::event::KeyCode::Char('k') => { // Mover selección hacia arriba
                if !self.versions_artifact_data.is_empty() {
                    if self.selected_repo == 0 {
                        self.selected_repo = self.versions_artifact_data.len() - 1;
                    } else {
                        self.selected_repo -= 1;
                    }
                }
                Ok(ScreenOutcome::Continue)
            }
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        self.check_generation_progress();

        if let GenerationState::InProgress { spinner_frame, .. } = &mut self.generation_state {
            if self.last_ui_update.elapsed() >= Duration::from_millis(80) {
                *spinner_frame = (*spinner_frame + 1) % 10;
                self.last_ui_update = Instant::now();
            }
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(size);

        let content_area = main_layout[0];
        let help_area = main_layout[1];

        // Dibuja el bloque principal primero, esto envuelve todo el contenido.
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(" 🚀 Repositories Overview ")
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        f.render_widget(&main_block, size);

        let inner_content_area = main_block.inner(content_area);

        // *** Solución definitiva para la limpieza: Renderizar widget Clear ***
        // Esto borra completamente el área de contenido interior antes de dibujar cualquier otra cosa.
        f.render_widget(Clear, inner_content_area);


        // Renderizar el contenido principal SOLO SI NO ESTAMOS EN PROGRESO
        if !matches!(self.generation_state, GenerationState::InProgress { .. }) {
            if let Some(error) = &self.error_message {
                let error_paragraph = Paragraph::new(
                    Span::styled(error, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                )
                .alignment(Alignment::Center)
                .block(Block::default().padding(ratatui::widgets::Padding::uniform(1)));
                f.render_widget(error_paragraph, inner_content_area);
            } else if self.versions_artifact_data.is_empty() {
                let no_repos_paragraph = Paragraph::new(
                    Span::styled(format!("No se encontraron repositorios en la carpeta '{}'. Presione 'G' para generarlos.", self.base_path_display), Style::default().fg(Color::Yellow))
                )
                .alignment(Alignment::Center)
                .block(Block::default().padding(ratatui::widgets::Padding::uniform(1)));
                f.render_widget(no_repos_paragraph, inner_content_area);
            } else {
                let header_style = Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray);
                let header_cells = ["Repository", "dev", "release_status", "prod"] // CAMBIO: Encabezados actualizados
                    .iter()
                    .map(|h| Cell::from(*h).style(header_style));

                let header = Row::new(header_cells)
                    .height(1)
                    .bottom_margin(1)
                    .style(Style::default().bg(Color::DarkGray));

                // Definir los colores pastel para las filas alternas
                let pastel_color1 = Color::Rgb(50, 50, 60); // Un gris oscuro azulado
                let pastel_color2 = Color::Rgb(40, 40, 50); // Un gris oscuro un poco más oscuro

                let rows: Vec<Row> = self.versions_artifact_data
                    .iter()
                    .enumerate() // <-- Usar enumerate para obtener el índice
                    .map(|(i, repo)| { // <-- i es el índice de la fila
                        let is_selected = i == self.selected_repo; // <-- Comprobar si está seleccionado

                        let base_style = if is_selected {
                            Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD) // <-- Estilo de selección
                        } else if i % 2 == 0 { // Filas pares
                            Style::default().fg(Color::Gray).bg(pastel_color1)
                        } else { // Filas impares
                            Style::default().fg(Color::Gray).bg(pastel_color2)
                        };

                        let cells = vec![
                            Cell::from(repo.name.clone()),
                            Cell::from(repo.dev_version.clone()),
                            Cell::from(repo.release_version.clone()), // CAMBIO: Mostrar release_version (que ahora es el estado)
                            Cell::from(repo.prod_version.clone()),
                        ];
                        Row::new(cells)
                            .height(1)
                            .style(base_style) // Aplicar el estilo base
                    })
                    .collect(); // <-- Recolectar en un Vec<Row>


                let table = Table::new(rows, [
                    Constraint::Percentage(25), // Repository
                    Constraint::Percentage(25), // dev
                    Constraint::Percentage(25), // release_status
                    Constraint::Percentage(25), // prod
                ])
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(" Versions ", Style::default().fg(Color::LightBlue)))
                        .border_style(Style::default().fg(Color::DarkGray))
                )
                // Eliminamos highlight_style y highlight_symbol porque ahora manejamos el resaltado directamente en la fila
                // .highlight_style(Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD))
                // .highlight_symbol("▶ ")
                .column_spacing(2)
                .bg(Color::Black); // Color de fondo por defecto para la tabla

                f.render_widget(table, inner_content_area);
            }
        }


        // Este bloque solo se ejecuta y renderiza el pop-up de progreso
        // si el estado es GenerationState::InProgress.
        if let GenerationState::InProgress { current_step, total_steps, message, error_message, spinner_frame, start_time } = &self.generation_state {
            let popup_area = Self::centered_rect(60, 30, size);

            // Importante para borrar cualquier remanente del área del pop-up
            f.render_widget(Clear, popup_area);


            let spinner_chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner = spinner_chars[*spinner_frame % spinner_chars.len()];

            let percentage = if *total_steps > 0 {
                (*current_step as f64 / *total_steps as f64 * 100.0) as usize
            } else {
                0
            };

            let elapsed_time = start_time.elapsed();
            let time_str = format!("{:02}:{:02}", elapsed_time.as_secs() / 60, elapsed_time.as_secs() % 60);

            let mut lines = vec![
                Line::from(vec![
                    Span::raw(format!(" {} ", spinner)),
                    Span::styled(format!("Generando versiones... ({}/{}) ", current_step, total_steps), Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{}%", percentage), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(Span::raw("")),
                Line::from(Span::styled(format!("Tiempo transcurrido: {}", time_str), Style::default().fg(Color::LightGreen))),
                Line::from(Span::styled(message.clone(), Style::default().fg(Color::White))),
            ];

            if let Some(err_msg) = error_message {
                lines.push(Line::from(Span::styled(format!("Error: {}", err_msg), Style::default().fg(Color::Red))));
            }

            let popup_block = Block::default()
                .title(Span::styled(" Procesando ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Blue))
                .style(Style::default().bg(Color::Black));

            let popup_paragraph = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .block(popup_block)
                .wrap(ratatui::widgets::Wrap { trim: true });

            f.render_widget(popup_paragraph, popup_area);
        }

        let help_block = Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Plain)
            .title(Span::styled(" ⚙ Menu ", Style::default().fg(Color::Magenta)));

        let help_text = Paragraph::new(Span::styled(" Q: Quit | B: Back | G: Generate | J/K: Navigate ", Style::default().fg(Color::White)))
            .alignment(Alignment::Center)
            .block(help_block);

        f.render_widget(help_text, help_area);
    }
}

impl VersionsScreen {
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

