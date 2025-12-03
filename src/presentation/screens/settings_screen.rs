use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Clear},
    Frame,
};
use std::fs;
use std::path::{Path, PathBuf};

use super::screen::{Screen, ScreenContext, ScreenOutcome};

#[derive(Debug, Clone, PartialEq)]
enum FocusState {
    HarlequinPath,
    ReposPath,
    DocsPath,
    None,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
}

pub struct SettingsScreen {
    harlequin_config_path: String,
    repos_path: String,
    docs_path: String,
    show_notification: bool,
    notification_message: String,
    notification_is_error: bool,
    focus_state: FocusState,
    show_filetree: bool,
    current_dir: PathBuf,
    file_entries: Vec<FileEntry>,
    selected_entry: usize,
    scroll_offset: usize,
    search_filter: String,
}

impl SettingsScreen {
    pub fn new() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        let mut screen = Self {
            harlequin_config_path: String::new(),
            repos_path: String::new(),
            docs_path: String::new(),
            show_notification: false,
            notification_message: String::new(),
            notification_is_error: false,
            focus_state: FocusState::None,
            show_filetree: false,
            current_dir: PathBuf::from(&home_dir),
            file_entries: Vec::new(),
            selected_entry: 0,
            scroll_offset: 0,
            search_filter: String::new(),
        };
        
        // Cargar configuración existente si existe
        if let Ok(config_str) = fs::read_to_string("config/settings.json") {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                if let Some(path) = config["harlequin_config_path"].as_str() {
                    screen.harlequin_config_path = path.to_string();
                }
                if let Some(path) = config["repos_path"].as_str() {
                    screen.repos_path = path.to_string();
                }
                if let Some(path) = config["docs_path"].as_str() {
                    screen.docs_path = path.to_string();
                }
            }
        }
        
        screen
    }

    fn load_directory(&mut self) {
        self.file_entries.clear();
        self.selected_entry = 0;
        self.scroll_offset = 0;
        self.search_filter.clear();
        
        // Agregar entrada para subir al directorio padre
        if let Some(parent) = self.current_dir.parent() {
            self.file_entries.push(FileEntry {
                path: parent.to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
            });
        }
        
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    let file_entry = FileEntry {
                        path: path.clone(),
                        name: name.clone(),
                        is_dir: metadata.is_dir(),
                    };
                    
                    if metadata.is_dir() {
                        dirs.push(file_entry);
                    } else {
                        files.push(file_entry);
                    }
                }
            }
            
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            
            self.file_entries.extend(dirs);
            self.file_entries.extend(files);
        }
    }

    fn get_filtered_entries(&self) -> Vec<(usize, &FileEntry)> {
        if self.search_filter.is_empty() {
            self.file_entries.iter().enumerate().collect()
        } else {
            self.file_entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.name.to_lowercase().contains(&self.search_filter.to_lowercase())
                })
                .collect()
        }
    }

    fn save_config(&mut self) {
        // Verificar si el archivo existe
        if !Path::new(&self.harlequin_config_path).exists() {
            self.notification_message = "Error: File does not exist".to_string();
            self.notification_is_error = true;
            self.show_notification = true;
            return;
        }

        // Crear directorio config si no existe
        if let Err(_) = fs::create_dir_all("config") {
            self.notification_message = "Error: Could not create config directory".to_string();
            self.notification_is_error = true;
            self.show_notification = true;
            return;
        }

        // Crear el JSON de configuración
        let config = serde_json::json!({
            "harlequin_config_path": self.harlequin_config_path,
            "repos_path": self.repos_path,
            "docs_path": self.docs_path
        });

        // Guardar el archivo
        match fs::write("config/settings.json", serde_json::to_string_pretty(&config).unwrap()) {
            Ok(_) => {
                self.notification_message = "Configuration saved successfully".to_string();
                self.notification_is_error = false;
                self.show_notification = true;
            }
            Err(_) => {
                self.notification_message = "Error: Could not save configuration".to_string();
                self.notification_is_error = true;
                self.show_notification = true;
            }
        }
    }
}

impl Screen for SettingsScreen {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<ScreenOutcome> {
        if self.show_notification {
            match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                    self.show_notification = false;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_filetree {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_filetree = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    let filtered = self.get_filtered_entries();
                    if self.selected_entry < filtered.len().saturating_sub(1) {
                        self.selected_entry += 1;
                        // Ajustar scroll
                        let visible_height = 20;
                        if self.selected_entry >= self.scroll_offset + visible_height {
                            self.scroll_offset = self.selected_entry - visible_height + 1;
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.selected_entry > 0 {
                        self.selected_entry -= 1;
                        // Ajustar scroll
                        if self.selected_entry < self.scroll_offset {
                            self.scroll_offset = self.selected_entry;
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    let filtered = self.get_filtered_entries();
                    if self.selected_entry < filtered.len() {
                        let (original_idx, _) = filtered[self.selected_entry];
                        let entry = &self.file_entries[original_idx];
                        if entry.is_dir {
                            // Siempre navegar hacia adentro de carpetas
                            self.current_dir = entry.path.clone();
                            self.load_directory();
                        } else {
                            // Solo archivos para HarlequinPath
                            if matches!(self.focus_state, FocusState::HarlequinPath) {
                                self.harlequin_config_path = entry.path.to_string_lossy().to_string();
                                self.show_filetree = false;
                            }
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                    // Guardar carpeta actual para ReposPath o DocsPath
                    if matches!(self.focus_state, FocusState::ReposPath) {
                        self.repos_path = self.current_dir.to_string_lossy().to_string();
                        self.show_filetree = false;
                    } else if matches!(self.focus_state, FocusState::DocsPath) {
                        self.docs_path = self.current_dir.to_string_lossy().to_string();
                        self.show_filetree = false;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char(c) => {
                    self.search_filter.push(c);
                    self.selected_entry = 0;
                    self.scroll_offset = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Backspace => {
                    self.search_filter.pop();
                    self.selected_entry = 0;
                    self.scroll_offset = 0;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('b') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                crossterm::event::KeyCode::Char('q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Tab => {
                    self.focus_state = match self.focus_state {
                        FocusState::None => FocusState::HarlequinPath,
                        FocusState::HarlequinPath => FocusState::ReposPath,
                        FocusState::ReposPath => FocusState::DocsPath,
                        FocusState::DocsPath => FocusState::None,
                    };
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if self.focus_state == FocusState::HarlequinPath || self.focus_state == FocusState::ReposPath || self.focus_state == FocusState::DocsPath {
                        self.load_directory();
                        self.show_filetree = true;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                    self.save_config();
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char(c) => {
                    match self.focus_state {
                        FocusState::HarlequinPath => self.harlequin_config_path.push(c),
                        FocusState::ReposPath => self.repos_path.push(c),
                        FocusState::DocsPath => self.docs_path.push(c),
                        FocusState::None => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Backspace => {
                    match self.focus_state {
                        FocusState::HarlequinPath => { self.harlequin_config_path.pop(); },
                        FocusState::ReposPath => { self.repos_path.pop(); },
                        FocusState::DocsPath => { self.docs_path.pop(); },
                        FocusState::None => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(size);

        let settings_area = main_layout[0];
        let help_area = main_layout[1];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" ⚙️ Settings ", Style::default().fg(Color::Blue)));

        // Crear el contenido con los campos de configuración
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .margin(2)
            .split(settings_area);

        let harlequin_area = content_layout[0];
        let repos_area = content_layout[1];
        let docs_area = content_layout[2];

        // Renderizar el bloque principal
        f.render_widget(block, settings_area);

        // Campo 1: Configuración de Harlequin
        let harlequin_border_color = if self.focus_state == FocusState::HarlequinPath {
            Color::Yellow
        } else {
            Color::Green
        };
        
        let harlequin_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(harlequin_border_color))
            .title(Span::styled("Archivo configuracion harlequin", Style::default().fg(harlequin_border_color)));

        let harlequin_text = if self.harlequin_config_path.is_empty() {
            "<campo a llenar>"
        } else {
            &self.harlequin_config_path
        };

        let harlequin_style = if self.harlequin_config_path.is_empty() {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        };

        let harlequin_paragraph = Paragraph::new(Span::styled(harlequin_text, harlequin_style))
            .block(harlequin_block);

        f.render_widget(harlequin_paragraph, harlequin_area);

        // Campo 2: Path inicial de repositorios
        let repos_border_color = if self.focus_state == FocusState::ReposPath {
            Color::Yellow
        } else {
            Color::Green
        };
        
        let repos_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(repos_border_color))
            .title(Span::styled("Path inicial de repositorios", Style::default().fg(repos_border_color)));

        let repos_text = if self.repos_path.is_empty() {
            "<campo a llenar>"
        } else {
            &self.repos_path
        };

        let repos_style = if self.repos_path.is_empty() {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        };

        let repos_paragraph = Paragraph::new(Span::styled(repos_text, repos_style))
            .block(repos_block);

        f.render_widget(repos_paragraph, repos_area);

        // Campo 3: Carpeta de documentación
        let docs_border_color = if self.focus_state == FocusState::DocsPath {
            Color::Yellow
        } else {
            Color::Green
        };
        
        let docs_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(docs_border_color))
            .title(Span::styled("Carpeta de documentación", Style::default().fg(docs_border_color)));

        let docs_text = if self.docs_path.is_empty() {
            "<campo a llenar>"
        } else {
            &self.docs_path
        };

        let docs_style = if self.docs_path.is_empty() {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        };

        let docs_paragraph = Paragraph::new(Span::styled(docs_text, docs_style))
            .block(docs_block);

        f.render_widget(docs_paragraph, docs_area);

        // Menú de ayuda
        let help_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" ⚙ Menu ", Style::default().fg(Color::Magenta)));

        let help_text = Paragraph::new("Q: Quit | B: Back | Tab: Focus | Enter: Browse | S: Save")
            .alignment(Alignment::Center)
            .block(help_block);

        f.render_widget(help_text, help_area);

        // Mostrar filetree si está activo
        if self.show_filetree {
            self.draw_filetree(f);
        }

        // Mostrar notificación si está activa
        if self.show_notification {
            self.draw_notification(f);
        }
    }
}

impl SettingsScreen {
    fn draw_filetree(&self, f: &mut Frame) {
        let size = f.size();
        let popup_width = 80;
        let popup_height = 25;
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        // Layout para dividir en área de búsqueda, lista y ayuda
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(popup_area);

        let search_area = popup_layout[0];
        let list_area = popup_layout[1];
        let help_area = popup_layout[2];

        // Área de búsqueda
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(" 🔍 Filter ", Style::default().fg(Color::Yellow)));

        let search_text = if self.search_filter.is_empty() {
            "<type to filter>".to_string()
        } else {
            self.search_filter.clone()
        };

        let search_style = if self.search_filter.is_empty() {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        };

        let search_paragraph = Paragraph::new(Span::styled(search_text, search_style))
            .block(search_block);

        f.render_widget(search_paragraph, search_area);

        // Lista de archivos con scroll
        let current_dir_str = self.current_dir.to_string_lossy();
        let title = format!(" 📁 Browse: {} ", current_dir_str);
        
        let filtered = self.get_filtered_entries();
        let visible_height = (list_area.height.saturating_sub(2)) as usize;
        
        let items: Vec<ListItem> = filtered
            .iter()
            .skip(self.scroll_offset)
            .take(visible_height)
            .enumerate()
            .map(|(display_idx, (_, entry))| {
                let actual_idx = display_idx + self.scroll_offset;
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let text = format!("{} {}", icon, entry.name);
                if actual_idx == self.selected_entry {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let list_title = if filtered.len() != self.file_entries.len() {
            format!("{} ({}/{})", title, filtered.len(), self.file_entries.len())
        } else {
            title
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(list_title, Style::default().fg(Color::Cyan)));

        let list = List::new(items).block(block);
        f.render_widget(list, list_area);

        // Área de ayuda
        let help_text = if matches!(self.focus_state, FocusState::ReposPath | FocusState::DocsPath) {
            "j/k: Navigate | Enter: Open folder | S: Select current folder | Esc: Cancel"
        } else {
            "j/k: Navigate | Enter: Select file/Open folder | Esc: Cancel"
        };
        
        let help_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        
        let help_paragraph = Paragraph::new(help_text)
            .block(help_block)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(help_paragraph, help_area);
    }

    fn draw_notification(&self, f: &mut Frame) {
        use ratatui::widgets::Clear;
        
        let size = f.size();
        let popup_width = 50;
        let popup_height = 7;
        let popup_x = (size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (size.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let (title, color) = if self.notification_is_error {
            (" ❌ Error ", Color::Red)
        } else {
            (" ✅ Success ", Color::Green)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(Span::styled(title, Style::default().fg(color)));

        // let paragraph = Paragraph::new(Line::from(&self.notification_message))
        let paragraph = Paragraph::new(Line::from(self.notification_message.as_str()))
            .alignment(Alignment::Center)
            .block(block);

        f.render_widget(paragraph, popup_area);
    }
}
