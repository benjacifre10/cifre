// src/presentation/screens/releases_screen.rs
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::screen::{Screen, ScreenContext, ScreenOutcome};
use crate::AppState;
use crate::domain::models::{Country, Stage, Artifact, ArtifactType, Release, ReleaseArtifact};
use crate::infrastructure::persistence::{CountryJsonRepository, StageJsonRepository, ArtifactJsonRepository, ArtifactTypeJsonRepository, ReleaseJsonRepository};
use crate::presentation::components::Notification;

#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    Options,
    ManageReleases,
    EditRelease(String), // ID del release a editar
    ManageCountries,
    EditCountry(String), // ID del país a editar
    ManageStages,
    EditStage(String), // ID del stage a editar
    ManageArtifacts,
    EditArtifact(String), // ID del artifact a editar
    AddReleaseArtifact, // Popup para agregar artifact al release
    SearchArtifacts(String), // Popup de búsqueda con texto ingresado
    SelectCountry, // Popup para seleccionar país
    SelectStage, // Popup para seleccionar stage
    EnterVersion, // Popup para ingresar versión
    EditArtifactStage, // Popup para editar stage de artifact
    EditArtifactVersion, // Popup para editar version de artifact
    MoveArtifact, // Popup para mover/intercambiar orden de artifacts
    PrintArtifacts, // Popup para seleccionar formato de impresión (PNG/PDF)
}

pub struct ReleasesScreen {
    popup_state: PopupState,
    selected_option: usize,
    options: Vec<&'static str>,
    countries: Vec<Country>,
    selected_country: usize,
    country_repo: CountryJsonRepository,
    stages: Vec<Stage>,
    selected_stage: usize,
    stage_repo: StageJsonRepository,
    selected_country_for_stage: usize,
    artifacts: Vec<Artifact>,
    selected_artifact: usize,
    artifact_repo: ArtifactJsonRepository,
    artifact_types: Vec<ArtifactType>,
    artifact_type_repo: ArtifactTypeJsonRepository,
    selected_artifact_type: usize,
    releases: Vec<Release>,
    selected_release: usize,
    release_repo: ReleaseJsonRepository,
    edit_year: String,
    edit_date_init: String,
    edit_date_qa: String,
    edit_date_finish: String,
    edit_field: usize, // 0: name, 1: year, 2: date_init, 3: date_qa, 4: date_finish
    edit_text: String,
    notification: Option<Notification>,
    list_focused: bool,
    selected_release_in_list: usize,
    submenu_focused: bool,
    release_focused: bool,
    selected_release_for_info: Option<Release>,
    search_text: String,
    filtered_artifacts: Vec<Artifact>,
    selected_filtered_artifact: usize,
    selected_country_for_artifact: usize,
    selected_stage_for_artifact: usize,
    temp_artifact_id: Option<String>,
    temp_country_id: Option<String>,
    deploy_focused: bool,
    selected_deploy_artifact: usize,
    deploy_scroll_offset: usize, // offset de scroll para artifacts en deploy
    version_text: String,
    selected_stage_for_edit: usize,
    last_notified_release: Option<String>, // Para evitar notificaciones repetidas
    selected_move_artifact: usize, // Índice seleccionado en el popup de mover
    source_artifact_index: usize, // Índice del artefacto que se está moviendo
    selected_print_format: usize, // 0: PNG, 1: PDF
}

impl ReleasesScreen {
    pub fn new() -> Self {
        let mut screen = Self {
            popup_state: PopupState::None,
            selected_option: 0,
            options: vec![
                "Manage Releases",
                "Manage Artifacts", 
                "Manage Stages",
                "Manage Countries",
            ],
            countries: Vec::new(),
            selected_country: 0,
            country_repo: CountryJsonRepository::new(),
            stages: Vec::new(),
            selected_stage: 0,
            stage_repo: StageJsonRepository::new(),
            selected_country_for_stage: 0,
            artifacts: Vec::new(),
            selected_artifact: 0,
            artifact_repo: ArtifactJsonRepository::new(),
            artifact_types: Vec::new(),
            artifact_type_repo: ArtifactTypeJsonRepository::new(),
            selected_artifact_type: 0,
            releases: Vec::new(),
            selected_release: 0,
            release_repo: ReleaseJsonRepository::new(),
            edit_year: String::new(),
            edit_date_init: String::new(),
            edit_date_qa: String::new(),
            edit_date_finish: String::new(),
            edit_field: 0,
            edit_text: String::new(),
            notification: None,
            list_focused: false,
            selected_release_in_list: 0,
            submenu_focused: true,
            release_focused: false,
            selected_release_for_info: None,
            search_text: String::new(),
            filtered_artifacts: Vec::new(),
            selected_filtered_artifact: 0,
            selected_country_for_artifact: 0,
            selected_stage_for_artifact: 0,
            temp_artifact_id: None,
            temp_country_id: None,
            deploy_focused: false,
            selected_deploy_artifact: 0,
            deploy_scroll_offset: 0,
            version_text: String::new(),
            selected_stage_for_edit: 0,
            last_notified_release: None,
            selected_move_artifact: 0,
            source_artifact_index: 0,
            selected_print_format: 0,
        };
        
        // Cargar releases al inicializar
        if let Err(e) = screen.load_releases() {
            eprintln!("Error loading releases: {}", e);
        }
        
        screen
    }

    fn load_countries(&mut self) -> Result<()> {
        self.countries = self.country_repo.get_all_countries()?;
        if self.selected_country >= self.countries.len() && !self.countries.is_empty() {
            self.selected_country = self.countries.len() - 1;
        }
        Ok(())
    }

    fn load_stages(&mut self) -> Result<()> {
        self.stages = self.stage_repo.get_all_stages()?;
        if self.selected_stage >= self.stages.len() && !self.stages.is_empty() {
            self.selected_stage = self.stages.len() - 1;
        }
        Ok(())
    }

    fn load_artifacts(&mut self) -> Result<()> {
        self.artifacts = self.artifact_repo.get_all_artifacts()?;
        if self.selected_artifact >= self.artifacts.len() && !self.artifacts.is_empty() {
            self.selected_artifact = self.artifacts.len() - 1;
        }
        Ok(())
    }

    fn load_artifact_types(&mut self) -> Result<()> {
        self.artifact_types = self.artifact_type_repo.get_all_artifact_types()?;
        Ok(())
    }

    fn load_releases(&mut self) -> Result<()> {
        self.releases = self.release_repo.get_all_releases()?;
        
        // Ordenar por date_init de más nuevo a más viejo
        self.releases.sort_by(|a, b| b.date_init.cmp(&a.date_init));
        
        if self.selected_release >= self.releases.len() && !self.releases.is_empty() {
            self.selected_release = self.releases.len() - 1;
        }
        Ok(())
    }

    fn draw_options_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 30;
        let popup_height = self.options.len() as u16 + 2;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                if i == self.selected_option {
                    ListItem::new(*option).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(*option)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(" Options ", Style::default().fg(Color::Yellow)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);
    }

    fn draw_countries_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 50;
        let popup_height = (self.countries.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.countries
            .iter()
            .enumerate()
            .map(|(i, country)| {
                let text = format!("{} ({})", country.name, &country.id[..8]);
                if i == self.selected_country {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Manage Countries ", Style::default().fg(Color::Green)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);

        // Mostrar controles en la parte inferior del popup
        let controls_text = "A: Add | E: Edit | D: Delete | Esc: Back";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_edit_popup(&self, f: &mut Frame, area: ratatui::layout::Rect, is_new: bool) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 40;
        let popup_height = 5;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let title = if is_new { " Add Country " } else { " Edit Country " };
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(title, Style::default().fg(Color::Cyan)));

        let input_text = format!("Name: {}", self.edit_text);
        let paragraph = Paragraph::new(input_text).block(popup_block);
        f.render_widget(paragraph, popup_area);

        // Mostrar controles
        let controls_text = "Enter: Save | Esc: Cancel";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_stages_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 60;
        let popup_height = (self.stages.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.stages
            .iter()
            .enumerate()
            .map(|(i, stage)| {
                let country_name = self.countries.iter()
                    .find(|c| c.id == stage.country_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                let text = format!("{} - {} ({})", stage.name, country_name, &stage.id[..8]);
                if i == self.selected_stage {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Manage Stages ", Style::default().fg(Color::Green)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);

        let controls_text = "A: Add | E: Edit | D: Delete | Esc: Back";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_edit_stage_popup(&self, f: &mut Frame, area: ratatui::layout::Rect, is_new: bool) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 50;
        let popup_height = 7;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let title = if is_new { " Add Stage " } else { " Edit Stage " };
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(title, Style::default().fg(Color::Cyan)));

        let selected_country_name = if !self.countries.is_empty() && self.selected_country_for_stage < self.countries.len() {
            &self.countries[self.selected_country_for_stage].name
        } else {
            "No countries available"
        };

        let content = format!("Name: {}\nCountry: {}\n\nTab: Change Country", 
                             self.edit_text, selected_country_name);
        let paragraph = Paragraph::new(content).block(popup_block);
        f.render_widget(paragraph, popup_area);

        let controls_text = "Enter: Save | Tab: Change Country | Esc: Cancel";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_notification(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        if let Some(ref notification) = self.notification {
            if notification.is_expired() {
                self.notification = None;
            } else {
                notification.draw(f, area);
            }
        }
    }

    fn draw_artifacts_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 60;
        let popup_height = (self.artifacts.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.artifacts
            .iter()
            .enumerate()
            .map(|(i, artifact)| {
                let type_name = self.artifact_types.iter()
                    .find(|t| t.id == artifact.artifact_type_id)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown");
                let text = format!("{} - {} ({})", artifact.name, type_name, &artifact.id[..8]);
                if i == self.selected_artifact {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Manage Artifacts ", Style::default().fg(Color::Green)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);

        let controls_text = "A: Add | E: Edit | D: Delete | Esc: Back";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_edit_artifact_popup(&self, f: &mut Frame, area: ratatui::layout::Rect, is_new: bool) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 50;
        let popup_height = 7;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let title = if is_new { " Add Artifact " } else { " Edit Artifact " };
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(title, Style::default().fg(Color::Cyan)));

        let selected_type_name = if !self.artifact_types.is_empty() && self.selected_artifact_type < self.artifact_types.len() {
            &self.artifact_types[self.selected_artifact_type].name
        } else {
            "No types available"
        };

        let content = format!("Name: {}\nType: {}\n\nTab: Change Type", 
                             self.edit_text, selected_type_name);
        let paragraph = Paragraph::new(content).block(popup_block);
        f.render_widget(paragraph, popup_area);

        let controls_text = "Enter: Save | Tab: Change Type | Esc: Cancel";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_releases_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 80;
        let popup_height = (self.releases.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.releases
            .iter()
            .enumerate()
            .map(|(i, release)| {
                let text = format!("{} ({}) - Init: {} QA: {} Finish: {} ({})", 
                    release.name, release.year, release.date_init, release.date_qa, release.date_finish, &release.id[..8]);
                if i == self.selected_release {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Manage Releases ", Style::default().fg(Color::Green)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);

        let controls_text = "A: Add | E: Edit | D: Delete | Esc: Back";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_edit_release_popup(&self, f: &mut Frame, area: ratatui::layout::Rect, is_new: bool) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 60;
        let popup_height = 10;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let title = if is_new { " Add Release " } else { " Edit Release " };
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(title, Style::default().fg(Color::Cyan)));

        let field_names = ["Name", "Year (yyyy)", "Date Init (yyyy-mm-dd)", "Date QA (yyyy-mm-dd)", "Date Finish (yyyy-mm-dd)"];
        let field_values = [&self.edit_text, &self.edit_year, &self.edit_date_init, &self.edit_date_qa, &self.edit_date_finish];
        
        let mut content = String::new();
        for (i, (name, value)) in field_names.iter().zip(field_values.iter()).enumerate() {
            if i == self.edit_field {
                content.push_str(&format!("> {}: {}\n", name, value));
            } else {
                content.push_str(&format!("  {}: {}\n", name, value));
            }
        }
        content.push_str("\nTab: Next Field");

        let paragraph = Paragraph::new(content).block(popup_block);
        f.render_widget(paragraph, popup_area);

        let controls_text = "Enter: Save | Tab: Next Field | Esc: Cancel";
        let controls_y = popup_area.y + popup_area.height;
        if controls_y < area.height {
            let controls_area = ratatui::layout::Rect {
                x: popup_area.x,
                y: controls_y,
                width: popup_area.width,
                height: 1,
            };
            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(controls, controls_area);
        }
    }

    fn draw_add_artifact_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 50;
        let popup_height = 5;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(" Add Artifact ", Style::default().fg(Color::Cyan)));

        let content = format!("Search (min 3 chars): {}", self.search_text);
        let paragraph = Paragraph::new(content).block(popup_block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_search_artifacts_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 60;
        let popup_height = (self.filtered_artifacts.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.filtered_artifacts
            .iter()
            .enumerate()
            .map(|(i, artifact)| {
                let text = format!("{} ({})", artifact.name, &artifact.id[..8]);
                if i == self.selected_filtered_artifact {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Select Artifact ", Style::default().fg(Color::Green)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);
    }

    fn draw_select_country_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 50;
        let popup_height = (self.countries.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.countries
            .iter()
            .enumerate()
            .map(|(i, country)| {
                let text = format!("{} ({})", country.name, &country.id[..8]);
                if i == self.selected_country_for_artifact {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(" Select Country ", Style::default().fg(Color::Yellow)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);
    }

    fn draw_select_stage_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 60;
        let popup_height = (self.stages.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.stages
            .iter()
            .enumerate()
            .map(|(i, stage)| {
                let country_name = self.countries.iter()
                    .find(|c| c.id == stage.country_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                let text = format!("{} - {} ({})", stage.name, country_name, &stage.id[..8]);
                if i == self.selected_stage_for_artifact {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(Span::styled(" Select Stage ", Style::default().fg(Color::Magenta)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);
    }

    fn draw_enter_version_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 40;
        let popup_height = 5;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };
        
        f.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Enter Version ", Style::default().fg(Color::Green)));

        let content = format!("Version: {}", self.version_text);
        let paragraph = Paragraph::new(content).block(popup_block);
        f.render_widget(paragraph, popup_area);
    }

    fn draw_edit_artifact_stage_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let popup_width = 60;
        let popup_height = (self.stages.len() as u16).max(5) + 4;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self.stages
            .iter()
            .enumerate()
            .map(|(i, stage)| {
                let country_name = self.countries.iter()
                    .find(|c| c.id == stage.country_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                let text = format!("{} - {} ({})", stage.name, country_name, &stage.id[..8]);
                if i == self.selected_stage_for_edit {
                    ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(text)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(" Edit Artifact Stage ", Style::default().fg(Color::Red)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);
    }

    fn draw_edit_artifact_version_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, Paragraph};
        
        let popup_width = 40;
        let popup_height = 5;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };
        
        f.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(" Edit Version ", Style::default().fg(Color::Cyan)));

        let content = format!("Version: {}", self.version_text);
        let paragraph = Paragraph::new(content).block(popup_block);
        f.render_widget(paragraph, popup_area);
    }

    fn check_release_date_notification(&mut self, release_name: &str, date_finish: &str) {
        use chrono::{NaiveDate, Local};
        
        // Solo mostrar notificación si no hay una activa y no se ha notificado este release
        if self.notification.is_some() || 
           self.last_notified_release.as_ref() == Some(&release_name.to_string()) {
            return;
        }
        
        // Parsear la fecha de finalización
        if let Ok(finish_date) = NaiveDate::parse_from_str(date_finish, "%Y-%m-%d") {
            let today = Local::now().date_naive();
            
            if finish_date < today {
                // Fecha ya pasó - Warning
                self.notification = Some(Notification::warning(
                    format!("Release '{}' finalizó el {}", release_name, date_finish)
                ));
            } else {
                // Fecha futura - Info
                self.notification = Some(Notification::info(
                    format!("Release '{}' finaliza el {}", release_name, date_finish)
                ));
            }
            
            // Marcar este release como notificado
            self.last_notified_release = Some(release_name.to_string());
        }
    }

    fn get_order_circle(order: usize) -> &'static str {
        match order {
            1 => "①", 2 => "②", 3 => "③", 4 => "④", 5 => "⑤",
            6 => "⑥", 7 => "⑦", 8 => "⑧", 9 => "⑨", 10 => "⑩",
            11 => "⑪", 12 => "⑫", 13 => "⑬", 14 => "⑭", 15 => "⑮",
            16 => "⑯", 17 => "⑰", 18 => "⑱", 19 => "⑲", 20 => "⑳",
            _ => "㊿",
        }
    }

    fn draw_move_artifact_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        if let Some(ref release) = self.selected_release_for_info {
            let mut sorted_artifacts: Vec<_> = release.artifacts.iter().enumerate().collect();
            sorted_artifacts.sort_by_key(|(_, a)| a.order);
            
            let filtered: Vec<_> = sorted_artifacts.iter()
                .filter(|(idx, _)| *idx != self.source_artifact_index)
                .collect();
            
            let popup_width = 60;
            let popup_height = (filtered.len() as u16).max(5) + 4;
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;
            
            let popup_area = ratatui::layout::Rect {
                x: popup_x,
                y: popup_y,
                width: popup_width,
                height: popup_height,
            };

            f.render_widget(Clear, popup_area);

            let items: Vec<ListItem> = filtered
                .iter()
                .enumerate()
                .map(|(i, (_, artifact))| {
                    let artifact_name = self.artifacts.iter()
                        .find(|a| a.id == artifact.artifact_id)
                        .map(|a| a.name.as_str())
                        .unwrap_or("Unknown");
                    let order_symbol = Self::get_order_circle(artifact.order);
                    let text = format!("{} {}", order_symbol, artifact_name);
                    if i == self.selected_move_artifact {
                        ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                    } else {
                        ListItem::new(text)
                    }
                })
                .collect();

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(Span::styled(" Move Artifact - Select Target ", Style::default().fg(Color::Magenta)));

            let list = List::new(items).block(popup_block);
            f.render_widget(list, popup_area);
        }
    }

    fn draw_print_artifacts_popup(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Clear, List, ListItem};
        
        let formats = vec!["PNG", "PDF"];
        
        let popup_width = 30;
        let popup_height = 6;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        
        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = formats
            .iter()
            .enumerate()
            .map(|(i, format)| {
                if i == self.selected_print_format {
                    ListItem::new(*format).style(Style::default().bg(Color::Blue).fg(Color::White))
                } else {
                    ListItem::new(*format)
                }
            })
            .collect();

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(" Export Format ", Style::default().fg(Color::Green)));

        let list = List::new(items).block(popup_block);
        f.render_widget(list, popup_area);
    }

    fn export_release_artifacts(&self, release: &Release, format: &str) -> Result<String> {
        use std::env;
        use std::path::PathBuf;
        
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        let downloads_dir = PathBuf::from(home_dir).join("Downloads");
        
        let filename = format!("{}-{}.{}", release.year, release.name.replace(" ", "_"), format);
        let filepath = downloads_dir.join(&filename);
        
        match format {
            "png" => self.export_to_png(release, &filepath)?,
            "pdf" => self.export_to_pdf(release, &filepath)?,
            _ => return Err(anyhow::anyhow!("Formato no soportado")),
        }
        
        Ok(filepath.to_string_lossy().to_string())
    }

    fn export_to_png(&self, release: &Release, filepath: &std::path::Path) -> Result<()> {
        use image::{Rgb, RgbImage};
        use imageproc::drawing::{draw_text_mut, draw_hollow_rect_mut};
        use imageproc::rect::Rect as ImgRect;
        use rusttype::{Font, Scale};
        
        let mut sorted_artifacts: Vec<_> = release.artifacts.iter().collect();
        sorted_artifacts.sort_by_key(|a| a.order);
        
        let artifact_height = 80;
        let width = 800;
        let height = 100 + (sorted_artifacts.len() * artifact_height);
        
        let mut img = RgbImage::from_pixel(width as u32, height as u32, Rgb([255, 255, 255]));
        
        let font_data = include_bytes!("/System/Library/Fonts/Helvetica.ttc");
        let font = Font::try_from_bytes(font_data as &[u8]).ok_or_else(|| anyhow::anyhow!("Error loading font"))?;
        
        let title = format!("Release: {} ({})", release.name, release.year);
        draw_text_mut(&mut img, Rgb([0, 0, 0]), 20, 20, Scale::uniform(24.0), &font, &title);
        
        for (i, artifact) in sorted_artifacts.iter().enumerate() {
            let y = 80 + (i * artifact_height);
            
            let artifact_name = self.artifacts.iter()
                .find(|a| a.id == artifact.artifact_id)
                .map(|a| a.name.as_str())
                .unwrap_or("Unknown");
            let artifact_type = self.artifacts.iter()
                .find(|a| a.id == artifact.artifact_id)
                .and_then(|a| self.artifact_types.iter().find(|t| t.id == a.artifact_type_id))
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown");
            let country_name = self.countries.iter()
                .find(|c| c.id == artifact.country_id)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            let stage_name = self.stages.iter()
                .find(|s| s.id == artifact.stage_id)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown");
            
            draw_hollow_rect_mut(&mut img, ImgRect::at(10, y as i32).of_size(780, 70), Rgb([128, 128, 128]));
            
            let order_text = format!("{}", artifact.order);
            let country_code = country_name.chars().take(3).collect::<String>().to_uppercase();
            draw_text_mut(&mut img, Rgb([0, 0, 0]), 20, y as i32 + 10, Scale::uniform(16.0), &font, &country_code);
            draw_text_mut(&mut img, Rgb([0, 0, 0]), 30, y as i32 + 35, Scale::uniform(14.0), &font, &order_text);
            
            let artifact_text = format!("{} - {} v{}", artifact_name, artifact_type, artifact.version);
            draw_text_mut(&mut img, Rgb([0, 0, 0]), 150, y as i32 + 20, Scale::uniform(18.0), &font, &artifact_text);
            
            draw_text_mut(&mut img, Rgb([0, 0, 0]), 650, y as i32 + 20, Scale::uniform(16.0), &font, stage_name);
        }
        
        img.save(filepath)?;
        Ok(())
    }

    fn export_to_pdf(&self, release: &Release, filepath: &std::path::Path) -> Result<()> {
        use printpdf::*;
        use std::fs::File;
        use std::io::BufWriter;
        
        let mut sorted_artifacts: Vec<_> = release.artifacts.iter().collect();
        sorted_artifacts.sort_by_key(|a| a.order);
        
        let (doc, page1, layer1) = PdfDocument::new(&format!("Release: {} ({})", release.name, release.year), Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);
        
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        
        current_layer.use_text(&format!("Release: {} ({})", release.name, release.year), 24.0, Mm(20.0), Mm(270.0), &font_bold);
        
        let mut y_pos = 250.0;
        for artifact in sorted_artifacts.iter() {
            let artifact_name = self.artifacts.iter()
                .find(|a| a.id == artifact.artifact_id)
                .map(|a| a.name.as_str())
                .unwrap_or("Unknown");
            let artifact_type = self.artifacts.iter()
                .find(|a| a.id == artifact.artifact_id)
                .and_then(|a| self.artifact_types.iter().find(|t| t.id == a.artifact_type_id))
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown");
            let country_name = self.countries.iter()
                .find(|c| c.id == artifact.country_id)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            let stage_name = self.stages.iter()
                .find(|s| s.id == artifact.stage_id)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown");
            
            let country_code = country_name.chars().take(3).collect::<String>().to_uppercase();
            let text = format!("[{}] {} - {} - {} v{} - {}", artifact.order, country_code, artifact_name, artifact_type, artifact.version, stage_name);
            
            current_layer.use_text(&text, 12.0, Mm(20.0), Mm(y_pos), &font);
            y_pos -= 10.0;
            
            if y_pos < 20.0 {
                break;
            }
        }
        
        doc.save(&mut BufWriter::new(File::create(filepath)?))?;
        Ok(())
    }

    fn draw_custom_artifacts(&self, f: &mut Frame, area: ratatui::layout::Rect, release: &Release) {
        use ratatui::widgets::{Paragraph, Block, Borders};
        use ratatui::layout::{Layout, Direction, Constraint};
        
        // Ordenar artefactos por order ascendente
        let mut sorted_artifacts: Vec<_> = release.artifacts.iter().collect();
        sorted_artifacts.sort_by_key(|a| a.order);
        
        let available_height = (area.height / 4) as usize; // Más espacio para doble línea
        let visible_artifacts = sorted_artifacts.iter().skip(self.deploy_scroll_offset).take(available_height).enumerate();
        
        for (i, ra) in visible_artifacts {
            if i >= available_height { break; }
            
            let y = area.y + (i * 4) as u16; // Más espacio entre componentes
            let artifact_area = ratatui::layout::Rect {
                x: area.x,
                y,
                width: area.width,
                height: 4, // Altura mayor para acomodar doble línea + contorno
            };
            
            // Obtener datos del artefacto
            let artifact_name = self.artifacts.iter()
                .find(|a| a.id == ra.artifact_id)
                .map(|a| a.name.as_str())
                .unwrap_or("Unknown");
            let artifact_type = self.artifacts.iter()
                .find(|a| a.id == ra.artifact_id)
                .and_then(|a| self.artifact_types.iter().find(|t| t.id == a.artifact_type_id))
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown");
            let country_name = self.countries.iter()
                .find(|c| c.id == ra.country_id)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            let stage_name = self.stages.iter()
                .find(|s| s.id == ra.stage_id)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown");
            
            // Crear el componente personalizado
            let country_code = country_name.chars().take(3).collect::<String>().to_uppercase();
            let stage_upper = stage_name.to_uppercase();
            
            // Color del contorno según selección
            let actual_index = i + self.deploy_scroll_offset;
            let border_color = if actual_index == self.selected_deploy_artifact && self.deploy_focused {
                Color::Blue
            } else {
                Color::Gray
            };
            
            // Crear el bloque con contorno de doble línea (grosor máximo)
            let container_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .border_type(ratatui::widgets::BorderType::Double);
            
            let inner_area = container_block.inner(artifact_area);
            f.render_widget(container_block, artifact_area);
            
            // Layout de tres columnas con separadores dentro del contorno
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(5),  // País + Order
                    Constraint::Length(1),  // Separador 1
                    Constraint::Min(15),    // Artefacto (flexible)
                    Constraint::Length(1),  // Separador 2
                    Constraint::Length(stage_upper.len() as u16 + 2), // Stage
                ])
                .split(inner_area);
            
            // Estilo de texto
            let text_color = Color::Reset;
            
            // Sección 1: País con order debajo en círculo
            let order_circle = Self::get_order_circle(ra.order);
            let country_with_order = format!("{}\n {}", country_code, order_circle);
            let country_text = Paragraph::new(country_with_order)
                .style(Style::default().fg(text_color))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(country_text, columns[0]);
            
            // Separador 1 (doble línea)
            let sep1 = Paragraph::new("│\n│")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(sep1, columns[1]);
            
            // Sección 2: Artefacto (doble línea centralizada)
            let artifact_text = format!("{}\n{} • v{}", artifact_name, artifact_type, ra.version);
            let artifact_paragraph = Paragraph::new(artifact_text)
                .style(Style::default().fg(text_color).add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(artifact_paragraph, columns[2]);
            
            // Separador 2 (doble línea)
            let sep2 = Paragraph::new("│\n│")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(sep2, columns[3]);
            
            // Sección 3: Stage (con color de fondo según el stage)
            let stage_bg_color = match stage_name.to_uppercase().as_str() {
                "DEV" => Color::Green,
                "QA" => Color::Yellow,
                "BETA" => Color::Magenta,
                "PROD" => Color::Red,
                _ => Color::DarkGray, // Color por defecto para otros stages
            };
            
            let stage_text = Paragraph::new(stage_upper)
                .style(Style::default().bg(stage_bg_color).fg(Color::Black))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(stage_text, columns[4]);
        }
    }
}

impl Screen for ReleasesScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        match &self.popup_state {
            PopupState::None => {
                match key.code {
                    KeyCode::Char('b') | KeyCode::Char('B') => Ok(ScreenOutcome::ChangeState(AppState::Home)),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Ok(ScreenOutcome::Quit),
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        self.popup_state = PopupState::Options;
                        self.selected_option = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        if self.submenu_focused {
                            self.list_focused = true;
                            self.submenu_focused = false;
                            if let Err(e) = self.load_releases() {
                                eprintln!("Error loading releases: {}", e);
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if self.submenu_focused {
                            self.release_focused = true;
                            self.submenu_focused = false;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if self.list_focused && !self.releases.is_empty() && self.selected_release_in_list < self.releases.len() - 1 {
                            self.selected_release_in_list += 1;
                        } else if self.deploy_focused {
                            if let Some(ref release) = self.selected_release_for_info {
                                if !release.artifacts.is_empty() && self.selected_deploy_artifact < release.artifacts.len() - 1 {
                                    self.selected_deploy_artifact += 1;
                                    
                                    // Ajustar scroll si es necesario
                                    let available_slots = 3; // Aproximadamente 3 artifacts visibles
                                    if self.selected_deploy_artifact >= self.deploy_scroll_offset + available_slots {
                                        self.deploy_scroll_offset = self.selected_deploy_artifact - available_slots + 1;
                                    }
                                }
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.list_focused && self.selected_release_in_list > 0 {
                            self.selected_release_in_list -= 1;
                        } else if self.deploy_focused && self.selected_deploy_artifact > 0 {
                            self.selected_deploy_artifact -= 1;
                            
                            // Ajustar scroll si es necesario
                            if self.selected_deploy_artifact < self.deploy_scroll_offset {
                                self.deploy_scroll_offset = self.selected_deploy_artifact;
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if self.list_focused && !self.releases.is_empty() && self.selected_release_in_list < self.releases.len() {
                            self.selected_release_for_info = Some(self.releases[self.selected_release_in_list].clone());
                            // Limpiar estado de notificación al cambiar de release
                            self.last_notified_release = None;
                            // Cargar datos necesarios para mostrar artifacts
                            if let Err(e) = self.load_artifacts() {
                                eprintln!("Error loading artifacts: {}", e);
                            }
                            if let Err(e) = self.load_artifact_types() {
                                eprintln!("Error loading artifact types: {}", e);
                            }
                            if let Err(e) = self.load_countries() {
                                eprintln!("Error loading countries: {}", e);
                            }
                            if let Err(e) = self.load_stages() {
                                eprintln!("Error loading stages: {}", e);
                            }
                            self.list_focused = false;
                            self.release_focused = true;
                            self.selected_deploy_artifact = 0;
                            self.deploy_scroll_offset = 0;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Esc => {
                        if self.list_focused {
                            self.list_focused = false;
                            self.submenu_focused = true;
                        } else if self.release_focused {
                            self.release_focused = false;
                            self.submenu_focused = true;
                            self.deploy_focused = false;
                        } else if self.deploy_focused {
                            self.deploy_focused = false;
                            self.release_focused = true;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Tab => {
                        if self.release_focused {
                            self.release_focused = false;
                            self.deploy_focused = true;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if self.release_focused {
                            self.selected_release_for_info = None;
                            self.last_notified_release = None; // Limpiar estado de notificación
                            self.release_focused = false;
                            self.list_focused = true;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        if self.release_focused && self.selected_release_for_info.is_some() {
                            self.popup_state = PopupState::AddReleaseArtifact;
                            self.search_text = String::new();
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if self.deploy_focused {
                            if let Some(ref release) = self.selected_release_for_info {
                                if !release.artifacts.is_empty() && self.selected_deploy_artifact < release.artifacts.len() {
                                    if let Err(e) = self.load_stages() {
                                        eprintln!("Error loading stages: {}", e);
                                    }
                                    self.selected_stage_for_edit = 0;
                                    self.popup_state = PopupState::EditArtifactStage;
                                }
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        if self.deploy_focused {
                            if let Some(ref release) = self.selected_release_for_info {
                                if !release.artifacts.is_empty() && self.selected_deploy_artifact < release.artifacts.len() {
                                    self.version_text = release.artifacts[self.selected_deploy_artifact].version.clone();
                                    self.popup_state = PopupState::EditArtifactVersion;
                                }
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        if self.deploy_focused {
                            if let Some(ref release) = self.selected_release_for_info {
                                if release.artifacts.len() > 1 && self.selected_deploy_artifact < release.artifacts.len() {
                                    self.source_artifact_index = self.selected_deploy_artifact;
                                    self.selected_move_artifact = 0;
                                    self.popup_state = PopupState::MoveArtifact;
                                }
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        if self.release_focused && self.selected_release_for_info.is_some() {
                            self.selected_print_format = 0;
                            self.popup_state = PopupState::PrintArtifacts;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if self.deploy_focused {
                            if let Some(ref mut release) = self.selected_release_for_info {
                                if !release.artifacts.is_empty() && self.selected_deploy_artifact < release.artifacts.len() {
                                    release.artifacts.remove(self.selected_deploy_artifact);
                                    
                                    // Ajustar el índice seleccionado si es necesario
                                    if self.selected_deploy_artifact >= release.artifacts.len() && !release.artifacts.is_empty() {
                                        self.selected_deploy_artifact = release.artifacts.len() - 1;
                                    } else if release.artifacts.is_empty() {
                                        self.selected_deploy_artifact = 0;
                                    }
                                    
                                    if let Err(e) = self.release_repo.update_release_complete(release) {
                                        eprintln!("Error updating release: {}", e);
                                        self.notification = Some(Notification::error("Error al eliminar artefacto".to_string()));
                                    } else {
                                        self.notification = Some(Notification::success("Artefacto eliminado exitosamente".to_string()));
                                    }
                                }
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::Options => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if self.selected_option < self.options.len() - 1 {
                            self.selected_option += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_option > 0 {
                            self.selected_option -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if self.selected_option == 0 { // "Manage Releases"
                            self.popup_state = PopupState::ManageReleases;
                            self.selected_release = 0;
                            if let Err(e) = self.load_releases() {
                                eprintln!("Error loading releases: {}", e);
                            }
                        } else if self.selected_option == 1 { // "Manage Artifacts"
                            self.popup_state = PopupState::ManageArtifacts;
                            self.selected_artifact = 0;
                            if let Err(e) = self.load_artifacts() {
                                eprintln!("Error loading artifacts: {}", e);
                            }
                            if let Err(e) = self.load_artifact_types() {
                                eprintln!("Error loading artifact types: {}", e);
                            }
                        } else if self.selected_option == 2 { // "Manage Stages"
                            self.popup_state = PopupState::ManageStages;
                            self.selected_stage = 0;
                            if let Err(e) = self.load_stages() {
                                eprintln!("Error loading stages: {}", e);
                            }
                            if let Err(e) = self.load_countries() {
                                eprintln!("Error loading countries: {}", e);
                            }
                        } else if self.selected_option == 3 { // "Manage Countries"
                            self.popup_state = PopupState::ManageCountries;
                            self.selected_country = 0;
                            if let Err(e) = self.load_countries() {
                                eprintln!("Error loading countries: {}", e);
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::ManageCountries => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::Options;
                        self.notification = None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.countries.is_empty() && self.selected_country < self.countries.len() - 1 {
                            self.selected_country += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_country > 0 {
                            self.selected_country -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.popup_state = PopupState::EditCountry(String::new());
                        self.edit_text = String::new();
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if !self.countries.is_empty() && self.selected_country < self.countries.len() {
                            let country_id = self.countries[self.selected_country].id.clone();
                            self.edit_text = self.countries[self.selected_country].name.clone();
                            self.popup_state = PopupState::EditCountry(country_id);
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if !self.countries.is_empty() && self.selected_country < self.countries.len() {
                            let country_id = &self.countries[self.selected_country].id;
                            
                            // Verificar si hay stages asociados a este país
                            if let Ok(stages) = self.stage_repo.get_all_stages() {
                                let has_associated_stages = stages.iter().any(|stage| stage.country_id == *country_id);
                                
                                if has_associated_stages {
                                    self.notification = Some(Notification::error("No se puede borrar un país asociado a un stage".to_string()));
                                } else {
                                    if let Err(e) = self.country_repo.delete_country(country_id) {
                                        eprintln!("Error deleting country: {}", e);
                                    } else {
                                        self.notification = Some(Notification::success("País eliminado exitosamente".to_string()));
                                    }
                                    if let Err(e) = self.load_countries() {
                                        eprintln!("Error loading countries: {}", e);
                                    }
                                }
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EditCountry(_) => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::ManageCountries;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let PopupState::EditCountry(id) = &self.popup_state {
                            if id.is_empty() {
                                // Agregar nuevo país
                                if let Err(e) = self.country_repo.add_country(self.edit_text.clone()) {
                                    eprintln!("Error adding country: {}", e);
                                } else {
                                    self.notification = Some(Notification::success("País agregado exitosamente".to_string()));
                                }
                            } else {
                                // Editar país existente
                                if let Err(e) = self.country_repo.update_country(id, self.edit_text.clone()) {
                                    eprintln!("Error updating country: {}", e);
                                } else {
                                    self.notification = Some(Notification::success("País actualizado exitosamente".to_string()));
                                }
                            }
                            if let Err(e) = self.load_countries() {
                                eprintln!("Error loading countries: {}", e);
                            }
                            self.popup_state = PopupState::ManageCountries;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        self.edit_text.push(c);
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        self.edit_text.pop();
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::ManageStages => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::Options;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.stages.is_empty() && self.selected_stage < self.stages.len() - 1 {
                            self.selected_stage += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_stage > 0 {
                            self.selected_stage -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.popup_state = PopupState::EditStage(String::new());
                        self.edit_text = String::new();
                        self.selected_country_for_stage = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if !self.stages.is_empty() && self.selected_stage < self.stages.len() {
                            let stage = &self.stages[self.selected_stage];
                            self.edit_text = stage.name.clone();
                            self.selected_country_for_stage = self.countries.iter()
                                .position(|c| c.id == stage.country_id)
                                .unwrap_or(0);
                            self.popup_state = PopupState::EditStage(stage.id.clone());
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if !self.stages.is_empty() && self.selected_stage < self.stages.len() {
                            let stage_id = &self.stages[self.selected_stage].id;
                            if let Err(e) = self.stage_repo.delete_stage(stage_id) {
                                eprintln!("Error deleting stage: {}", e);
                            } else {
                                self.notification = Some(Notification::success("Stage eliminado exitosamente".to_string()));
                            }
                            if let Err(e) = self.load_stages() {
                                eprintln!("Error loading stages: {}", e);
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EditStage(_) => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::ManageStages;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Tab => {
                        if !self.countries.is_empty() {
                            self.selected_country_for_stage = (self.selected_country_for_stage + 1) % self.countries.len();
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let PopupState::EditStage(id) = &self.popup_state {
                            if !self.countries.is_empty() && self.selected_country_for_stage < self.countries.len() {
                                let country_id = self.countries[self.selected_country_for_stage].id.clone();
                                if id.is_empty() {
                                    if let Err(e) = self.stage_repo.add_stage(self.edit_text.clone(), country_id) {
                                        eprintln!("Error adding stage: {}", e);
                                    } else {
                                        self.notification = Some(Notification::success("Stage agregado exitosamente".to_string()));
                                    }
                                } else {
                                    if let Err(e) = self.stage_repo.update_stage(id, self.edit_text.clone(), country_id) {
                                        eprintln!("Error updating stage: {}", e);
                                    } else {
                                        self.notification = Some(Notification::success("Stage actualizado exitosamente".to_string()));
                                    }
                                }
                                if let Err(e) = self.load_stages() {
                                    eprintln!("Error loading stages: {}", e);
                                }
                                self.popup_state = PopupState::ManageStages;
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        self.edit_text.push(c);
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        self.edit_text.pop();
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::ManageArtifacts => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::Options;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.artifacts.is_empty() && self.selected_artifact < self.artifacts.len() - 1 {
                            self.selected_artifact += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_artifact > 0 {
                            self.selected_artifact -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.popup_state = PopupState::EditArtifact(String::new());
                        self.edit_text = String::new();
                        self.selected_artifact_type = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if !self.artifacts.is_empty() && self.selected_artifact < self.artifacts.len() {
                            let artifact = &self.artifacts[self.selected_artifact];
                            self.edit_text = artifact.name.clone();
                            self.selected_artifact_type = self.artifact_types.iter()
                                .position(|t| t.id == artifact.artifact_type_id)
                                .unwrap_or(0);
                            self.popup_state = PopupState::EditArtifact(artifact.id.clone());
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if !self.artifacts.is_empty() && self.selected_artifact < self.artifacts.len() {
                            let artifact_id = &self.artifacts[self.selected_artifact].id;
                            if let Err(e) = self.artifact_repo.delete_artifact(artifact_id) {
                                eprintln!("Error deleting artifact: {}", e);
                            } else {
                                self.notification = Some(Notification::success("Artifact eliminado exitosamente".to_string()));
                            }
                            if let Err(e) = self.load_artifacts() {
                                eprintln!("Error loading artifacts: {}", e);
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EditArtifact(_) => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::ManageArtifacts;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Tab => {
                        if !self.artifact_types.is_empty() {
                            self.selected_artifact_type = (self.selected_artifact_type + 1) % self.artifact_types.len();
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let PopupState::EditArtifact(id) = &self.popup_state {
                            if !self.artifact_types.is_empty() && self.selected_artifact_type < self.artifact_types.len() {
                                let artifact_type_id = self.artifact_types[self.selected_artifact_type].id;
                                if id.is_empty() {
                                    if let Err(e) = self.artifact_repo.add_artifact(self.edit_text.clone(), artifact_type_id) {
                                        eprintln!("Error adding artifact: {}", e);
                                    } else {
                                        self.notification = Some(Notification::success("Artifact agregado exitosamente".to_string()));
                                    }
                                } else {
                                    if let Err(e) = self.artifact_repo.update_artifact(id, self.edit_text.clone(), artifact_type_id) {
                                        eprintln!("Error updating artifact: {}", e);
                                    } else {
                                        self.notification = Some(Notification::success("Artifact actualizado exitosamente".to_string()));
                                    }
                                }
                                if let Err(e) = self.load_artifacts() {
                                    eprintln!("Error loading artifacts: {}", e);
                                }
                                self.popup_state = PopupState::ManageArtifacts;
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        self.edit_text.push(c);
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        self.edit_text.pop();
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::ManageReleases => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::Options;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.releases.is_empty() && self.selected_release < self.releases.len() - 1 {
                            self.selected_release += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_release > 0 {
                            self.selected_release -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.popup_state = PopupState::EditRelease(String::new());
                        self.edit_text = String::new();
                        self.edit_year = String::new();
                        self.edit_date_init = String::new();
                        self.edit_date_qa = String::new();
                        self.edit_date_finish = String::new();
                        self.edit_field = 0;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if !self.releases.is_empty() && self.selected_release < self.releases.len() {
                            let release = &self.releases[self.selected_release];
                            self.edit_text = release.name.clone();
                            self.edit_year = release.year.to_string();
                            self.edit_date_init = release.date_init.clone();
                            self.edit_date_qa = release.date_qa.clone();
                            self.edit_date_finish = release.date_finish.clone();
                            self.edit_field = 0;
                            self.popup_state = PopupState::EditRelease(release.id.clone());
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if !self.releases.is_empty() && self.selected_release < self.releases.len() {
                            let release_id = &self.releases[self.selected_release].id;
                            if let Err(e) = self.release_repo.delete_release(release_id) {
                                eprintln!("Error deleting release: {}", e);
                            } else {
                                self.notification = Some(Notification::success("Release eliminado exitosamente".to_string()));
                            }
                            if let Err(e) = self.load_releases() {
                                eprintln!("Error loading releases: {}", e);
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EditRelease(_) => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::ManageReleases;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Tab => {
                        self.edit_field = (self.edit_field + 1) % 5;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let PopupState::EditRelease(id) = &self.popup_state {
                            let year = self.edit_year.parse::<u32>().unwrap_or(2024);
                            if id.is_empty() {
                                if let Err(e) = self.release_repo.add_release(
                                    self.edit_text.clone(),
                                    year,
                                    self.edit_date_init.clone(),
                                    self.edit_date_qa.clone(),
                                    self.edit_date_finish.clone()
                                ) {
                                    eprintln!("Error adding release: {}", e);
                                } else {
                                    self.notification = Some(Notification::success("Release agregado exitosamente".to_string()));
                                }
                            } else {
                                if let Err(e) = self.release_repo.update_release(
                                    id,
                                    self.edit_text.clone(),
                                    year,
                                    self.edit_date_init.clone(),
                                    self.edit_date_qa.clone(),
                                    self.edit_date_finish.clone()
                                ) {
                                    eprintln!("Error updating release: {}", e);
                                } else {
                                    self.notification = Some(Notification::success("Release actualizado exitosamente".to_string()));
                                }
                            }
                            if let Err(e) = self.load_releases() {
                                eprintln!("Error loading releases: {}", e);
                            }
                            self.popup_state = PopupState::ManageReleases;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        match self.edit_field {
                            0 => self.edit_text.push(c),
                            1 => self.edit_year.push(c),
                            2 => self.edit_date_init.push(c),
                            3 => self.edit_date_qa.push(c),
                            4 => self.edit_date_finish.push(c),
                            _ => {}
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        match self.edit_field {
                            0 => { self.edit_text.pop(); }
                            1 => { self.edit_year.pop(); }
                            2 => { self.edit_date_init.pop(); }
                            3 => { self.edit_date_qa.pop(); }
                            4 => { self.edit_date_finish.pop(); }
                            _ => {}
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::AddReleaseArtifact => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if self.search_text.len() >= 3 {
                            if let Err(e) = self.load_artifacts() {
                                eprintln!("Error loading artifacts: {}", e);
                            }
                            self.filtered_artifacts = self.artifacts.iter()
                                .filter(|a| a.name.to_lowercase().contains(&self.search_text.to_lowercase()))
                                .cloned()
                                .collect();
                            self.selected_filtered_artifact = 0;
                            self.popup_state = PopupState::SearchArtifacts(self.search_text.clone());
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        self.search_text.push(c);
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        self.search_text.pop();
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::SearchArtifacts(_) => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::AddReleaseArtifact;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.filtered_artifacts.is_empty() && self.selected_filtered_artifact < self.filtered_artifacts.len() - 1 {
                            self.selected_filtered_artifact += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_filtered_artifact > 0 {
                            self.selected_filtered_artifact -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if !self.filtered_artifacts.is_empty() && self.selected_filtered_artifact < self.filtered_artifacts.len() {
                            self.temp_artifact_id = Some(self.filtered_artifacts[self.selected_filtered_artifact].id.clone());
                            if let Err(e) = self.load_countries() {
                                eprintln!("Error loading countries: {}", e);
                            }
                            self.selected_country_for_artifact = 0;
                            self.popup_state = PopupState::SelectCountry;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::SelectCountry => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::AddReleaseArtifact;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.countries.is_empty() && self.selected_country_for_artifact < self.countries.len() - 1 {
                            self.selected_country_for_artifact += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_country_for_artifact > 0 {
                            self.selected_country_for_artifact -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if !self.stages.is_empty() && self.selected_stage_for_artifact < self.stages.len() {
                            self.temp_country_id = Some(self.countries[self.selected_country_for_artifact].id.clone());
                            if let Err(e) = self.load_stages() {
                                eprintln!("Error loading stages: {}", e);
                            }
                            self.selected_stage_for_artifact = 0;
                            self.popup_state = PopupState::SelectStage;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::SelectStage => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::SelectCountry;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if !self.stages.is_empty() && self.selected_stage_for_artifact < self.stages.len() - 1 {
                            self.selected_stage_for_artifact += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_stage_for_artifact > 0 {
                            self.selected_stage_for_artifact -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if !self.stages.is_empty() && self.selected_stage_for_artifact < self.stages.len() {
                            self.version_text = String::new();
                            self.popup_state = PopupState::EnterVersion;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EnterVersion => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::SelectStage;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let Some(ref mut release) = self.selected_release_for_info {
                            if !self.filtered_artifacts.is_empty() && 
                               self.selected_filtered_artifact < self.filtered_artifacts.len() &&
                               !self.countries.is_empty() && 
                               self.selected_country_for_artifact < self.countries.len() &&
                               !self.stages.is_empty() && 
                               self.selected_stage_for_artifact < self.stages.len() {
                                
                                let artifact_id = self.filtered_artifacts[self.selected_filtered_artifact].id.clone();
                                let country_id = self.countries[self.selected_country_for_artifact].id.clone();
                                let stage_id = self.stages[self.selected_stage_for_artifact].id.clone();
                                let version = self.version_text.clone();
                                let order = release.artifacts.len() + 1;
                                
                                let new_artifact = ReleaseArtifact {
                                    artifact_id,
                                    country_id,
                                    stage_id,
                                    version,
                                    order,
                                };
                                
                                release.artifacts.push(new_artifact);
                                
                                if let Err(e) = self.release_repo.update_release_complete(release) {
                                    eprintln!("Error updating release: {}", e);
                                    self.notification = Some(Notification::error("Error al agregar artefacto".to_string()));
                                } else {
                                    self.notification = Some(Notification::success("Artefacto agregado exitosamente".to_string()));
                                }
                            }
                        }
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        self.version_text.push(c);
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        self.version_text.pop();
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EditArtifactStage => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if self.selected_stage_for_edit < self.stages.len().saturating_sub(1) {
                            self.selected_stage_for_edit += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_stage_for_edit > 0 {
                            self.selected_stage_for_edit -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let Some(ref mut release) = self.selected_release_for_info {
                            if !release.artifacts.is_empty() && 
                               self.selected_deploy_artifact < release.artifacts.len() &&
                               !self.stages.is_empty() && 
                               self.selected_stage_for_edit < self.stages.len() {
                                
                                let new_stage_id = self.stages[self.selected_stage_for_edit].id.clone();
                                release.artifacts[self.selected_deploy_artifact].stage_id = new_stage_id;
                                
                                if let Err(e) = self.release_repo.update_release_complete(release) {
                                    eprintln!("Error updating release: {}", e);
                                    self.notification = Some(Notification::error("Error al actualizar stage".to_string()));
                                } else {
                                    self.notification = Some(Notification::success("Stage actualizado exitosamente".to_string()));
                                }
                            }
                        }
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::EditArtifactVersion => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let Some(ref mut release) = self.selected_release_for_info {
                            if !release.artifacts.is_empty() && 
                               self.selected_deploy_artifact < release.artifacts.len() {
                                
                                release.artifacts[self.selected_deploy_artifact].version = self.version_text.clone();
                                
                                if let Err(e) = self.release_repo.update_release_complete(release) {
                                    eprintln!("Error updating release: {}", e);
                                    self.notification = Some(Notification::error("Error al actualizar versión".to_string()));
                                } else {
                                    self.notification = Some(Notification::success("Versión actualizada exitosamente".to_string()));
                                }
                            }
                        }
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char(c) => {
                        self.version_text.push(c);
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Backspace => {
                        self.version_text.pop();
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::MoveArtifact => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if let Some(ref release) = self.selected_release_for_info {
                            let available_count = release.artifacts.len() - 1;
                            if self.selected_move_artifact < available_count.saturating_sub(1) {
                                self.selected_move_artifact += 1;
                            }
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_move_artifact > 0 {
                            self.selected_move_artifact -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let Some(ref mut release) = self.selected_release_for_info {
                            let mut sorted_artifacts: Vec<_> = release.artifacts.iter().enumerate().collect();
                            sorted_artifacts.sort_by_key(|(_, a)| a.order);
                            
                            let filtered: Vec<_> = sorted_artifacts.iter()
                                .filter(|(idx, _)| *idx != self.source_artifact_index)
                                .collect();
                            
                            if self.selected_move_artifact < filtered.len() {
                                let target_index = filtered[self.selected_move_artifact].0;
                                let source_order = release.artifacts[self.source_artifact_index].order;
                                let target_order = release.artifacts[target_index].order;
                                
                                release.artifacts[self.source_artifact_index].order = target_order;
                                release.artifacts[target_index].order = source_order;
                                
                                if let Err(e) = self.release_repo.update_release_complete(release) {
                                    eprintln!("Error updating release: {}", e);
                                    self.notification = Some(Notification::error("Error al mover artefacto".to_string()));
                                } else {
                                    self.notification = Some(Notification::success("Artefacto movido exitosamente".to_string()));
                                }
                            }
                        }
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
            PopupState::PrintArtifacts => {
                match key.code {
                    KeyCode::Esc => {
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('j') => {
                        if self.selected_print_format < 1 {
                            self.selected_print_format += 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Char('k') => {
                        if self.selected_print_format > 0 {
                            self.selected_print_format -= 1;
                        }
                        Ok(ScreenOutcome::Continue)
                    }
                    KeyCode::Enter => {
                        if let Some(ref release) = self.selected_release_for_info {
                            let format = if self.selected_print_format == 0 { "png" } else { "pdf" };
                            match self.export_release_artifacts(release, format) {
                                Ok(path) => {
                                    self.notification = Some(Notification::success(
                                        format!("Archivo exportado: {}", path)
                                    ));
                                }
                                Err(e) => {
                                    self.notification = Some(Notification::error(
                                        format!("Error al exportar: {}", e)
                                    ));
                                }
                            }
                        }
                        self.popup_state = PopupState::None;
                        Ok(ScreenOutcome::Continue)
                    }
                    _ => Ok(ScreenOutcome::Continue),
                }
            }
        }
    }

    fn draw(&mut self, f: &mut Frame, _context: &ScreenContext) {
        let size = f.size();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(size);

        let title_block = Line::from(vec![
            Span::styled("🚀 ", Style::default().fg(Color::LightBlue)),
            Span::styled("Releases", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .border_type(ratatui::widgets::BorderType::Thick)
            .title(title_block);
        
        // Calcular área interna antes de renderizar
        let inner_area = main_block.inner(main_layout[0]);
        f.render_widget(main_block, main_layout[0]);

        // Layout interno para List, Release y Submenu
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(inner_layout[0]);

        // Recuadro List (20%)
        let list_style = if self.list_focused {
            Style::default().fg(Color::LightYellow)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(list_style)
            .title(Span::styled(" 📋 List ", list_style));
        
        let list_inner = list_block.inner(top_layout[0]);
        f.render_widget(list_block, top_layout[0]);

        // Mostrar releases en el List
        if !self.releases.is_empty() {
            use ratatui::widgets::{List, ListItem};
            let items: Vec<ListItem> = self.releases
                .iter()
                .enumerate()
                .map(|(i, release)| {
                    let text = format!("{} - {}", release.year, release.name);
                    if i == self.selected_release_in_list && self.list_focused {
                        ListItem::new(text).style(Style::default().bg(Color::Blue).fg(Color::White))
                    } else {
                        ListItem::new(text)
                    }
                })
                .collect();
            let list = List::new(items);
            f.render_widget(list, list_inner);
        }

        // Recuadro Release (80%)
        let release_style = if self.release_focused {
            Style::default().fg(Color::LightGreen)
        } else {
            Style::default().fg(Color::Green)
        };
        let release_block = Block::default()
            .borders(Borders::ALL)
            .border_style(release_style)
            .title(Span::styled(" 🎯 Release ", release_style));
        
        let release_inner = release_block.inner(top_layout[1]);
        f.render_widget(release_block, top_layout[1]);

        // Layout interno del Release con margen
        let release_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .margin(1)
            .split(release_inner);

        // Subrecuadro Info
        let info_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(Span::styled(" ℹ Info ", Style::default().fg(Color::Blue)));
        
        let info_inner = info_block.inner(release_layout[0]);
        f.render_widget(info_block, release_layout[0]);

        // Mostrar información del release seleccionado
        if let Some(ref release) = self.selected_release_for_info {
            let info_text = format!("Name: {} | Year: {} | Date-Init: {} | Date-Qa: {} | Date-Finish: {}", 
                release.name, release.year, release.date_init, release.date_qa, release.date_finish);
            let info_paragraph = Paragraph::new(info_text);
            f.render_widget(info_paragraph, info_inner);
        }

        // Subrecuadro Deploy
        let deploy_style = if self.deploy_focused {
            Style::default().fg(Color::LightRed)
        } else {
            Style::default().fg(Color::Red)
        };
        let deploy_block = Block::default()
            .borders(Borders::ALL)
            .border_style(deploy_style)
            .title(Span::styled(" 🚀 Deploy ", deploy_style));
        
        let deploy_inner = deploy_block.inner(release_layout[1]);
        f.render_widget(deploy_block, release_layout[1]);

        // Mostrar artifacts del release con componente personalizado
        if let Some(ref release) = self.selected_release_for_info {
            if !release.artifacts.is_empty() {
                self.draw_custom_artifacts(f, deploy_inner, release);
            }
            
            // Verificar fecha de finalización y mostrar notificación
            let release_name = release.name.clone();
            let date_finish = release.date_finish.clone();
            self.check_release_date_notification(&release_name, &date_finish);
        }

        // Subrecuadro Options
        let options_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(Span::styled(" ⚙ Options ", Style::default().fg(Color::Magenta)));
        
        let options_inner = options_block.inner(release_layout[2]);
        f.render_widget(options_block, release_layout[2]);

        // Mostrar opciones - Edit, Version y Delete solo disponibles si hay artefacto seleccionado en Deploy
        let options_text = if self.deploy_focused && 
                             self.selected_release_for_info.is_some() && 
                             !self.selected_release_for_info.as_ref().unwrap().artifacts.is_empty() {
            "E: Edit | V: Version | M: Move | D: Delete"
        } else if self.release_focused && self.selected_release_for_info.is_some() {
            "A: Add | P: Print | C: Clear"
        } else {
            "A: Add | C: Clear"
        };
        let options_paragraph = Paragraph::new(options_text)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(options_paragraph, options_inner);

        // Recuadro Submenu
        let submenu_style = if self.submenu_focused {
            Style::default().fg(Color::LightCyan)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let submenu_text = Paragraph::new("L: List | R: Release")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(submenu_style)
                    .title(Span::styled(" ⚙ Submenu ", submenu_style))
            );
        f.render_widget(submenu_text, inner_layout[1]);

        let menu_block_title = Line::from(vec![
            Span::styled("⚙ ", Style::default().fg(Color::Magenta)),
            Span::styled("Menu", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let menu_text = Paragraph::new("Q: Quit | B: Back | O: Options")
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .title(menu_block_title)
            );
        f.render_widget(menu_text, main_layout[1]);

        // Dibujar popups según el estado
        match &self.popup_state {
            PopupState::Options => {
                self.draw_options_popup(f, size);
            }
            PopupState::ManageReleases => {
                self.draw_releases_popup(f, size);
            }
            PopupState::EditRelease(id) => {
                self.draw_edit_release_popup(f, size, id.is_empty());
            }
            PopupState::ManageCountries => {
                self.draw_countries_popup(f, size);
            }
            PopupState::EditCountry(id) => {
                self.draw_edit_popup(f, size, id.is_empty());
            }
            PopupState::ManageStages => {
                self.draw_stages_popup(f, size);
            }
            PopupState::EditStage(id) => {
                self.draw_edit_stage_popup(f, size, id.is_empty());
            }
            PopupState::ManageArtifacts => {
                self.draw_artifacts_popup(f, size);
            }
            PopupState::EditArtifact(id) => {
                self.draw_edit_artifact_popup(f, size, id.is_empty());
            }
            PopupState::AddReleaseArtifact => {
                self.draw_add_artifact_popup(f, size);
            }
            PopupState::SearchArtifacts(_) => {
                self.draw_search_artifacts_popup(f, size);
            }
            PopupState::SelectCountry => {
                self.draw_select_country_popup(f, size);
            }
            PopupState::SelectStage => {
                self.draw_select_stage_popup(f, size);
            }
            PopupState::EnterVersion => {
                self.draw_enter_version_popup(f, size);
            }
            PopupState::EditArtifactStage => {
                self.draw_edit_artifact_stage_popup(f, size);
            }
            PopupState::EditArtifactVersion => {
                self.draw_edit_artifact_version_popup(f, size);
            }
            PopupState::MoveArtifact => {
                self.draw_move_artifact_popup(f, size);
            }
            PopupState::PrintArtifacts => {
                self.draw_print_artifacts_popup(f, size);
            }
            PopupState::None => {}
        }

        // Dibujar notificación si existe
        self.draw_notification(f, size);
    }
}
