// src/presentation/screens/todo_screen.rs
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::fs;
use uuid::Uuid;
use chrono::Utc;

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;
use crate::domain::models::Task;
use crate::presentation::components::{task_popup::TaskPopup, move_popup::MovePopup, notification::Notification, task_component::draw_task_component};

pub struct TodoScreen {
    selected_section: Option<u8>,
    show_popup: bool,
    show_move_popup: bool,
    popup: TaskPopup,
    move_popup: MovePopup,
    tasks: Vec<Task>,
    selected_task_index: Option<usize>,
    section_task_indices: [usize; 4], // índices seleccionados para cada sección [pending, progress, block, done]
    editing_task_id: Option<Uuid>,
    notification: Option<Notification>,
    flipped_task_id: Option<Uuid>, // ID de la tarea que está volteada
    section_scroll_offsets: [usize; 4], // offset de scroll para cada sección
}

impl TodoScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            selected_section: None,
            show_popup: false,
            show_move_popup: false,
            popup: TaskPopup::new(),
            move_popup: MovePopup::new(),
            tasks: Vec::new(),
            selected_task_index: None,
            section_task_indices: [0, 0, 0, 0],
            editing_task_id: None,
            notification: None,
            flipped_task_id: None,
            section_scroll_offsets: [0, 0, 0, 0],
        };
        screen.load_tasks();
        screen
    }

    fn save_task(&mut self) -> Result<()> {
        let task = Task {
            id: Uuid::new_v4(),
            name: self.popup.name.clone(),
            description: self.popup.description.chars().take(96).collect(),
            state: self.popup.states[self.popup.state].clone(),
            priority: self.popup.priorities[self.popup.priority].clone(),
            tag: self.popup.tags[self.popup.tag].clone(),
            creation_date: Utc::now().format("%Y-%m-%d").to_string(),
            finish_date: self.popup.finish_date.clone(),
            alert: self.popup.alert,
        };

        // Crear carpeta data si no existe
        fs::create_dir_all("data")?;
        
        // Cargar tareas existentes o crear lista vacía
        let mut tasks: Vec<Task> = fs::read_to_string("data/task.json")
            .and_then(|content| serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            .unwrap_or_else(|_| Vec::new());
        
        // Agregar nueva tarea
        tasks.push(task);
        
        // Guardar todas las tareas
        let json = serde_json::to_string_pretty(&tasks)?;
        fs::write("data/task.json", json)?;
        
        // Recargar tareas en memoria
        self.load_tasks();
        
        // Mostrar notificación de éxito
        self.notification = Some(Notification::success("Task created successfully!".to_string()));
        
        Ok(())
    }

    fn move_task(&mut self) -> Result<()> {
        if let (Some(task_index), Some(new_state)) = (self.selected_task_index, self.move_popup.get_selected_state()) {
            if task_index < self.tasks.len() {
                self.tasks[task_index].state = new_state.clone();
                
                // Guardar todas las tareas
                let json = serde_json::to_string_pretty(&self.tasks)?;
                fs::write("data/task.json", json)?;
                
                // Mostrar notificación según el estado
                if new_state == "block" {
                    self.notification = Some(Notification::warning("Task moved to blocked state!".to_string()));
                }
            }
        }
        Ok(())
    }

    fn delete_task(&mut self) -> Result<()> {
        if let Some(section) = self.selected_section {
            let section_tasks = self.get_tasks_for_section(section);
            let selected_index = self.section_task_indices[(section - 1) as usize];
            
            if selected_index < section_tasks.len() {
                let task_id = section_tasks[selected_index].id;
                self.tasks.retain(|task| task.id != task_id);
                
                // Ajustar índice si es necesario
                let new_section_tasks = self.get_tasks_for_section(section);
                if new_section_tasks.is_empty() {
                    self.section_task_indices[(section - 1) as usize] = 0;
                } else if selected_index >= new_section_tasks.len() {
                    self.section_task_indices[(section - 1) as usize] = new_section_tasks.len() - 1;
                }
                
                // Guardar cambios
                let json = serde_json::to_string_pretty(&self.tasks)?;
                fs::write("data/task.json", json)?;
                
                // Mostrar notificación de eliminación
                self.notification = Some(Notification::error("Task deleted!".to_string()));
            }
        }
        Ok(())
    }

    fn load_task_for_edit(&mut self) {
        if let Some(section) = self.selected_section {
            let selected_index = self.section_task_indices[(section - 1) as usize];
            let state = match section {
                1 => "pending",
                2 => "progress", 
                3 => "block",
                4 => "done",
                _ => return,
            };
            
            let section_tasks: Vec<&Task> = self.tasks.iter().filter(|task| task.state == state).collect();
            
            if selected_index < section_tasks.len() {
                let task = section_tasks[selected_index];
                let task_id = task.id;
                let task_name = task.name.clone();
                let task_description = task.description.clone();
                let task_state = task.state.clone();
                let task_priority = task.priority.clone();
                let task_tag = task.tag.clone();
                let task_finish_date = task.finish_date.clone();
                let task_alert = task.alert;
                
                self.editing_task_id = Some(task_id);
                
                // Cargar datos de la tarea en el popup
                self.popup.name = task_name;
                self.popup.description = task_description;
                self.popup.finish_date = task_finish_date;
                self.popup.alert = task_alert;
                
                // Encontrar índices para state, priority y tag
                if let Some(state_index) = self.popup.states.iter().position(|s| s == &task_state) {
                    self.popup.state = state_index;
                }
                if let Some(priority_index) = self.popup.priorities.iter().position(|p| p == &task_priority) {
                    self.popup.priority = priority_index;
                }
                if let Some(tag_index) = self.popup.tags.iter().position(|t| t == &task_tag) {
                    self.popup.tag = tag_index;
                }
                
                self.show_popup = true;
            }
        }
    }

    fn update_task(&mut self) -> Result<()> {
        if let Some(task_id) = self.editing_task_id {
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                task.name = self.popup.name.clone();
                task.description = self.popup.description.chars().take(96).collect();
                task.state = self.popup.states[self.popup.state].clone();
                task.priority = self.popup.priorities[self.popup.priority].clone();
                task.tag = self.popup.tags[self.popup.tag].clone();
                task.finish_date = self.popup.finish_date.clone();
                task.alert = self.popup.alert;
                
                // Guardar cambios
                let json = serde_json::to_string_pretty(&self.tasks)?;
                fs::write("data/task.json", json)?;
                
                // Mostrar notificación de edición
                self.notification = Some(Notification::info("Task updated successfully!".to_string()));
            }
        }
        Ok(())
    }

    fn get_tasks_for_section(&self, section: u8) -> Vec<&Task> {
        let state = match section {
            1 => "pending",
            2 => "progress", 
            3 => "block",
            4 => "done",
            _ => return Vec::new(),
        };
        self.tasks.iter().filter(|task| task.state == state).collect()
    }

    fn load_tasks(&mut self) {
        self.tasks = match fs::read_to_string("data/task.json") {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
            }
            Err(_) => Vec::new()
        };
    }
}

impl Screen for TodoScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        if self.show_move_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_move_popup = false;
                    self.selected_task_index = None;
                }
                KeyCode::Enter => {
                    if let Err(_) = self.move_task() {
                        // Handle error silently for now
                    }
                    self.show_move_popup = false;
                    self.selected_task_index = None;
                }
                KeyCode::Char('j') => self.move_popup.next(),
                KeyCode::Char('k') => self.move_popup.previous(),
                _ => {}
            }
            return Ok(ScreenOutcome::Continue);
        }

        if self.show_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_popup = false;
                    self.popup = TaskPopup::new();
                    self.editing_task_id = None;
                }
                KeyCode::Enter => {
                    if self.editing_task_id.is_some() {
                        if let Err(_) = self.update_task() {
                            // Handle error silently for now
                        }
                        self.editing_task_id = None;
                    } else {
                        if let Err(_) = self.save_task() {
                            // Handle error silently for now
                        }
                    }
                    self.show_popup = false;
                    self.popup = TaskPopup::new();
                }
                KeyCode::Tab => {
                    let max_fields = if self.editing_task_id.is_some() { 6 } else { 7 };
                    self.popup.current_field = (self.popup.current_field + 1) % max_fields;
                }
                KeyCode::BackTab => {
                    let max_fields = if self.editing_task_id.is_some() { 6 } else { 7 };
                    self.popup.current_field = if self.popup.current_field == 0 { max_fields - 1 } else { self.popup.current_field - 1 };
                }
                KeyCode::Char('h') => {
                    let is_editing = self.editing_task_id.is_some();
                    match self.popup.current_field {
                        2 if !is_editing => self.popup.state = if self.popup.state == 0 { 3 } else { self.popup.state - 1 },
                        field if (is_editing && field == 2) || (!is_editing && field == 3) => {
                            self.popup.priority = if self.popup.priority == 0 { 3 } else { self.popup.priority - 1 }
                        },
                        field if (is_editing && field == 3) || (!is_editing && field == 4) => {
                            self.popup.tag = if self.popup.tag == 0 { self.popup.tags.len() - 1 } else { self.popup.tag - 1 }
                        },
                        field if (is_editing && field == 5) || (!is_editing && field == 6) => {
                            self.popup.alert = !self.popup.alert
                        },
                        0 => self.popup.name.push('h'),
                        1 => {
                            if self.popup.description.len() < 96 {
                                self.popup.description.push('h');
                            }
                        }
                        field if (is_editing && field == 4) || (!is_editing && field == 5) => {
                            self.popup.finish_date.push('h')
                        },
                        _ => {}
                    }
                }
                KeyCode::Char('l') => {
                    let is_editing = self.editing_task_id.is_some();
                    match self.popup.current_field {
                        2 if !is_editing => self.popup.state = (self.popup.state + 1) % 4,
                        field if (is_editing && field == 2) || (!is_editing && field == 3) => {
                            self.popup.priority = (self.popup.priority + 1) % 4
                        },
                        field if (is_editing && field == 3) || (!is_editing && field == 4) => {
                            self.popup.tag = (self.popup.tag + 1) % self.popup.tags.len()
                        },
                        field if (is_editing && field == 5) || (!is_editing && field == 6) => {
                            self.popup.alert = !self.popup.alert
                        },
                        0 => self.popup.name.push('l'),
                        1 => {
                            if self.popup.description.len() < 96 {
                                self.popup.description.push('l');
                            }
                        }
                        field if (is_editing && field == 4) || (!is_editing && field == 5) => {
                            self.popup.finish_date.push('l')
                        },
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    let is_editing = self.editing_task_id.is_some();
                    match self.popup.current_field {
                        0 => { self.popup.name.pop(); }
                        1 => { self.popup.description.pop(); }
                        field if (is_editing && field == 4) || (!is_editing && field == 5) => {
                            self.popup.finish_date.pop();
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    let is_editing = self.editing_task_id.is_some();
                    match self.popup.current_field {
                        0 => self.popup.name.push(c),
                        1 => {
                            if self.popup.description.len() < 96 {
                                self.popup.description.push(c);
                            }
                        }
                        field if (is_editing && field == 4) || (!is_editing && field == 5) => {
                            self.popup.finish_date.push(c)
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
            return Ok(ScreenOutcome::Continue);
        }

        match key.code {
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.show_popup = true;
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                if let Some(section) = self.selected_section {
                    let section_tasks = self.get_tasks_for_section(section);
                    if !section_tasks.is_empty() {
                        let selected_index = self.section_task_indices[(section - 1) as usize];
                        if selected_index < section_tasks.len() {
                            // Encontrar el índice global de la tarea seleccionada
                            if let Some(global_index) = self.tasks.iter().position(|t| t.id == section_tasks[selected_index].id) {
                                self.selected_task_index = Some(global_index);
                                self.show_move_popup = true;
                            }
                        }
                    }
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('j') => {
                if let Some(section) = self.selected_section {
                    let section_tasks = self.get_tasks_for_section(section);
                    if !section_tasks.is_empty() {
                        let current_index = self.section_task_indices[(section - 1) as usize];
                        if current_index < section_tasks.len() - 1 {
                            let new_index = current_index + 1;
                            self.section_task_indices[(section - 1) as usize] = new_index;
                            
                            // Ajustar scroll si es necesario
                            let available_slots = 3; // Aproximadamente 3 tareas visibles por recuadro
                            let scroll_offset = &mut self.section_scroll_offsets[(section - 1) as usize];
                            
                            if new_index >= *scroll_offset + available_slots {
                                *scroll_offset = new_index - available_slots + 1;
                            }
                        }
                    }
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('k') => {
                if let Some(section) = self.selected_section {
                    let section_tasks = self.get_tasks_for_section(section);
                    if !section_tasks.is_empty() {
                        let current_index = self.section_task_indices[(section - 1) as usize];
                        if current_index > 0 {
                            let new_index = current_index - 1;
                            self.section_task_indices[(section - 1) as usize] = new_index;
                            
                            // Ajustar scroll si es necesario
                            let scroll_offset = &mut self.section_scroll_offsets[(section - 1) as usize];
                            
                            if new_index < *scroll_offset {
                                *scroll_offset = new_index;
                            }
                        }
                    }
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Enter => {
                if let Some(section) = self.selected_section {
                    let section_tasks = self.get_tasks_for_section(section);
                    let selected_index = self.section_task_indices[(section - 1) as usize];
                    
                    if selected_index < section_tasks.len() {
                        let task_id = section_tasks[selected_index].id;
                        // Toggle flip: si ya está volteada, la des-voltea, si no, la voltea
                        if self.flipped_task_id == Some(task_id) {
                            self.flipped_task_id = None;
                        } else {
                            self.flipped_task_id = Some(task_id);
                        }
                    }
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if self.selected_section.is_some() {
                    if let Err(_) = self.delete_task() {
                        // Handle error silently for now
                    }
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if self.selected_section.is_some() {
                    self.load_task_for_edit();
                }
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('1') => {
                self.selected_section = Some(1);
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('2') => {
                self.selected_section = Some(2);
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('3') => {
                self.selected_section = Some(3);
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('4') => {
                self.selected_section = Some(4);
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Esc => {
                self.selected_section = None;
                Ok(ScreenOutcome::Continue)
            }
            KeyCode::Char('b') | KeyCode::Char('B') => Ok(ScreenOutcome::ChangeState(AppState::Home)),
            KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
            _ => Ok(ScreenOutcome::Continue),
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(size);

        let title_block = Line::from(vec![
            Span::styled("📋 ", Style::default().fg(Color::LightGreen)),
            Span::styled("Todo List", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightGreen))
            .title(title_block);

        // Calcular área interna antes de renderizar
        let todo_inner = main_block.inner(main_layout[0]);
        f.render_widget(main_block, main_layout[0]);

        // Layout interno del Todo List
        let todo_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(todo_inner);

        // Layout para los 4 recuadros superiores
        let top_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(todo_layout[0]);

        let upper_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(top_layout[0]);

        let lower_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(top_layout[1]);

        // Recuadro Pending
        let pending_color = if self.selected_section == Some(1) { Color::Yellow } else { Color::DarkGray };
        let pending_title = Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(pending_color)),
            Span::styled("Pending", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let pending_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(Style::default().fg(pending_color))
            .title(pending_title);
        let pending_inner = pending_block.inner(upper_row[0]);
        f.render_widget(pending_block, upper_row[0]);
        
        // Renderizar tareas pending
        let pending_tasks: Vec<&Task> = self.tasks.iter().filter(|task| task.state == "pending").collect();
        let available_height = (pending_inner.height / 4) as usize;
        let scroll_offset = self.section_scroll_offsets[0];
        
        for (i, task) in pending_tasks.iter().skip(scroll_offset).take(available_height).enumerate() {
            let y = pending_inner.y + (i * 4) as u16;
            let task_area = Rect {
                x: pending_inner.x,
                y,
                width: pending_inner.width,
                height: 4,
            };
            let actual_index = i + scroll_offset;
            let is_selected = self.selected_section == Some(1) && actual_index == self.section_task_indices[0];
            let is_flipped = self.flipped_task_id == Some(task.id);
            draw_task_component(f, task_area, task, is_selected, is_flipped);
        }

        // Recuadro Progress
        let progress_color = if self.selected_section == Some(2) { Color::Blue } else { Color::DarkGray };
        let progress_title = Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(progress_color)),
            Span::styled("Progress", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let progress_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(Style::default().fg(progress_color))
            .title(progress_title);
        let progress_inner = progress_block.inner(upper_row[1]);
        f.render_widget(progress_block, upper_row[1]);
        
        // Renderizar tareas progress
        let progress_tasks: Vec<&Task> = self.tasks.iter().filter(|task| task.state == "progress").collect();
        let scroll_offset = self.section_scroll_offsets[1];
        
        for (i, task) in progress_tasks.iter().skip(scroll_offset).take(available_height).enumerate() {
            let y = progress_inner.y + (i * 4) as u16;
            let task_area = Rect {
                x: progress_inner.x,
                y,
                width: progress_inner.width,
                height: 4,
            };
            let actual_index = i + scroll_offset;
            let is_selected = self.selected_section == Some(2) && actual_index == self.section_task_indices[1];
            let is_flipped = self.flipped_task_id == Some(task.id);
            draw_task_component(f, task_area, task, is_selected, is_flipped);
        }

        // Recuadro Block
        let block_color = if self.selected_section == Some(3) { Color::Red } else { Color::DarkGray };
        let block_title = Line::from(vec![
            Span::styled("🚫 ", Style::default().fg(block_color)),
            Span::styled("Block", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let block_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(Style::default().fg(block_color))
            .title(block_title);
        let block_inner = block_block.inner(lower_row[0]);
        f.render_widget(block_block, lower_row[0]);
        
        // Renderizar tareas block
        let block_tasks: Vec<&Task> = self.tasks.iter().filter(|task| task.state == "block").collect();
        let scroll_offset = self.section_scroll_offsets[2];
        
        for (i, task) in block_tasks.iter().skip(scroll_offset).take(available_height).enumerate() {
            let y = block_inner.y + (i * 4) as u16;
            let task_area = Rect {
                x: block_inner.x,
                y,
                width: block_inner.width,
                height: 4,
            };
            let actual_index = i + scroll_offset;
            let is_selected = self.selected_section == Some(3) && actual_index == self.section_task_indices[2];
            let is_flipped = self.flipped_task_id == Some(task.id);
            draw_task_component(f, task_area, task, is_selected, is_flipped);
        }

        // Recuadro Done
        let done_color = if self.selected_section == Some(4) { Color::Green } else { Color::DarkGray };
        let done_title = Line::from(vec![
            Span::styled("✅ ", Style::default().fg(done_color)),
            Span::styled("Done", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let done_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(Style::default().fg(done_color))
            .title(done_title);
        let done_inner = done_block.inner(lower_row[1]);
        f.render_widget(done_block, lower_row[1]);
        
        // Renderizar tareas done
        let done_tasks: Vec<&Task> = self.tasks.iter().filter(|task| task.state == "done").collect();
        let scroll_offset = self.section_scroll_offsets[3];
        
        for (i, task) in done_tasks.iter().skip(scroll_offset).take(available_height).enumerate() {
            let y = done_inner.y + (i * 4) as u16;
            let task_area = Rect {
                x: done_inner.x,
                y,
                width: done_inner.width,
                height: 4,
            };
            let actual_index = i + scroll_offset;
            let is_selected = self.selected_section == Some(4) && actual_index == self.section_task_indices[3];
            let is_flipped = self.flipped_task_id == Some(task.id);
            draw_task_component(f, task_area, task, is_selected, is_flipped);
        }

        // Recuadro Options
        let options_title = Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Cyan)),
            Span::styled("Options", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let options_text = Paragraph::new("A: Add | E: Edit | D: Delete | M: Move")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(options_title)
            );
        f.render_widget(options_text, todo_layout[1]);

        // Menú inferior
        let menu_block_title = Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
            Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let menu_text = Paragraph::new("Q: Quit | B: Back | 1-4: Panels")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .title(menu_block_title)
            );
        f.render_widget(menu_text, main_layout[1]);

        // Mostrar popup si está activo
        if self.show_popup {
            self.popup.draw(f, self.editing_task_id.is_some());
        }

        // Mostrar popup de mover si está activo
        if self.show_move_popup {
            self.move_popup.draw(f);
        }

        // Mostrar notificación si existe
        if let Some(ref notification) = self.notification {
            if notification.is_expired() {
                self.notification = None;
            } else {
                notification.draw(f, f.size());
            }
        }
    }
}
