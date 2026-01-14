use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::fs;

use super::screen::{Screen, ScreenContext, ScreenOutcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Property {
    id: String,
    street_name: String,
    street_number: String,
    city: String,
    state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Category {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Service {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PropertyServiceTemplate {
    property_id: String,
    service_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixedBill {
    property_id: String,
    service_id: String,
    dia_vencimiento: u8,
    monto: f64,
}

pub struct BillsScreen {
    show_options_menu: bool,
    selected_option: usize,
    show_properties_panel: bool,
    properties: Vec<Property>,
    focused_property_index: usize,
    show_property_popup: bool,
    is_editing_property: bool,
    property_form: PropertyForm,
    current_field: usize,
    show_categories_panel: bool,
    categories: Vec<Category>,
    focused_category_index: usize,
    show_category_popup: bool,
    is_editing_category: bool,
    category_form: CategoryForm,
    show_services_panel: bool,
    services: Vec<Service>,
    focused_service_index: usize,
    show_service_popup: bool,
    is_editing_service: bool,
    service_form: ServiceForm,
    show_gastos_panel: bool,
    show_gastos_fijos: bool,
    show_template_menu: bool,
    selected_template_option: usize,
    templates: Vec<PropertyServiceTemplate>,
    focused_template_index: usize,
    current_month: u8,
    current_year: u16,
    fixed_bills: Vec<FixedBill>,
    focused_bill_index: usize,
}

#[derive(Debug, Clone)]
struct PropertyForm {
    street_name: String,
    street_number: String,
    city: String,
    state: String,
}

#[derive(Debug, Clone)]
struct CategoryForm {
    name: String,
}

#[derive(Debug, Clone)]
struct ServiceForm {
    name: String,
}

impl BillsScreen {
    pub fn new() -> Self {
        let mut screen = BillsScreen {
            show_options_menu: false,
            selected_option: 0,
            show_properties_panel: false,
            properties: Vec::new(),
            focused_property_index: 0,
            show_property_popup: false,
            is_editing_property: false,
            property_form: PropertyForm {
                street_name: String::new(),
                street_number: String::new(),
                city: String::new(),
                state: String::new(),
            },
            current_field: 0,
            show_categories_panel: false,
            categories: Vec::new(),
            focused_category_index: 0,
            show_category_popup: false,
            is_editing_category: false,
            category_form: CategoryForm {
                name: String::new(),
            },
            show_services_panel: false,
            services: Vec::new(),
            focused_service_index: 0,
            show_service_popup: false,
            is_editing_service: false,
            service_form: ServiceForm {
                name: String::new(),
            },
            show_gastos_panel: false,
            show_gastos_fijos: true,
            show_template_menu: false,
            selected_template_option: 0,
            templates: Vec::new(),
            focused_template_index: 0,
            current_month: 1,
            current_year: 2026,
            fixed_bills: Vec::new(),
            focused_bill_index: 0,
        };
        screen.load_properties();
        screen.load_categories();
        screen.load_services();
        screen.load_templates();
        screen.load_fixed_bills();
        screen
    }

    fn load_properties(&mut self) {
        if let Ok(content) = fs::read_to_string("data/bills/property.json") {
            if let Ok(properties) = serde_json::from_str::<Vec<Property>>(&content) {
                self.properties = properties;
            }
        }
    }

    fn save_properties(&self) {
        if let Err(_) = fs::create_dir_all("data/bills") {
            return;
        }
        
        if let Ok(json) = serde_json::to_string_pretty(&self.properties) {
            let _ = fs::write("data/bills/property.json", json);
        }
    }

    fn load_categories(&mut self) {
        if let Ok(content) = fs::read_to_string("data/bills/category.json") {
            if let Ok(categories) = serde_json::from_str::<Vec<Category>>(&content) {
                self.categories = categories;
            }
        }
    }

    fn save_categories(&self) {
        if let Err(_) = fs::create_dir_all("data/bills") {
            return;
        }
        
        if let Ok(json) = serde_json::to_string_pretty(&self.categories) {
            let _ = fs::write("data/bills/category.json", json);
        }
    }

    fn generate_category_id(&self) -> String {
        if self.categories.is_empty() {
            "1".to_string()
        } else {
            let max_id = self.categories.iter()
                .filter_map(|c| c.id.parse::<u32>().ok())
                .max()
                .unwrap_or(0);
            (max_id + 1).to_string()
        }
    }

    fn load_services(&mut self) {
        if let Ok(content) = fs::read_to_string("data/bills/services.json") {
            if let Ok(services) = serde_json::from_str::<Vec<Service>>(&content) {
                self.services = services;
            }
        }
    }

    fn save_services(&self) {
        if let Err(_) = fs::create_dir_all("data/bills") {
            return;
        }
        
        if let Ok(json) = serde_json::to_string_pretty(&self.services) {
            let _ = fs::write("data/bills/services.json", json);
        }
    }

    fn generate_service_id(&self) -> String {
        if self.services.is_empty() {
            "1".to_string()
        } else {
            let max_id = self.services.iter()
                .filter_map(|s| s.id.parse::<u32>().ok())
                .max()
                .unwrap_or(0);
            (max_id + 1).to_string()
        }
    }

    fn load_templates(&mut self) {
        if let Ok(content) = fs::read_to_string("data/bills/property_services_template.json") {
            if let Ok(templates) = serde_json::from_str::<Vec<PropertyServiceTemplate>>(&content) {
                self.templates = templates;
            }
        }
    }

    fn save_templates(&self) {
        if let Err(_) = fs::create_dir_all("data/bills") {
            return;
        }
        
        if let Ok(json) = serde_json::to_string_pretty(&self.templates) {
            let _ = fs::write("data/bills/property_services_template.json", json);
        }
    }

    fn load_fixed_bills(&mut self) {
        let filename = format!("data/bills/bills_fixed_{:02}_{}.json", self.current_month, self.current_year);
        if let Ok(content) = fs::read_to_string(&filename) {
            if let Ok(bills) = serde_json::from_str::<Vec<FixedBill>>(&content) {
                self.fixed_bills = bills;
            }
        }
    }

    fn save_fixed_bills(&self) {
        if let Err(_) = fs::create_dir_all("data/bills") {
            return;
        }
        
        let filename = format!("data/bills/bills_fixed_{:02}_{}.json", self.current_month, self.current_year);
        if let Ok(json) = serde_json::to_string_pretty(&self.fixed_bills) {
            let _ = fs::write(&filename, json);
        }
    }

    fn generate_fixed_bills_from_template(&mut self) {
        self.fixed_bills.clear();
        for template in &self.templates {
            for service_id in &template.service_ids {
                let bill = FixedBill {
                    property_id: template.property_id.clone(),
                    service_id: service_id.clone(),
                    dia_vencimiento: 1,
                    monto: 0.0,
                };
                self.fixed_bills.push(bill);
            }
        }
        self.save_fixed_bills();
    }

    fn get_property_name(&self, property_id: &str) -> String {
        self.properties.iter()
            .find(|p| p.id == property_id)
            .map(|p| format!("{} {}", p.street_name, p.street_number))
            .unwrap_or_else(|| "Unknown Property".to_string())
    }

    fn get_service_name(&self, service_id: &str) -> String {
        self.services.iter()
            .find(|s| s.id == service_id)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Unknown Service".to_string())
    }

    fn generate_id(&self) -> String {
        if self.properties.is_empty() {
            "1".to_string()
        } else {
            let max_id = self.properties.iter()
                .filter_map(|p| p.id.parse::<u32>().ok())
                .max()
                .unwrap_or(0);
            (max_id + 1).to_string()
        }
    }
}

impl Screen for BillsScreen {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<ScreenOutcome> {
        if self.show_property_popup {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_property_popup = false;
                    self.is_editing_property = false;
                    self.property_form = PropertyForm {
                        street_name: String::new(),
                        street_number: String::new(),
                        city: String::new(),
                        state: String::new(),
                    };
                    self.current_field = 0;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Tab => {
                    self.current_field = (self.current_field + 1) % 4;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char(c) => {
                    match self.current_field {
                        0 => self.property_form.street_name.push(c),
                        1 => self.property_form.street_number.push(c),
                        2 => self.property_form.city.push(c),
                        3 => self.property_form.state.push(c),
                        _ => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Backspace => {
                    match self.current_field {
                        0 => { self.property_form.street_name.pop(); }
                        1 => { self.property_form.street_number.pop(); }
                        2 => { self.property_form.city.pop(); }
                        3 => { self.property_form.state.pop(); }
                        _ => {}
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if !self.property_form.street_name.is_empty() {
                        if self.is_editing_property && self.focused_property_index < self.properties.len() {
                            self.properties[self.focused_property_index].street_name = self.property_form.street_name.clone();
                            self.properties[self.focused_property_index].street_number = self.property_form.street_number.clone();
                            self.properties[self.focused_property_index].city = self.property_form.city.clone();
                            self.properties[self.focused_property_index].state = self.property_form.state.clone();
                        } else {
                            let new_property = Property {
                                id: self.generate_id(),
                                street_name: self.property_form.street_name.clone(),
                                street_number: self.property_form.street_number.clone(),
                                city: self.property_form.city.clone(),
                                state: self.property_form.state.clone(),
                            };
                            self.properties.push(new_property);
                        }
                        
                        self.save_properties();
                        self.show_property_popup = false;
                        self.is_editing_property = false;
                        self.property_form = PropertyForm {
                            street_name: String::new(),
                            street_number: String::new(),
                            city: String::new(),
                            state: String::new(),
                        };
                        self.current_field = 0;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_category_popup {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_category_popup = false;
                    self.is_editing_category = false;
                    self.category_form = CategoryForm {
                        name: String::new(),
                    };
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char(c) => {
                    self.category_form.name.push(c);
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Backspace => {
                    self.category_form.name.pop();
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if !self.category_form.name.is_empty() {
                        if self.is_editing_category && self.focused_category_index < self.categories.len() {
                            self.categories[self.focused_category_index].name = self.category_form.name.clone();
                        } else {
                            let new_category = Category {
                                id: self.generate_category_id(),
                                name: self.category_form.name.clone(),
                            };
                            self.categories.push(new_category);
                        }
                        
                        self.save_categories();
                        self.show_category_popup = false;
                        self.is_editing_category = false;
                        self.category_form = CategoryForm {
                            name: String::new(),
                        };
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_service_popup {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_service_popup = false;
                    self.is_editing_service = false;
                    self.service_form = ServiceForm {
                        name: String::new(),
                    };
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char(c) => {
                    self.service_form.name.push(c);
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Backspace => {
                    self.service_form.name.pop();
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if !self.service_form.name.is_empty() {
                        if self.is_editing_service && self.focused_service_index < self.services.len() {
                            self.services[self.focused_service_index].name = self.service_form.name.clone();
                        } else {
                            let new_service = Service {
                                id: self.generate_service_id(),
                                name: self.service_form.name.clone(),
                            };
                            self.services.push(new_service);
                        }
                        
                        self.save_services();
                        self.show_service_popup = false;
                        self.is_editing_service = false;
                        self.service_form = ServiceForm {
                            name: String::new(),
                        };
                    }
                    Ok(ScreenOutcome::Continue)
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_gastos_panel {
            match key.code {
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                    self.show_gastos_panel = false;
                    self.show_options_menu = true;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F') => {
                    self.show_gastos_fijos = true;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('v') | crossterm::event::KeyCode::Char('V') => {
                    self.show_gastos_fijos = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('t') | crossterm::event::KeyCode::Char('T') => {
                    if self.show_gastos_fijos {
                        self.show_template_menu = !self.show_template_menu;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if self.show_template_menu && self.selected_template_option < 2 {
                        self.selected_template_option += 1;
                    } else if self.show_gastos_fijos && !self.show_template_menu && !self.fixed_bills.is_empty() && self.focused_bill_index < self.fixed_bills.len() - 1 {
                        self.focused_bill_index += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.show_template_menu && self.selected_template_option > 0 {
                        self.selected_template_option -= 1;
                    } else if self.show_gastos_fijos && !self.show_template_menu && self.focused_bill_index > 0 {
                        self.focused_bill_index -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('b') | crossterm::event::KeyCode::Char('B') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_services_panel {
            match key.code {
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                    self.show_services_panel = false;
                    self.show_options_menu = true;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => {
                    self.show_service_popup = true;
                    self.is_editing_service = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E') => {
                    if !self.services.is_empty() && self.focused_service_index < self.services.len() {
                        let service = &self.services[self.focused_service_index];
                        self.service_form = ServiceForm {
                            name: service.name.clone(),
                        };
                        self.show_service_popup = true;
                        self.is_editing_service = true;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                    if !self.services.is_empty() && self.focused_service_index < self.services.len() {
                        self.services.remove(self.focused_service_index);
                        if self.focused_service_index >= self.services.len() && !self.services.is_empty() {
                            self.focused_service_index = self.services.len() - 1;
                        }
                        self.save_services();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if !self.services.is_empty() && self.focused_service_index < self.services.len() - 1 {
                        self.focused_service_index += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.focused_service_index > 0 {
                        self.focused_service_index -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('b') | crossterm::event::KeyCode::Char('B') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_categories_panel {
            match key.code {
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                    self.show_categories_panel = false;
                    self.show_options_menu = true;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => {
                    self.show_category_popup = true;
                    self.is_editing_category = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E') => {
                    if !self.categories.is_empty() && self.focused_category_index < self.categories.len() {
                        let category = &self.categories[self.focused_category_index];
                        self.category_form = CategoryForm {
                            name: category.name.clone(),
                        };
                        self.show_category_popup = true;
                        self.is_editing_category = true;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                    if !self.categories.is_empty() && self.focused_category_index < self.categories.len() {
                        self.categories.remove(self.focused_category_index);
                        if self.focused_category_index >= self.categories.len() && !self.categories.is_empty() {
                            self.focused_category_index = self.categories.len() - 1;
                        }
                        self.save_categories();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if !self.categories.is_empty() && self.focused_category_index < self.categories.len() - 1 {
                        self.focused_category_index += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.focused_category_index > 0 {
                        self.focused_category_index -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('b') | crossterm::event::KeyCode::Char('B') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else if self.show_properties_panel {
            match key.code {
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                    self.show_properties_panel = false;
                    self.show_options_menu = true;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => {
                    self.show_property_popup = true;
                    self.is_editing_property = false;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E') => {
                    if !self.properties.is_empty() && self.focused_property_index < self.properties.len() {
                        let property = &self.properties[self.focused_property_index];
                        self.property_form = PropertyForm {
                            street_name: property.street_name.clone(),
                            street_number: property.street_number.clone(),
                            city: property.city.clone(),
                            state: property.state.clone(),
                        };
                        self.show_property_popup = true;
                        self.is_editing_property = true;
                        self.current_field = 0;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                    if !self.properties.is_empty() && self.focused_property_index < self.properties.len() {
                        self.properties.remove(self.focused_property_index);
                        if self.focused_property_index >= self.properties.len() && !self.properties.is_empty() {
                            self.focused_property_index = self.properties.len() - 1;
                        }
                        self.save_properties();
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if !self.properties.is_empty() && self.focused_property_index < self.properties.len() - 1 {
                        self.focused_property_index += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.focused_property_index > 0 {
                        self.focused_property_index -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('b') | crossterm::event::KeyCode::Char('B') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                _ => Ok(ScreenOutcome::Continue),
            }
        } else {
            match key.code {
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                    Ok(ScreenOutcome::Quit)
                }
                crossterm::event::KeyCode::Char('b') | crossterm::event::KeyCode::Char('B') => {
                    Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home))
                }
                crossterm::event::KeyCode::Char('o') | crossterm::event::KeyCode::Char('O') => {
                    self.show_options_menu = !self.show_options_menu;
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('j') => {
                    if self.show_options_menu && self.selected_option < 4 {
                        self.selected_option += 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Char('k') => {
                    if self.show_options_menu && self.selected_option > 0 {
                        self.selected_option -= 1;
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Enter => {
                    if self.show_options_menu {
                        match self.selected_option {
                            0 => { // Gastos
                                self.show_options_menu = false;
                                self.show_gastos_panel = true;
                                self.focused_bill_index = 0;
                            }
                            1 => { // Categorias
                                self.show_options_menu = false;
                                self.show_categories_panel = true;
                                self.focused_category_index = 0;
                            }
                            2 => { // Inmuebles
                                self.show_options_menu = false;
                                self.show_properties_panel = true;
                                self.focused_property_index = 0;
                            }
                            3 => { // Servicios
                                self.show_options_menu = false;
                                self.show_services_panel = true;
                                self.focused_service_index = 0;
                            }
                            _ => {}
                        }
                    }
                    Ok(ScreenOutcome::Continue)
                }
                crossterm::event::KeyCode::Esc => {
                    if self.show_options_menu {
                        self.show_options_menu = false;
                    } else {
                        return Ok(ScreenOutcome::ChangeState(crate::presentation::tui::AppState::Home));
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
            .constraints([
                Constraint::Min(0),
                if self.show_options_menu { Constraint::Length(8) } 
                else if self.show_properties_panel || self.show_categories_panel || self.show_services_panel || (self.show_gastos_panel && !self.show_template_menu) { Constraint::Length(3) }
                else if self.show_gastos_panel && self.show_template_menu { Constraint::Length(6) }
                else { Constraint::Length(0) },
                Constraint::Length(3),
            ])
            .split(size);

        let content_area = main_layout[0];
        let options_area = main_layout[1];
        let menu_area = main_layout[2];

        // Main content
        if self.show_gastos_panel {
            // Gastos panel
            let gastos_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .title(Span::styled(" 💰 Gastos ", Style::default().fg(Color::Blue)));

            if self.show_gastos_fijos {
                if self.fixed_bills.is_empty() {
                    let content = Paragraph::new("No hay gastos fijos cargados")
                        .alignment(Alignment::Center)
                        .block(gastos_block);
                    f.render_widget(content, content_area);
                } else {
                    let mut lines = Vec::new();
                    for (i, bill) in self.fixed_bills.iter().enumerate() {
                        let style = if i == self.focused_bill_index {
                            Style::default().bg(Color::Yellow).fg(Color::Black)
                        } else {
                            Style::default()
                        };
                        let property_name = self.get_property_name(&bill.property_id);
                        let service_name = self.get_service_name(&bill.service_id);
                        let text = format!("{} - {} | Vence: {} | ${}",
                            property_name,
                            service_name,
                            bill.dia_vencimiento,
                            bill.monto
                        );
                        lines.push(Line::from(Span::styled(text, style)));
                    }
                    
                    let gastos_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                        .alignment(Alignment::Left)
                        .block(gastos_block);
                    f.render_widget(gastos_paragraph, content_area);
                }
            } else {
                let content = Paragraph::new("Gastos Variables - En desarrollo")
                    .alignment(Alignment::Center)
                    .block(gastos_block);
                f.render_widget(content, content_area);
            }
        } else if self.show_properties_panel {
            // Properties panel
            let properties_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .title(Span::styled(" 🏠 Inmuebles ", Style::default().fg(Color::Blue)));

            if self.properties.is_empty() {
                let content = Paragraph::new("No hay inmuebles cargados")
                    .alignment(Alignment::Center)
                    .block(properties_block);
                f.render_widget(content, content_area);
            } else {
                let mut lines = Vec::new();
                for (i, property) in self.properties.iter().enumerate() {
                    let style = if i == self.focused_property_index {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };
                    let text = format!("{} {} - {}, {}", 
                        property.street_name, 
                        property.street_number, 
                        property.city, 
                        property.state);
                    lines.push(Line::from(Span::styled(text, style)));
                }
                
                let properties_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                    .alignment(Alignment::Left)
                    .block(properties_block);
                f.render_widget(properties_paragraph, content_area);
            }
        } else if self.show_categories_panel {
            // Categories panel
            let categories_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .title(Span::styled(" 📂 Categorias ", Style::default().fg(Color::Blue)));

            if self.categories.is_empty() {
                let content = Paragraph::new("No hay categorias cargadas")
                    .alignment(Alignment::Center)
                    .block(categories_block);
                f.render_widget(content, content_area);
            } else {
                let mut lines = Vec::new();
                for (i, category) in self.categories.iter().enumerate() {
                    let style = if i == self.focused_category_index {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };
                    lines.push(Line::from(Span::styled(category.name.clone(), style)));
                }
                
                let categories_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                    .alignment(Alignment::Left)
                    .block(categories_block);
                f.render_widget(categories_paragraph, content_area);
            }
        } else if self.show_services_panel {
            // Services panel
            let services_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .title(Span::styled(" 🔧 Servicios ", Style::default().fg(Color::Blue)));

            if self.services.is_empty() {
                let content = Paragraph::new("No hay servicios cargados")
                    .alignment(Alignment::Center)
                    .block(services_block);
                f.render_widget(content, content_area);
            } else {
                let mut lines = Vec::new();
                for (i, service) in self.services.iter().enumerate() {
                    let style = if i == self.focused_service_index {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };
                    lines.push(Line::from(Span::styled(service.name.clone(), style)));
                }
                
                let services_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                    .alignment(Alignment::Left)
                    .block(services_block);
                f.render_widget(services_paragraph, content_area);
            }
        } else {
            // Default content
            let content_block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .title(Span::styled(" 💳 Bills ", Style::default().fg(Color::Blue)));

            let content = Paragraph::new("Bills management coming soon...")
                .alignment(Alignment::Center)
                .block(content_block);
            f.render_widget(content, content_area);
        }

        // Options menu
        if self.show_options_menu {
            let options_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(" ☰ Opciones ", Style::default().fg(Color::Cyan)));

            let options = vec!["Gastos", "Categorias", "Inmuebles", "Servicios", "Historico"];
            let mut lines = Vec::new();

            for (i, option) in options.iter().enumerate() {
                let style = if i == self.selected_option {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                lines.push(Line::from(Span::styled(format!(" {} ", option), style)));
            }

            let options_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                .alignment(Alignment::Left)
                .block(options_block);
            f.render_widget(options_paragraph, options_area);
        }

        // Gastos submenu
        if self.show_gastos_panel {
            if self.show_template_menu {
                let template_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(Span::styled(" Template ", Style::default().fg(Color::Cyan)));

                let template_options = vec!["Add", "Edit", "Delete"];
                let mut lines = Vec::new();

                for (i, option) in template_options.iter().enumerate() {
                    let style = if i == self.selected_template_option {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };
                    lines.push(Line::from(Span::styled(format!(" {} ", option), style)));
                }

                let template_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                    .alignment(Alignment::Left)
                    .block(template_block);
                f.render_widget(template_paragraph, options_area);
            } else {
                let submenu_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(Span::styled(" Acciones ", Style::default().fg(Color::Cyan)));

                let submenu_text = if self.show_gastos_fijos {
                    "C: Clear | F: Fijos | V: Variables | T: Template"
                } else {
                    "C: Clear | F: Fijos | V: Variables"
                };

                let submenu_paragraph = Paragraph::new(submenu_text)
                    .alignment(Alignment::Center)
                    .block(submenu_block);
                f.render_widget(submenu_paragraph, options_area);
            }
        }

        // Properties submenu
        if self.show_properties_panel {
            let submenu_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(" Acciones ", Style::default().fg(Color::Cyan)));

            let submenu_paragraph = Paragraph::new("C: Clear | A: Add | E: Edit | D: Delete")
                .alignment(Alignment::Center)
                .block(submenu_block);
            f.render_widget(submenu_paragraph, options_area);
        }

        // Categories submenu
        if self.show_categories_panel {
            let submenu_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(" Acciones ", Style::default().fg(Color::Cyan)));

            let submenu_paragraph = Paragraph::new("C: Clear | A: Add | E: Edit | D: Delete")
                .alignment(Alignment::Center)
                .block(submenu_block);
            f.render_widget(submenu_paragraph, options_area);
        }

        // Services submenu
        if self.show_services_panel {
            let submenu_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(" Acciones ", Style::default().fg(Color::Cyan)));

            let submenu_paragraph = Paragraph::new("C: Clear | A: Add | E: Edit | D: Delete")
                .alignment(Alignment::Center)
                .block(submenu_block);
            f.render_widget(submenu_paragraph, options_area);
        }

        // Menu
        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" ⚙ Menu ", Style::default().fg(Color::Green)));

        let menu_text = if self.show_gastos_panel && self.show_template_menu {
            "T: Hide Template | J/K: Navigate | Enter: Select | B: Back | Q: Quit"
        } else if self.show_properties_panel || self.show_categories_panel || self.show_services_panel || self.show_gastos_panel {
            "J/K: Navigate | B: Back | Q: Quit"
        } else if self.show_options_menu {
            "O: Hide Options | J/K: Navigate | Enter: Select | B: Back | Q: Quit"
        } else {
            "O: Show Options | B: Back | Q: Quit"
        };

        let menu_paragraph = Paragraph::new(menu_text)
            .alignment(Alignment::Center)
            .block(menu_block);
        f.render_widget(menu_paragraph, menu_area);

        // Property popup
        if self.show_property_popup {
            let popup_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Length(12),
                    Constraint::Percentage(25),
                ])
                .split(size)[1];

            let popup_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(popup_area)[1];

            f.render_widget(Clear, popup_area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(if self.is_editing_property { " Editar Inmueble " } else { " Agregar Inmueble " });

            let fields = [
                ("Calle:", &self.property_form.street_name),
                ("Número:", &self.property_form.street_number),
                ("Ciudad:", &self.property_form.city),
                ("Estado:", &self.property_form.state),
            ];

            let mut lines = Vec::new();
            for (i, (label, value)) in fields.iter().enumerate() {
                let style = if i == self.current_field {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default()
                };
                lines.push(Line::from(vec![
                    Span::raw(format!("{:<10}", label)),
                    Span::styled(format!("{:<20}", value), style),
                ]));
                lines.push(Line::from(""));
            }

            let popup_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                .block(popup_block);
            f.render_widget(popup_paragraph, popup_area);
        }

        // Category popup
        if self.show_category_popup {
            let popup_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35),
                    Constraint::Length(6),
                    Constraint::Percentage(35),
                ])
                .split(size)[1];

            let popup_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(popup_area)[1];

            f.render_widget(Clear, popup_area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(if self.is_editing_category { " Editar Categoria " } else { " Agregar Categoria " });

            let lines = vec![
                Line::from(vec![
                    Span::raw("Nombre: "),
                    Span::styled(&self.category_form.name, Style::default().bg(Color::Yellow).fg(Color::Black)),
                ]),
                Line::from(""),
                Line::from("Enter: Guardar | Esc: Cancelar"),
            ];

            let popup_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                .block(popup_block);
            f.render_widget(popup_paragraph, popup_area);
        }

        // Service popup
        if self.show_service_popup {
            let popup_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35),
                    Constraint::Length(6),
                    Constraint::Percentage(35),
                ])
                .split(size)[1];

            let popup_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(popup_area)[1];

            f.render_widget(Clear, popup_area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(if self.is_editing_service { " Editar Servicio " } else { " Agregar Servicio " });

            let lines = vec![
                Line::from(vec![
                    Span::raw("Nombre: "),
                    Span::styled(&self.service_form.name, Style::default().bg(Color::Yellow).fg(Color::Black)),
                ]),
                Line::from(""),
                Line::from("Enter: Guardar | Esc: Cancelar"),
            ];

            let popup_paragraph = Paragraph::new(ratatui::text::Text::from(lines))
                .block(popup_block);
            f.render_widget(popup_paragraph, popup_area);
        }
    }
}
