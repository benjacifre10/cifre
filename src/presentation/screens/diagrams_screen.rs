use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;

pub struct DiagramsScreen {
    show_options_popup: bool,
    selected_option: usize,
    show_diagram_type_popup: bool,
    selected_diagram_type: usize,
    show_sequence_canvas: bool,
    show_diagram_name_popup: bool,
    diagram_name_input: String,
    current_diagram_name: String,
    show_add_object_popup: bool,
    object_name_input: String,
    selected_object_type: usize,
    artifact_types: Vec<String>,
    show_load_popup: bool,
    available_diagrams: Vec<String>,
    selected_diagram: usize,
    show_add_message_popup: bool,
    message_description_input: String,
    message_notes_input: String,
    selected_from_object: usize,
    selected_to_object: usize,
    current_message_field: usize, // 0=description, 1=from, 2=to, 3=notes
    available_objects: Vec<String>,
    view_objects_mode: bool,
    view_messages_mode: bool,
    focused_object_index: usize,
    focused_message_index: usize,
    show_message_notes_popup: bool,
    current_message_notes: String,
    show_edit_object_popup: bool,
    show_edit_message_popup: bool,
    edit_object_name_input: String,
    edit_selected_object_type: usize,
    show_delete_confirmation_popup: bool,
    delete_confirmation_message: String,
    show_export_popup: bool,
    selected_export_format: usize,
    diagram_navigation_mode: bool,
    scroll_offset_x: i32,
    scroll_offset_y: i32,
}

impl DiagramsScreen {
    pub fn new() -> Self {
        Self {
            show_options_popup: false,
            selected_option: 0,
            show_diagram_type_popup: false,
            selected_diagram_type: 0,
            show_sequence_canvas: false,
            show_diagram_name_popup: false,
            diagram_name_input: String::new(),
            current_diagram_name: String::new(),
            show_add_object_popup: false,
            object_name_input: String::new(),
            selected_object_type: 0,
            artifact_types: vec!["ms".to_string(), "lambda".to_string(), "db".to_string(), "mobile".to_string(), "bff".to_string()],
            show_load_popup: false,
            available_diagrams: Vec::new(),
            selected_diagram: 0,
            show_add_message_popup: false,
            message_description_input: String::new(),
            message_notes_input: String::new(),
            selected_from_object: 0,
            selected_to_object: 0,
            current_message_field: 0,
            available_objects: Vec::new(),
            view_objects_mode: false,
            view_messages_mode: false,
            focused_object_index: 0,
            focused_message_index: 0,
            show_message_notes_popup: false,
            current_message_notes: String::new(),
            show_edit_object_popup: false,
            show_edit_message_popup: false,
            edit_object_name_input: String::new(),
            edit_selected_object_type: 0,
            show_delete_confirmation_popup: false,
            delete_confirmation_message: String::new(),
            show_export_popup: false,
            selected_export_format: 0,
            diagram_navigation_mode: false,
            scroll_offset_x: 0,
            scroll_offset_y: 0,
        }
    }
}

impl Screen for DiagramsScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        if self.show_export_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_export_popup = false;
                    self.selected_export_format = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    self.selected_export_format = (self.selected_export_format + 1) % 2;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    self.selected_export_format = if self.selected_export_format == 0 { 1 } else { 0 };
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    let format = if self.selected_export_format == 0 { "png" } else { "pdf" };
                    match self.export_diagram(format) {
                        Ok(_path) => {
                            // TODO: Show success notification
                        }
                        Err(_) => {
                            // TODO: Show error notification
                        }
                    }
                    self.show_export_popup = false;
                    self.selected_export_format = 0;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_delete_confirmation_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_delete_confirmation_popup = false;
                    self.delete_confirmation_message.clear();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    self.delete_object_with_messages();
                    self.show_delete_confirmation_popup = false;
                    self.delete_confirmation_message.clear();
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_message_notes_popup {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.show_message_notes_popup = false;
                    self.current_message_notes.clear();
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_edit_object_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_edit_object_popup = false;
                    self.edit_object_name_input.clear();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    self.edit_selected_object_type = (self.edit_selected_object_type + 1) % self.artifact_types.len();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    self.edit_selected_object_type = if self.edit_selected_object_type == 0 { 
                        self.artifact_types.len() - 1 
                    } else { 
                        self.edit_selected_object_type - 1 
                    };
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Tab => {
                    self.edit_selected_object_type = (self.edit_selected_object_type + 1) % self.artifact_types.len();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char(c) => {
                    self.edit_object_name_input.push(c);
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Backspace => {
                    self.edit_object_name_input.pop();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.edit_object_name_input.is_empty() {
                        self.update_object();
                        self.show_edit_object_popup = false;
                        self.edit_object_name_input.clear();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_edit_message_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_edit_message_popup = false;
                    self.message_description_input.clear();
                    self.message_notes_input.clear();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Tab => {
                    self.current_message_field = (self.current_message_field + 1) % 4; // 4 fields
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    if self.current_message_field == 1 && !self.available_objects.is_empty() { // from dropdown
                        self.selected_from_object = (self.selected_from_object + 1) % self.available_objects.len();
                    } else if self.current_message_field == 2 && !self.available_objects.is_empty() { // to dropdown
                        self.selected_to_object = (self.selected_to_object + 1) % self.available_objects.len();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    if self.current_message_field == 1 && !self.available_objects.is_empty() { // from dropdown
                        self.selected_from_object = if self.selected_from_object == 0 { 
                            self.available_objects.len() - 1 
                        } else { 
                            self.selected_from_object - 1 
                        };
                    } else if self.current_message_field == 2 && !self.available_objects.is_empty() { // to dropdown
                        self.selected_to_object = if self.selected_to_object == 0 { 
                            self.available_objects.len() - 1 
                        } else { 
                            self.selected_to_object - 1 
                        };
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char(c) => {
                    match self.current_message_field {
                        0 => self.message_description_input.push(c), // description
                        3 => self.message_notes_input.push(c),       // notes
                        _ => {} // from/to are dropdowns, no text input
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Backspace => {
                    match self.current_message_field {
                        0 => { self.message_description_input.pop(); }
                        3 => { self.message_notes_input.pop(); }
                        _ => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.message_description_input.is_empty() {
                        self.update_message();
                        self.show_edit_message_popup = false;
                        self.message_description_input.clear();
                        self.message_notes_input.clear();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_add_message_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_add_message_popup = false;
                    self.message_description_input.clear();
                    self.message_notes_input.clear();
                    self.current_message_field = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Tab => {
                    self.current_message_field = (self.current_message_field + 1) % 4; // 4 fields
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    // j/k always work on the currently focused field
                    if self.current_message_field == 1 && !self.available_objects.is_empty() { // from dropdown
                        self.selected_from_object = (self.selected_from_object + 1) % self.available_objects.len();
                    } else if self.current_message_field == 2 && !self.available_objects.is_empty() { // to dropdown
                        self.selected_to_object = (self.selected_to_object + 1) % self.available_objects.len();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    // j/k always work on the currently focused field
                    if self.current_message_field == 1 && !self.available_objects.is_empty() { // from dropdown
                        self.selected_from_object = if self.selected_from_object == 0 { 
                            self.available_objects.len() - 1 
                        } else { 
                            self.selected_from_object - 1 
                        };
                    } else if self.current_message_field == 2 && !self.available_objects.is_empty() { // to dropdown
                        self.selected_to_object = if self.selected_to_object == 0 { 
                            self.available_objects.len() - 1 
                        } else { 
                            self.selected_to_object - 1 
                        };
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char(c) => {
                    match self.current_message_field {
                        0 => self.message_description_input.push(c), // description
                        3 => self.message_notes_input.push(c),       // notes
                        _ => {} // from/to are dropdowns, no text input
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Backspace => {
                    match self.current_message_field {
                        0 => { self.message_description_input.pop(); }
                        3 => { self.message_notes_input.pop(); }
                        _ => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.message_description_input.is_empty() && !self.available_objects.is_empty() {
                        self.save_message_to_diagram();
                        self.show_add_message_popup = false;
                        self.message_description_input.clear();
                        self.message_notes_input.clear();
                        self.current_message_field = 0;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_load_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_load_popup = false;
                    self.available_diagrams.clear();
                    self.selected_diagram = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    if !self.available_diagrams.is_empty() {
                        self.selected_diagram = (self.selected_diagram + 1) % self.available_diagrams.len();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    if !self.available_diagrams.is_empty() {
                        self.selected_diagram = if self.selected_diagram == 0 { 
                            self.available_diagrams.len() - 1 
                        } else { 
                            self.selected_diagram - 1 
                        };
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.available_diagrams.is_empty() && self.selected_diagram < self.available_diagrams.len() {
                        self.current_diagram_name = self.available_diagrams[self.selected_diagram].clone();
                        self.show_load_popup = false;
                        self.available_diagrams.clear();
                        self.selected_diagram = 0;
                        self.show_sequence_canvas = true;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_add_object_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_add_object_popup = false;
                    self.object_name_input.clear();
                    self.selected_object_type = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    self.selected_object_type = (self.selected_object_type + 1) % self.artifact_types.len();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    self.selected_object_type = if self.selected_object_type == 0 { 
                        self.artifact_types.len() - 1 
                    } else { 
                        self.selected_object_type - 1 
                    };
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char(c) => {
                    self.object_name_input.push(c);
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Backspace => {
                    self.object_name_input.pop();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Tab => {
                    self.selected_object_type = (self.selected_object_type + 1) % self.artifact_types.len();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.object_name_input.is_empty() {
                        // Save object to diagram file
                        self.save_object_to_diagram();
                        self.show_add_object_popup = false;
                        self.object_name_input.clear();
                        self.selected_object_type = 0;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_diagram_name_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_diagram_name_popup = false;
                    self.diagram_name_input.clear();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char(c) => {
                    self.diagram_name_input.push(c);
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Backspace => {
                    self.diagram_name_input.pop();
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    if !self.diagram_name_input.is_empty() {
                        self.current_diagram_name = self.diagram_name_input.clone();
                        // Create empty diagram file
                        self.create_empty_diagram_file();
                        self.show_diagram_name_popup = false;
                        self.diagram_name_input.clear();
                        self.show_sequence_canvas = true;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_sequence_canvas {
            if self.diagram_navigation_mode {
                match key.code {
                    KeyCode::Esc => {
                        self.diagram_navigation_mode = false;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        if self.diagram_needs_horizontal_scroll() {
                            self.scroll_offset_x = (self.scroll_offset_x - 5).max(-100);
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        if self.diagram_needs_horizontal_scroll() {
                            self.scroll_offset_x = (self.scroll_offset_x + 5).min(100);
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') | KeyCode::Char('K') => {
                        if self.diagram_needs_vertical_scroll() {
                            self.scroll_offset_y = (self.scroll_offset_y - 3).max(-50);
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') | KeyCode::Char('J') => {
                        if self.diagram_needs_vertical_scroll() {
                            self.scroll_offset_y = (self.scroll_offset_y + 3).min(50);
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            } else if self.view_objects_mode || self.view_messages_mode {
                match key.code {
                    KeyCode::Esc => {
                        self.view_objects_mode = false;
                        self.view_messages_mode = false;
                        self.focused_object_index = 0;
                        self.focused_message_index = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Tab => {
                        if self.view_objects_mode {
                            let objects = self.load_diagram_objects();
                            if !objects.is_empty() {
                                self.focused_object_index = (self.focused_object_index + 1) % objects.len();
                            }
                        } else if self.view_messages_mode {
                            let messages = self.load_diagram_messages();
                            if !messages.is_empty() {
                                self.focused_message_index = (self.focused_message_index + 1) % messages.len();
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if self.view_messages_mode {
                            let messages = self.load_diagram_messages();
                            if self.focused_message_index < messages.len() {
                                let message = &messages[self.focused_message_index];
                                self.current_message_notes = message["notes"].as_str().unwrap_or("").to_string();
                                if self.current_message_notes.is_empty() {
                                    self.current_message_notes = "Note empty".to_string();
                                }
                                self.show_message_notes_popup = true;
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if self.view_objects_mode {
                            let objects = self.load_diagram_objects();
                            if self.focused_object_index < objects.len() {
                                let obj = &objects[self.focused_object_index];
                                self.edit_object_name_input = obj["name"].as_str().unwrap_or("").to_string();
                                let obj_type = obj["type"].as_str().unwrap_or("ms");
                                self.edit_selected_object_type = self.artifact_types.iter().position(|t| t == obj_type).unwrap_or(0);
                                self.show_edit_object_popup = true;
                            }
                        } else if self.view_messages_mode {
                            let messages = self.load_diagram_messages();
                            if self.focused_message_index < messages.len() {
                                let message = &messages[self.focused_message_index];
                                self.message_description_input = message["description"].as_str().unwrap_or("").to_string();
                                self.message_notes_input = message["notes"].as_str().unwrap_or("").to_string();
                                
                                // Find object indices
                                self.load_available_objects();
                                let from_name = message["from"].as_str().unwrap_or("");
                                let to_name = message["to"].as_str().unwrap_or("");
                                self.selected_from_object = self.available_objects.iter().position(|n| n == from_name).unwrap_or(0);
                                self.selected_to_object = self.available_objects.iter().position(|n| n == to_name).unwrap_or(0);
                                self.current_message_field = 0;
                                self.show_edit_message_popup = true;
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if self.view_objects_mode {
                            let objects = self.load_diagram_objects();
                            if self.focused_object_index < objects.len() {
                                let obj_name = objects[self.focused_object_index]["name"].as_str().unwrap_or("");
                                let associated_messages = self.count_messages_for_object(obj_name);
                                
                                if associated_messages > 0 {
                                    self.delete_confirmation_message = format!(
                                        "Object '{}' has {} associated message(s).\nPress Enter to delete object and all its messages, or Esc to cancel.",
                                        obj_name, associated_messages
                                    );
                                    self.show_delete_confirmation_popup = true;
                                } else {
                                    self.delete_object();
                                }
                            }
                        } else if self.view_messages_mode {
                            self.delete_message();
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        self.show_sequence_canvas = false;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.show_add_object_popup = true;
                        self.object_name_input.clear();
                        self.selected_object_type = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        self.load_available_objects();
                        if !self.available_objects.is_empty() {
                            self.show_add_message_popup = true;
                            self.message_description_input.clear();
                            self.message_notes_input.clear();
                            self.selected_from_object = 0;
                            self.selected_to_object = 0;
                            self.current_message_field = 0;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        let objects = self.load_diagram_objects();
                        if !objects.is_empty() {
                            self.view_objects_mode = true;
                            self.focused_object_index = 0;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        let messages = self.load_diagram_messages();
                        if !messages.is_empty() {
                            self.view_messages_mode = true;
                            self.focused_message_index = 0;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.show_export_popup = true;
                        self.selected_export_format = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Tab => {
                        self.diagram_navigation_mode = true;
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
        } else if self.show_diagram_type_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_diagram_type_popup = false;
                    self.selected_diagram_type = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    self.selected_diagram_type = (self.selected_diagram_type + 1) % 3;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    self.selected_diagram_type = if self.selected_diagram_type == 0 { 2 } else { self.selected_diagram_type - 1 };
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    match self.selected_diagram_type {
                        0 => {
                            self.show_diagram_type_popup = false;
                            self.selected_diagram_type = 0;
                            self.show_diagram_name_popup = true;
                            self.diagram_name_input.clear();
                        }
                        1 => {
                            // TODO: Create State diagram
                        }
                        2 => {
                            // TODO: Create Flow diagram
                        }
                        _ => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_options_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_options_popup = false;
                    self.selected_option = 0;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('j') => {
                    self.selected_option = (self.selected_option + 1) % 2;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('k') => {
                    self.selected_option = if self.selected_option == 0 { 1 } else { self.selected_option - 1 };
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Enter => {
                    match self.selected_option {
                        0 => {
                            self.show_options_popup = false;
                            self.selected_option = 0;
                            self.show_diagram_type_popup = true;
                        }
                        1 => {
                            // Load option - show available diagrams
                            self.load_available_diagrams();
                            if !self.available_diagrams.is_empty() {
                                self.show_options_popup = false;
                                self.selected_option = 0;
                                self.show_load_popup = true;
                                self.selected_diagram = 0;
                            }
                        }
                        _ => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.show_options_popup = false;
                    self.selected_option = 0;
                    self.show_diagram_type_popup = true;
                    Ok(ScreenOutcome::Continue)
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    // Load option - show available diagrams
                    self.load_available_diagrams();
                    if !self.available_diagrams.is_empty() {
                        self.show_options_popup = false;
                        self.selected_option = 0;
                        self.show_load_popup = true;
                        self.selected_diagram = 0;
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
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    self.show_options_popup = true;
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        if self.show_sequence_canvas {
            self.draw_sequence_canvas(f);
        } else {
            self.draw_main_screen(f);
        }

        // Draw popups
        if self.show_export_popup {
            self.draw_export_popup(f);
        } else if self.show_delete_confirmation_popup {
            self.draw_delete_confirmation_popup(f);
        } else if self.show_message_notes_popup {
            self.draw_message_notes_popup(f);
        } else if self.show_edit_object_popup {
            self.draw_edit_object_popup(f);
        } else if self.show_edit_message_popup {
            self.draw_edit_message_popup(f);
        } else if self.show_add_message_popup {
            self.draw_add_message_popup(f);
        } else if self.show_load_popup {
            self.draw_load_popup(f);
        } else if self.show_add_object_popup {
            self.draw_add_object_popup(f);
        } else if self.show_diagram_name_popup {
            self.draw_diagram_name_popup(f);
        } else if self.show_options_popup {
            self.draw_options_popup(f);
        } else if self.show_diagram_type_popup {
            self.draw_diagram_type_popup(f);
        }
    }
}

impl DiagramsScreen {
    fn draw_main_screen(&self, f: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.size());

        let title_block = Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::LightBlue)),
            Span::styled("Diagrams", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title_block);

        let content = Paragraph::new("Diagrams content will go here").block(main_block);
        f.render_widget(content, main_layout[0]);

        let menu_text = Paragraph::new("B: Back | Q: Quit | O: Options")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .title(Line::from(vec![
                        Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
                        Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
                    ]))
            );
        
        f.render_widget(menu_text, main_layout[1]);
    }

    fn draw_sequence_canvas(&self, f: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3), Constraint::Length(3)])
            .split(f.size());

        // Main canvas area with objects integrated
        let canvas_title = Line::from(vec![
            Span::styled("🔄 ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                format!("Sequence Diagram: {}", self.current_diagram_name),
                Style::default().add_modifier(Modifier::BOLD)
            ),
        ]);
        
        let canvas_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(canvas_title);

        // Draw objects and messages in the main canvas
        self.draw_canvas_content(f, main_layout[0], canvas_block);

        // Submenu area
        let submenu_text = if self.diagram_navigation_mode {
            "Navigation Mode - h/j/k/l: Move | Esc: Exit"
        } else if self.view_objects_mode {
            "E: Edit | D: Delete | Tab: Next | Esc: Exit"
        } else if self.view_messages_mode {
            "E: Edit | D: Delete | Tab: Next | Esc: Exit"
        } else {
            "A: Add Object | L: Link Message | O: View Objects | M: View Messages | P: Print | Tab: Navigate"
        };
        
        let submenu = Paragraph::new(submenu_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(Span::styled(" Submenu ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            );
        
        f.render_widget(submenu, main_layout[1]);

        // Menu area
        let menu_text = Paragraph::new("B: Back | Q: Quit")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .title(Line::from(vec![
                        Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
                        Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
                    ]))
            );
        
        f.render_widget(menu_text, main_layout[2]);
    }

    fn draw_canvas_content(&self, f: &mut Frame, area: Rect, block: Block) {
        let objects = self.load_diagram_objects();
        
        if objects.is_empty() {
            let content = Paragraph::new("No objects yet. Press A to add objects.")
                .alignment(Alignment::Center)
                .block(block);
            f.render_widget(content, area);
            return;
        }

        // Draw the main block first
        f.render_widget(block, area);
        
        // Inner area for content
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Calculate object dimensions
        let max_name_len = objects.iter()
            .map(|obj| obj["name"].as_str().unwrap_or("").len())
            .max()
            .unwrap_or(8);
        
        let object_width = (max_name_len + 4).max(10);
        let object_height = 4;
        let spacing = 2;
        
        // Calculate layout
        let total_objects = objects.len();
        let total_width = total_objects * object_width + (total_objects.saturating_sub(1)) * spacing;
        let available_width = inner_area.width as usize;

        if total_width > available_width {
            let content = Paragraph::new(format!("{} objects (use horizontal scroll)", total_objects))
                .alignment(Alignment::Center);
            f.render_widget(content, inner_area);
            return;
        }

        // Calculate object positions
        let start_x = (available_width - total_width) / 2;
        let mut object_positions = Vec::new();
        
        // Draw objects and store their positions
        for (i, obj) in objects.iter().enumerate() {
            let x = start_x + i * (object_width + spacing);
            let obj_area = Rect {
                x: (inner_area.x as i32 + x as i32 + self.scroll_offset_x).max(inner_area.x as i32) as u16,
                y: (inner_area.y as i32 + self.scroll_offset_y).max(inner_area.y as i32) as u16,
                width: object_width as u16,
                height: object_height,
            };
            
            let is_focused = self.view_objects_mode && i == self.focused_object_index;
            self.draw_single_object(f, obj_area, obj, is_focused);
            
            // Store center position for lifelines and messages
            let center_x = obj_area.x + obj_area.width / 2;
            object_positions.push((obj["name"].as_str().unwrap_or("").to_string(), center_x));
        }

        // Draw lifelines (dotted vertical lines below objects)
        let lifeline_start_y = (inner_area.y as i32 + object_height as i32 + self.scroll_offset_y).max(inner_area.y as i32) as u16;
        let lifeline_height = inner_area.height.saturating_sub(object_height);
        
        for (_, center_x) in &object_positions {
            for y in 0..lifeline_height {
                if y % 2 == 0 { // Dotted line effect
                    let cell_area = Rect {
                        x: *center_x,
                        y: lifeline_start_y + y,
                        width: 1,
                        height: 1,
                    };
                    let dot = Paragraph::new("│").style(Style::default().fg(Color::Gray));
                    f.render_widget(dot, cell_area);
                }
            }
        }

        // Draw messages
        let messages = self.load_diagram_messages();
        let message_spacing = 4; // Increased spacing for self-calls
        
        for (i, message) in messages.iter().enumerate() {
            let from_name = message["from"].as_str().unwrap_or("");
            let to_name = message["to"].as_str().unwrap_or("");
            let description = message["description"].as_str().unwrap_or("");
            let order = message["order"].as_u64().unwrap_or((i + 1) as u64);
            
            // Find object positions
            let from_pos = object_positions.iter().find(|(name, _)| name == from_name);
            let to_pos = object_positions.iter().find(|(name, _)| name == to_name);
            
            if let (Some((_, from_x)), Some((_, to_x))) = (from_pos, to_pos) {
                let message_y = lifeline_start_y + 2 + (i * message_spacing) as u16;
                
                if message_y < inner_area.y + inner_area.height - 3 { // -3 for self-call height
                    let is_focused = self.view_messages_mode && i == self.focused_message_index;
                    self.draw_message_arrow(f, *from_x, *to_x, message_y, description, order, is_focused);
                }
            }
        }
    }

    fn draw_message_arrow(&self, f: &mut Frame, from_x: u16, to_x: u16, y: u16, description: &str, order: u64, is_focused: bool) {
        let arrow_color = if is_focused { Color::Gray } else { Color::Yellow };
        let desc_color = if is_focused { Color::Gray } else { Color::Cyan };
        
        if from_x == to_x {
            // Self-call: draw loop arrow with proper corners
            let loop_width = 6;
            
            // Horizontal line going right
            for x in (from_x + 1)..(from_x + loop_width - 1) {
                let line_area = Rect { x, y, width: 1, height: 1 };
                let line = Paragraph::new("─").style(Style::default().fg(arrow_color));
                f.render_widget(line, line_area);
            }
            
            // Top-right corner
            let corner_area = Rect { x: from_x + loop_width - 1, y, width: 1, height: 1 };
            let corner = Paragraph::new("┐").style(Style::default().fg(arrow_color));
            f.render_widget(corner, corner_area);
            
            // Vertical line going down
            let line_area = Rect { x: from_x + loop_width - 1, y: y + 1, width: 1, height: 1 };
            let line = Paragraph::new("│").style(Style::default().fg(arrow_color));
            f.render_widget(line, line_area);
            
            // Bottom-right corner with arrow
            let corner_area = Rect { x: from_x + loop_width - 1, y: y + 2, width: 1, height: 1 };
            let corner = Paragraph::new("┘").style(Style::default().fg(arrow_color));
            f.render_widget(corner, corner_area);
            
            // Horizontal line coming back with arrow
            for x in from_x..(from_x + loop_width - 1) {
                let line_area = Rect { x, y: y + 2, width: 1, height: 1 };
                let line_char = if x == from_x { "<" } else { "─" };
                let line = Paragraph::new(line_char).style(Style::default().fg(arrow_color));
                f.render_widget(line, line_area);
            }
            
            // Draw order number and description above the outgoing line
            if y > 0 {
                let order_text = format!("{}:", order);
                let full_text = if description.is_empty() {
                    order_text
                } else {
                    format!("{} {}", order_text, description)
                };
                
                let desc_width = full_text.len().min(25) as u16;
                let desc_area = Rect {
                    x: from_x + 1,
                    y: y - 1,
                    width: desc_width,
                    height: 1,
                };
                
                let desc_text = if full_text.len() > 25 {
                    format!("{}...", &full_text[..22])
                } else {
                    full_text
                };
                
                let desc_paragraph = Paragraph::new(desc_text)
                    .style(Style::default().fg(desc_color));
                f.render_widget(desc_paragraph, desc_area);
            }
        } else {
            // Regular arrow between different objects
            let (start_x, end_x, arrow_char) = if from_x < to_x {
                (from_x, to_x, ">")
            } else {
                (to_x, from_x, "<")
            };
            
            // Draw horizontal line
            for x in start_x..=end_x {
                let line_area = Rect { x, y, width: 1, height: 1 };
                let line_char = if x == end_x { arrow_char } else { "─" };
                let line = Paragraph::new(line_char).style(Style::default().fg(arrow_color));
                f.render_widget(line, line_area);
            }
            
            // Draw order number and description above the arrow
            if y > 0 {
                let order_text = format!("{}:", order);
                let full_text = if description.is_empty() {
                    order_text
                } else {
                    format!("{} {}", order_text, description)
                };
                
                let desc_x = (start_x + end_x) / 2;
                let desc_width = full_text.len().min(25) as u16;
                let desc_start_x = desc_x.saturating_sub(desc_width / 2);
                
                let desc_area = Rect {
                    x: desc_start_x,
                    y: y - 1,
                    width: desc_width,
                    height: 1,
                };
                
                let desc_text = if full_text.len() > 25 {
                    format!("{}...", &full_text[..22])
                } else {
                    full_text
                };
                
                let desc_paragraph = Paragraph::new(desc_text)
                    .style(Style::default().fg(desc_color))
                    .alignment(Alignment::Center);
                f.render_widget(desc_paragraph, desc_area);
            }
        }
    }

    fn draw_single_object(&self, f: &mut Frame, area: Rect, obj: &serde_json::Value, is_focused: bool) {
        let name = obj["name"].as_str().unwrap_or("Unknown");
        let obj_type = obj["type"].as_str().unwrap_or("unknown");
        
        // Get color based on type, or gray if focused
        let color = if is_focused {
            Color::Gray
        } else {
            match obj_type {
                "ms" => Color::Blue,
                "lambda" => Color::Rgb(255, 165, 0), // Orange
                "db" => Color::Green,
                "mobile" => Color::Magenta,
                "bff" => Color::Rgb(139, 69, 19), // Brown
                _ => Color::White,
            }
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .border_type(ratatui::widgets::BorderType::Double)
            .title(Span::styled(
                format!(" {} ", obj_type.to_uppercase()),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ));
        
        // Handle long names by wrapping or truncating intelligently
        let content = if name.len() > area.width.saturating_sub(2) as usize {
            // If name is too long, try to wrap it
            let mid = name.len() / 2;
            if let Some(space_pos) = name[..mid].rfind(' ') {
                format!("{}\n{}", &name[..space_pos], &name[space_pos + 1..])
            } else if let Some(dash_pos) = name[..mid].rfind('-') {
                format!("{}-\n{}", &name[..dash_pos], &name[dash_pos + 1..])
            } else {
                // Truncate with ellipsis
                format!("{}...", &name[..area.width.saturating_sub(5) as usize])
            }
        } else {
            name.to_string()
        };
        
        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(block);
        
        f.render_widget(paragraph, area);
    }

    fn load_diagram_messages(&self) -> Vec<serde_json::Value> {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(messages) = diagram["messages"].as_array() {
                    let mut sorted_messages = messages.clone();
                    sorted_messages.sort_by(|a, b| {
                        let order_a = a["order"].as_u64().unwrap_or(0);
                        let order_b = b["order"].as_u64().unwrap_or(0);
                        order_a.cmp(&order_b)
                    });
                    return sorted_messages;
                }
            }
        }
        
        Vec::new()
    }

    fn load_diagram_objects(&self) -> Vec<serde_json::Value> {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(objects) = diagram["objects"].as_array() {
                    let mut sorted_objects = objects.clone();
                    // Sort by order field
                    sorted_objects.sort_by(|a, b| {
                        let order_a = a["order"].as_u64().unwrap_or(0);
                        let order_b = b["order"].as_u64().unwrap_or(0);
                        order_a.cmp(&order_b)
                    });
                    return sorted_objects;
                }
            }
        }
        
        Vec::new()
    }

    fn draw_options_popup(&self, f: &mut Frame) {
        let options = vec!["N: New", "L: Load"];
        let max_option_width = options.iter().map(|s| s.len()).max().unwrap_or(0);
        let popup_width = (max_option_width + 4).max(20) as u16;
        let popup_height = (options.len() + 4) as u16;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Options ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

        let mut lines = vec![Line::from("")];
        for (i, option) in options.iter().enumerate() {
            let style = if i == self.selected_option {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(format!(" {}", option), style)));
        }
        lines.push(Line::from(""));

        let content = Text::from(lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_diagram_type_popup(&self, f: &mut Frame) {
        let diagram_types = vec!["Sequence", "State", "Flow"];
        let max_type_width = diagram_types.iter().map(|s| s.len()).max().unwrap_or(0);
        let popup_width = (max_type_width + 4).max(20) as u16;
        let popup_height = (diagram_types.len() + 4) as u16;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Diagram Type ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));

        let mut lines = vec![Line::from("")];
        for (i, diagram_type) in diagram_types.iter().enumerate() {
            let style = if i == self.selected_diagram_type {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(format!(" {}", diagram_type), style)));
        }
        lines.push(Line::from(""));

        let content = Text::from(lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_diagram_name_popup(&self, f: &mut Frame) {
        let popup_width = 40;
        let popup_height = 7;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Diagram Name ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)));

        let content = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("Name:", Style::default().fg(Color::White))),
            Line::from(Span::styled(
                format!("> {}", self.diagram_name_input),
                Style::default().fg(Color::Yellow)
            )),
            Line::from(""),
        ]);

        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_edit_object_popup(&self, f: &mut Frame) {
        let popup_width = 50;
        let popup_height = 10;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Edit Object ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)));

        let selected_type = &self.artifact_types[self.edit_selected_object_type];
        
        let content = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("Name:", Style::default().fg(Color::White))),
            Line::from(Span::styled(
                format!("> {}", self.edit_object_name_input),
                Style::default().fg(Color::Yellow)
            )),
            Line::from(""),
            Line::from(Span::styled("Type (j/k or Tab):", Style::default().fg(Color::White))),
            Line::from(Span::styled(
                format!("> {}", selected_type),
                Style::default().fg(Color::Green)
            )),
            Line::from(""),
        ]);

        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_edit_message_popup(&self, f: &mut Frame) {
        let popup_width = 70;
        let popup_height = 16;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Edit Message ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));

        let from_object = if self.selected_from_object < self.available_objects.len() {
            &self.available_objects[self.selected_from_object]
        } else {
            "None"
        };
        
        let to_object = if self.selected_to_object < self.available_objects.len() {
            &self.available_objects[self.selected_to_object]
        } else {
            "None"
        };

        let mut lines = vec![Line::from("")];
        
        // Description field
        let desc_style = if self.current_message_field == 0 { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        lines.push(Line::from(Span::styled("Description (Tab to switch):", desc_style)));
        lines.push(Line::from(Span::styled(format!("> {}", self.message_description_input), Style::default().fg(Color::Yellow))));
        lines.push(Line::from(""));
        
        // From field
        let from_style = if self.current_message_field == 1 { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        lines.push(Line::from(Span::styled("From (j/k to change):", from_style)));
        lines.push(Line::from(Span::styled(format!("> {}", from_object), Style::default().fg(Color::Green))));
        lines.push(Line::from(""));
        
        // To field
        let to_style = if self.current_message_field == 2 { Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        lines.push(Line::from(Span::styled("To (j/k to change):", to_style)));
        lines.push(Line::from(Span::styled(format!("> {}", to_object), Style::default().fg(Color::Blue))));
        lines.push(Line::from(""));
        
        // Notes field
        let notes_style = if self.current_message_field == 3 { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        lines.push(Line::from(Span::styled("Notes:", notes_style)));
        lines.push(Line::from(Span::styled(format!("> {}", self.message_notes_input), Style::default().fg(Color::Cyan))));

        let content = Text::from(lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_message_notes_popup(&self, f: &mut Frame) {
        let popup_width = 60;
        let popup_height = 8;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Message Notes ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));

        let content = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("Notes:", Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled(
                &self.current_message_notes,
                Style::default().fg(Color::Yellow)
            )),
            Line::from(""),
            Line::from(Span::styled("Press Esc or Enter to close", Style::default().fg(Color::Gray))),
        ]);

        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_add_object_popup(&self, f: &mut Frame) {
        let popup_width = 50;
        let popup_height = 10;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Add Object ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));

        let selected_type = &self.artifact_types[self.selected_object_type];
        
        let content = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("Name:", Style::default().fg(Color::White))),
            Line::from(Span::styled(
                format!("> {}", self.object_name_input),
                Style::default().fg(Color::Yellow)
            )),
            Line::from(""),
            Line::from(Span::styled("Type (j/k or Tab):", Style::default().fg(Color::White))),
            Line::from(Span::styled(
                format!("> {}", selected_type),
                Style::default().fg(Color::Green)
            )),
            Line::from(""),
        ]);

        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn create_empty_diagram_file(&self) {
        use std::fs;
        
        // Create data/diagrams directory if it doesn't exist
        let _ = fs::create_dir_all("data/diagrams");
        
        // Create empty diagram structure
        let empty_diagram = serde_json::json!({
            "name": self.current_diagram_name,
            "type": "sequence",
            "objects": [],
            "messages": []
        });
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        let _ = fs::write(file_path, serde_json::to_string_pretty(&empty_diagram).unwrap_or_default());
    }

    fn save_object_to_diagram(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        // Read existing diagram
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                // Calculate next order (number of existing objects + 1)
                let next_order = if let Some(objects) = diagram["objects"].as_array() {
                    objects.len() + 1
                } else {
                    1
                };
                
                // Add new object with order
                let new_object = serde_json::json!({
                    "name": self.object_name_input,
                    "type": self.artifact_types[self.selected_object_type],
                    "order": next_order
                });
                
                if let Some(objects) = diagram["objects"].as_array_mut() {
                    objects.push(new_object);
                }
                
                // Save updated diagram
                let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
            }
        }
    }

    fn load_available_diagrams(&mut self) {
        use std::fs;
        
        self.available_diagrams.clear();
        
        if let Ok(entries) = fs::read_dir("data/diagrams") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name() {
                            if let Some(name_str) = file_name.to_str() {
                                if name_str.ends_with(".json") {
                                    // Remove .json extension
                                    let diagram_name = name_str.trim_end_matches(".json").to_string();
                                    self.available_diagrams.push(diagram_name);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort diagrams alphabetically
        self.available_diagrams.sort();
    }

    fn draw_load_popup(&self, f: &mut Frame) {
        if self.available_diagrams.is_empty() {
            // Show "no diagrams" message
            let popup_width = 30;
            let popup_height = 5;
            
            let size = f.size();
            let popup_area = Rect {
                x: (size.width.saturating_sub(popup_width)) / 2,
                y: (size.height.saturating_sub(popup_height)) / 2,
                width: popup_width,
                height: popup_height,
            };

            f.render_widget(Clear, popup_area);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .border_type(ratatui::widgets::BorderType::Thick)
                .title(Span::styled(" Load Diagram ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

            let content = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled("No diagrams found", Style::default().fg(Color::White))),
            ]);

            let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(block);
            f.render_widget(paragraph, popup_area);
            return;
        }

        // Calculate popup size based on content
        let max_name_len = self.available_diagrams.iter().map(|s| s.len()).max().unwrap_or(0);
        let popup_width = (max_name_len + 6).max(25) as u16; // +6 for padding and selection indicator
        let popup_height = (self.available_diagrams.len() + 4).min(15) as u16; // Max 15 lines
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Load Diagram ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let mut lines = vec![Line::from("")];
        for (i, diagram_name) in self.available_diagrams.iter().enumerate() {
            let style = if i == self.selected_diagram {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };
            
            lines.push(Line::from(Span::styled(
                format!(" {}", diagram_name),
                style
            )));
        }
        lines.push(Line::from(""));

        let content = Text::from(lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn load_available_objects(&mut self) {
        self.available_objects.clear();
        let objects = self.load_diagram_objects();
        
        for obj in objects {
            if let Some(name) = obj["name"].as_str() {
                self.available_objects.push(name.to_string());
            }
        }
    }

    fn export_diagram(&self, format: &str) -> Result<String> {
        use std::env;
        use std::path::PathBuf;
        
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        let downloads_dir = PathBuf::from(home_dir).join("Downloads");
        
        let filename = format!("{}.{}", self.current_diagram_name.replace(" ", "_"), format);
        let filepath = downloads_dir.join(&filename);
        
        match format {
            "png" => self.export_to_png(&filepath)?,
            "pdf" => self.export_to_pdf(&filepath)?,
            _ => return Err(anyhow::anyhow!("Formato no soportado")),
        }
        
        Ok(filepath.to_string_lossy().to_string())
    }

    fn export_to_png(&self, filepath: &std::path::Path) -> Result<()> {
        use image::{Rgb, RgbImage};
        use imageproc::drawing::{draw_text_mut, draw_hollow_rect_mut, draw_line_segment_mut};
        use imageproc::rect::Rect as ImgRect;
        use rusttype::{Font, Scale};
        
        let objects = self.load_diagram_objects();
        let messages = self.load_diagram_messages();
        
        if objects.is_empty() {
            return Err(anyhow::anyhow!("No objects to export"));
        }
        
        let object_width = 120i32;
        let object_height = 60i32;
        let spacing = 80i32;
        let lifeline_length = 300i32 + (messages.len() as i32 * 40);
        let width = 100 + (objects.len() as i32 * (object_width + spacing));
        let height = 150 + object_height + lifeline_length;
        
        let mut img = RgbImage::from_pixel(width as u32, height as u32, Rgb([255, 255, 255]));
        
        let font_data = include_bytes!("/System/Library/Fonts/Helvetica.ttc");
        let font = Font::try_from_bytes(font_data as &[u8]).ok_or_else(|| anyhow::anyhow!("Error loading font"))?;
        
        // Title
        let title = format!("Sequence Diagram: {}", self.current_diagram_name);
        draw_text_mut(&mut img, Rgb([0, 0, 0]), 20, 20, Scale::uniform(20.0), &font, &title);
        
        // Draw objects and lifelines
        let mut object_positions = Vec::new();
        let objects_y = 80i32;
        
        for (i, obj) in objects.iter().enumerate() {
            let x = 50 + (i as i32 * (object_width + spacing));
            let center_x = x + object_width / 2;
            
            let name = obj["name"].as_str().unwrap_or("Unknown");
            let obj_type = obj["type"].as_str().unwrap_or("unknown");
            
            // Draw object rectangle
            draw_hollow_rect_mut(&mut img, ImgRect::at(x, objects_y).of_size(object_width as u32, object_height as u32), Rgb([0, 0, 0]));
            
            // Draw object text
            draw_text_mut(&mut img, Rgb([0, 0, 0]), x + 5, objects_y + 10, Scale::uniform(12.0), &font, &obj_type.to_uppercase());
            draw_text_mut(&mut img, Rgb([0, 0, 0]), x + 5, objects_y + 30, Scale::uniform(14.0), &font, name);
            
            // Draw lifeline (dotted vertical line)
            let lifeline_start = objects_y + object_height;
            for y in (lifeline_start..lifeline_start + lifeline_length).step_by(4) {
                if (y - lifeline_start) % 8 == 0 {
                    img.put_pixel(center_x as u32, y as u32, Rgb([128, 128, 128]));
                }
            }
            
            object_positions.push((name.to_string(), center_x));
        }
        
        // Draw messages
        let message_start_y = objects_y + object_height + 20;
        for (i, message) in messages.iter().enumerate() {
            let from_name = message["from"].as_str().unwrap_or("");
            let to_name = message["to"].as_str().unwrap_or("");
            let description = message["description"].as_str().unwrap_or("");
            let order = message["order"].as_u64().unwrap_or(i as u64 + 1);
            
            let from_pos = object_positions.iter().find(|(name, _)| name == from_name);
            let to_pos = object_positions.iter().find(|(name, _)| name == to_name);
            
            if let (Some((_, from_x)), Some((_, to_x))) = (from_pos, to_pos) {
                let y = message_start_y + (i as i32 * 40);
                
                if from_x == to_x {
                    // Self-call: draw loop
                    let loop_width = 40i32;
                    // Horizontal line right
                    draw_line_segment_mut(&mut img, (*from_x as f32, y as f32), (*from_x as f32 + loop_width as f32, y as f32), Rgb([255, 165, 0]));
                    // Vertical line down
                    draw_line_segment_mut(&mut img, (*from_x as f32 + loop_width as f32, y as f32), (*from_x as f32 + loop_width as f32, y as f32 + 20.0), Rgb([255, 165, 0]));
                    // Horizontal line back
                    draw_line_segment_mut(&mut img, (*from_x as f32 + loop_width as f32, y as f32 + 20.0), (*from_x as f32, y as f32 + 20.0), Rgb([255, 165, 0]));
                } else {
                    // Regular arrow with triangular head
                    draw_line_segment_mut(&mut img, (*from_x as f32, y as f32), (*to_x as f32, y as f32), Rgb([255, 165, 0]));
                    
                    // Draw triangular arrow head
                    let arrow_size = 8.0;
                    if from_x < to_x {
                        // Right arrow
                        let points = [
                            (*to_x as f32, y as f32),
                            (*to_x as f32 - arrow_size, y as f32 - arrow_size/2.0),
                            (*to_x as f32 - arrow_size, y as f32 + arrow_size/2.0),
                        ];
                        for i in 0..3 {
                            let next = (i + 1) % 3;
                            draw_line_segment_mut(&mut img, (points[i].0, points[i].1), (points[next].0, points[next].1), Rgb([255, 165, 0]));
                        }
                    } else {
                        // Left arrow
                        let points = [
                            (*to_x as f32, y as f32),
                            (*to_x as f32 + arrow_size, y as f32 - arrow_size/2.0),
                            (*to_x as f32 + arrow_size, y as f32 + arrow_size/2.0),
                        ];
                        for i in 0..3 {
                            let next = (i + 1) % 3;
                            draw_line_segment_mut(&mut img, (points[i].0, points[i].1), (points[next].0, points[next].1), Rgb([255, 165, 0]));
                        }
                    }
                }
                
                // Draw message description
                let desc_text = format!("{}: {}", order, description);
                let desc_x = if from_x == to_x { *from_x + 45 } else { (*from_x + *to_x) / 2 - 50 };
                draw_text_mut(&mut img, Rgb([0, 150, 150]), desc_x, y - 15, Scale::uniform(12.0), &font, &desc_text);
            }
        }
        
        img.save(filepath)?;
        Ok(())
    }

    fn export_to_pdf(&self, filepath: &std::path::Path) -> Result<()> {
        use printpdf::*;
        use std::fs::File;
        use std::io::BufWriter;
        
        let objects = self.load_diagram_objects();
        let messages = self.load_diagram_messages();
        
        if objects.is_empty() {
            return Err(anyhow::anyhow!("No objects to export"));
        }
        
        let (doc, page1, layer1) = PdfDocument::new(&format!("Sequence Diagram: {}", self.current_diagram_name), Mm(297.0), Mm(210.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);
        
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        
        // Title
        current_layer.use_text(&format!("Sequence Diagram: {}", self.current_diagram_name), 16.0, Mm(20.0), Mm(180.0), &font_bold);
        
        // Draw objects
        current_layer.use_text("Objects:", 12.0, Mm(20.0), Mm(160.0), &font_bold);
        let mut y_pos = 150.0;
        for (i, obj) in objects.iter().enumerate() {
            let name = obj["name"].as_str().unwrap_or("Unknown");
            let obj_type = obj["type"].as_str().unwrap_or("unknown");
            
            // Draw object as text with box representation
            let text = format!("[{}] {} ({})", i + 1, name, obj_type.to_uppercase());
            current_layer.use_text(&text, 10.0, Mm(30.0), Mm(y_pos), &font);
            
            // Draw lifeline representation
            current_layer.use_text("    |", 8.0, Mm(35.0), Mm(y_pos - 5.0), &font);
            current_layer.use_text("    |", 8.0, Mm(35.0), Mm(y_pos - 10.0), &font);
            current_layer.use_text("    |", 8.0, Mm(35.0), Mm(y_pos - 15.0), &font);
            
            y_pos -= 25.0;
            if y_pos < 50.0 { break; }
        }
        
        // Draw messages
        y_pos -= 10.0;
        current_layer.use_text("Messages:", 12.0, Mm(20.0), Mm(y_pos), &font_bold);
        y_pos -= 15.0;
        
        for message in messages.iter() {
            let description = message["description"].as_str().unwrap_or("Unknown");
            let from = message["from"].as_str().unwrap_or("Unknown");
            let to = message["to"].as_str().unwrap_or("Unknown");
            let order = message["order"].as_u64().unwrap_or(1);
            
            let arrow = if from == to { "↻" } else { "→" };
            let text = format!("{}. {} {} {} : {}", order, from, arrow, to, description);
            current_layer.use_text(&text, 9.0, Mm(30.0), Mm(y_pos), &font);
            y_pos -= 8.0;
            
            if y_pos < 20.0 { break; }
        }
        
        doc.save(&mut BufWriter::new(File::create(filepath)?))?;
        Ok(())
    }

    fn draw_export_popup(&self, f: &mut Frame) {
        let formats = vec!["PNG", "PDF"];
        let popup_width = 30;
        let popup_height = 8;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Export Format ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));

        let mut lines = vec![Line::from("")];
        for (i, format) in formats.iter().enumerate() {
            let style = if i == self.selected_export_format {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(format!("  {}", format), style)));
        }
        lines.push(Line::from(""));

        let content = Text::from(lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn diagram_needs_horizontal_scroll(&self) -> bool {
        let objects = self.load_diagram_objects();
        if objects.is_empty() {
            return false;
        }
        
        let object_width = 120;
        let spacing = 80;
        let total_width = objects.len() * (object_width + spacing);
        
        // Assume typical screen width minus borders
        total_width > 150
    }

    fn diagram_needs_vertical_scroll(&self) -> bool {
        let messages = self.load_diagram_messages();
        
        // Assume typical screen height minus borders and headers
        let message_height = messages.len() * 40;
        message_height > 300
    }

    fn count_messages_for_object(&self, obj_name: &str) -> usize {
        let messages = self.load_diagram_messages();
        messages.iter().filter(|msg| {
            msg["from"].as_str() == Some(obj_name) || msg["to"].as_str() == Some(obj_name)
        }).count()
    }

    fn delete_object(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(objects) = diagram["objects"].as_array_mut() {
                    if self.focused_object_index < objects.len() {
                        objects.remove(self.focused_object_index);
                        let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
                    }
                }
            }
        }
    }

    fn delete_object_with_messages(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                // Get object name first
                let obj_name = if let Some(objects) = diagram["objects"].as_array() {
                    if self.focused_object_index < objects.len() {
                        objects[self.focused_object_index]["name"].as_str().unwrap_or("").to_string()
                    } else {
                        return;
                    }
                } else {
                    return;
                };
                
                // Remove object
                if let Some(objects_mut) = diagram["objects"].as_array_mut() {
                    if self.focused_object_index < objects_mut.len() {
                        objects_mut.remove(self.focused_object_index);
                    }
                }
                
                // Remove associated messages
                if let Some(messages) = diagram["messages"].as_array_mut() {
                    messages.retain(|msg| {
                        msg["from"].as_str() != Some(&obj_name) && msg["to"].as_str() != Some(&obj_name)
                    });
                    
                    // Reorder remaining messages
                    for (i, message) in messages.iter_mut().enumerate() {
                        message["order"] = serde_json::Value::Number(serde_json::Number::from(i + 1));
                    }
                }
                
                let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
            }
        }
    }

    fn delete_message(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(messages) = diagram["messages"].as_array_mut() {
                    if self.focused_message_index < messages.len() {
                        messages.remove(self.focused_message_index);
                        
                        // Reorder remaining messages
                        for (i, message) in messages.iter_mut().enumerate() {
                            message["order"] = serde_json::Value::Number(serde_json::Number::from(i + 1));
                        }
                        
                        let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
                    }
                }
            }
        }
    }

    fn draw_delete_confirmation_popup(&self, f: &mut Frame) {
        let popup_width = 70;
        let popup_height = 10;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Confirm Delete ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));

        // Split message into lines to fit popup width
        let max_line_width = (popup_width - 4) as usize; // -4 for borders and padding
        let lines: Vec<Line> = self.delete_confirmation_message
            .split('\n')
            .flat_map(|line| {
                if line.len() <= max_line_width {
                    vec![Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))]
                } else {
                    // Wrap long lines
                    let mut wrapped_lines = Vec::new();
                    let mut remaining = line;
                    while remaining.len() > max_line_width {
                        let split_pos = remaining[..max_line_width].rfind(' ').unwrap_or(max_line_width);
                        wrapped_lines.push(Line::from(Span::styled(&remaining[..split_pos], Style::default().fg(Color::Yellow))));
                        remaining = &remaining[split_pos..].trim_start();
                    }
                    if !remaining.is_empty() {
                        wrapped_lines.push(Line::from(Span::styled(remaining, Style::default().fg(Color::Yellow))));
                    }
                    wrapped_lines
                }
            })
            .collect();

        let mut content_lines = vec![Line::from("")];
        content_lines.extend(lines);
        content_lines.push(Line::from(""));

        let content = Text::from(content_lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Center).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn update_object(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(objects) = diagram["objects"].as_array_mut() {
                    if self.focused_object_index < objects.len() {
                        let old_name = objects[self.focused_object_index]["name"].as_str().unwrap_or("").to_string();
                        let new_name = self.edit_object_name_input.clone();
                        
                        // Update object
                        objects[self.focused_object_index]["name"] = serde_json::Value::String(new_name.clone());
                        objects[self.focused_object_index]["type"] = serde_json::Value::String(self.artifact_types[self.edit_selected_object_type].clone());
                        
                        // Update message references
                        if let Some(messages) = diagram["messages"].as_array_mut() {
                            for message in messages.iter_mut() {
                                if message["from"].as_str() == Some(&old_name) {
                                    message["from"] = serde_json::Value::String(new_name.clone());
                                }
                                if message["to"].as_str() == Some(&old_name) {
                                    message["to"] = serde_json::Value::String(new_name.clone());
                                }
                            }
                        }
                        
                        let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
                    }
                }
            }
        }
    }

    fn update_message(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(messages) = diagram["messages"].as_array_mut() {
                    if self.focused_message_index < messages.len() {
                        let from_object = if self.selected_from_object < self.available_objects.len() {
                            &self.available_objects[self.selected_from_object]
                        } else {
                            ""
                        };
                        
                        let to_object = if self.selected_to_object < self.available_objects.len() {
                            &self.available_objects[self.selected_to_object]
                        } else {
                            ""
                        };
                        
                        messages[self.focused_message_index]["description"] = serde_json::Value::String(self.message_description_input.clone());
                        messages[self.focused_message_index]["from"] = serde_json::Value::String(from_object.to_string());
                        messages[self.focused_message_index]["to"] = serde_json::Value::String(to_object.to_string());
                        messages[self.focused_message_index]["notes"] = serde_json::Value::String(self.message_notes_input.clone());
                        
                        let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
                    }
                }
            }
        }
    }

    fn save_message_to_diagram(&self) {
        use std::fs;
        
        let file_path = format!("data/diagrams/{}.json", self.current_diagram_name);
        
        // Read existing diagram
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(mut diagram) = serde_json::from_str::<serde_json::Value>(&content) {
                // Calculate next order (number of existing messages + 1)
                let next_order = if let Some(messages) = diagram["messages"].as_array() {
                    messages.len() + 1
                } else {
                    1
                };
                
                let from_object = if self.selected_from_object < self.available_objects.len() {
                    &self.available_objects[self.selected_from_object]
                } else {
                    ""
                };
                
                let to_object = if self.selected_to_object < self.available_objects.len() {
                    &self.available_objects[self.selected_to_object]
                } else {
                    ""
                };
                
                // Add new message
                let new_message = serde_json::json!({
                    "description": self.message_description_input,
                    "from": from_object,
                    "to": to_object,
                    "notes": self.message_notes_input,
                    "order": next_order
                });
                
                if let Some(messages) = diagram["messages"].as_array_mut() {
                    messages.push(new_message);
                }
                
                // Save updated diagram
                let _ = fs::write(file_path, serde_json::to_string_pretty(&diagram).unwrap_or_default());
            }
        }
    }

    fn draw_add_message_popup(&self, f: &mut Frame) {
        let popup_width = 60;
        let popup_height = 16;
        
        let size = f.size();
        let popup_area = Rect {
            x: (size.width.saturating_sub(popup_width)) / 2,
            y: (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(Span::styled(" Add Message ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));

        let from_object = if self.selected_from_object < self.available_objects.len() {
            &self.available_objects[self.selected_from_object]
        } else {
            "No objects"
        };
        
        let to_object = if self.selected_to_object < self.available_objects.len() {
            &self.available_objects[self.selected_to_object]
        } else {
            "No objects"
        };
        
        let mut lines = vec![Line::from("")];
        
        // Description field
        let desc_style = if self.current_message_field == 0 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled("Description:", desc_style)));
        lines.push(Line::from(Span::styled(
            format!("> {}", self.message_description_input),
            if self.current_message_field == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Gray) }
        )));
        lines.push(Line::from(""));
        
        // From field
        let from_style = if self.current_message_field == 1 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled("From (j/k to change):", from_style)));
        lines.push(Line::from(Span::styled(
            format!("> {}", from_object),
            if self.current_message_field == 1 { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Gray) }
        )));
        lines.push(Line::from(""));
        
        // To field
        let to_style = if self.current_message_field == 2 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled("To (j/k to change):", to_style)));
        lines.push(Line::from(Span::styled(
            format!("> {}", to_object),
            if self.current_message_field == 2 { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Gray) }
        )));
        lines.push(Line::from(""));
        
        // Notes field
        let notes_style = if self.current_message_field == 3 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled("Notes:", notes_style)));
        lines.push(Line::from(Span::styled(
            format!("> {}", self.message_notes_input),
            if self.current_message_field == 3 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Gray) }
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Tab: Next field | Enter: Save", Style::default().fg(Color::Cyan))));

        let content = Text::from(lines);
        let paragraph = Paragraph::new(content).alignment(Alignment::Left).block(block);
        f.render_widget(paragraph, popup_area);
    }
}
