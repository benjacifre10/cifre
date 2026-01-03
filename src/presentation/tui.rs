// cifre/src/presentation/tui.rs
use crate::presentation::screens::home_screen::HomeScreen;
use crate::presentation::screens::versions_screen::VersionsScreen;
use crate::presentation::screens::todo_screen::TodoScreen;
use crate::presentation::screens::releases_screen::ReleasesScreen;
use crate::presentation::screens::settings_screen::SettingsScreen;
use crate::presentation::screens::flows_screen::FlowsScreen;
use crate::presentation::screens::artifacts_screen::ArtifactsScreen;
use crate::presentation::screens::artifact_description_screen::ArtifactDescriptionScreen;
use crate::presentation::screens::artifact_hpa_cpu_screen::ArtifactHpaCpuScreen;
use crate::presentation::screens::artifact_dependencies_screen::ArtifactDependenciesScreen;
use crate::presentation::screens::artifact_endpoints_screen::ArtifactEndpointsScreen;
use crate::presentation::screens::diagrams_screen::DiagramsScreen;
use crate::presentation::screens::miscellany_screen::MiscellanyScreen;
use crate::presentation::screens::bills_screen::BillsScreen;
use crate::presentation::screens::investments_screen::InvestmentsScreen;

use anyhow::{Context, Result};
use chrono::{Local, DateTime};
use crossterm::{
    event::{self, Event as CEvent}, // Importamos KeyEvent
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};
use super::screens::{
    screen::{Screen, ScreenContext, ScreenOutcome},
    AllScreens,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)] // Derivamos Hash para usarlo como clave en HashMap
pub enum AppState {
    Home,
    ViewingVersions,
    ViewingTodo,
    ViewingReleases,
    ViewingSettings,
    ViewingFlows,
    ViewingArtifacts,
    ViewingArtifactDescription(String),
    ViewingArtifactHpaCpu(String),
    ViewingArtifactDependencies(String),
    ViewingArtifactEndpoints(String),
    ViewingDiagrams,
    ViewingMiscellany,
    ViewingBills,
    ViewingInvestments,
    Quit,
}

pub struct App {
    pub state: AppState,
    last_tick: Instant,
    current_datetime: DateTime<Local>,
    current_screen: AllScreens,
    previous_state: Option<AppState>,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: AppState::Home,
            last_tick: Instant::now(),
            current_datetime: Local::now(),
            current_screen: AllScreens::Home(HomeScreen::new()?),
            previous_state: None,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(terminal.backend_mut(), EnterAlternateScreen).context("Failed to enter alternate screen")?;

        let tick_rate = Duration::from_millis(250);

        loop {
            // Dibuja la UI delegando a la pantalla actual
            terminal.draw(|f| {
                let context = ScreenContext {
                    current_datetime: self.current_datetime,
                };
                self.current_screen.draw(f, &context);
            })?;

            let timeout = tick_rate
                .checked_sub(self.last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).context("Event poll failed")? {
                if let CEvent::Key(key_event) = event::read().context("Event read failed")? {
                    // Delega el manejo del evento a la pantalla actual
                    match self.current_screen.handle_key_event(key_event)? {
                        ScreenOutcome::Continue => { /* No hacemos nada */ },
                        ScreenOutcome::ChangeState(new_state) => {
                            self.previous_state = Some(self.state.clone());
                            self.state = new_state.clone();
                            self.current_screen = match new_state {
                                AppState::Home => {
                                    if let Some(prev) = &self.previous_state {
                                        if *prev == AppState::ViewingTodo || *prev == AppState::ViewingReleases {
                                            AllScreens::Home(HomeScreen::new_with_focus()?)
                                        } else {
                                            AllScreens::Home(HomeScreen::new()?)
                                        }
                                    } else {
                                        AllScreens::Home(HomeScreen::new()?)
                                    }
                                },
                                AppState::ViewingVersions => AllScreens::Versions(VersionsScreen::new()),
                                AppState::ViewingTodo => AllScreens::Todo(TodoScreen::new()),
                                AppState::ViewingReleases => AllScreens::Releases(ReleasesScreen::new()),
                                AppState::ViewingSettings => AllScreens::Settings(SettingsScreen::new()),
                                AppState::ViewingFlows => AllScreens::Flows(FlowsScreen::new()),
                                AppState::ViewingArtifacts => AllScreens::Artifacts(ArtifactsScreen::new()),
                                AppState::ViewingArtifactDescription(name) => AllScreens::ArtifactDescription(ArtifactDescriptionScreen::new(name)),
                                AppState::ViewingArtifactHpaCpu(name) => AllScreens::ArtifactHpaCpu(ArtifactHpaCpuScreen::new(name)),
                                AppState::ViewingArtifactDependencies(name) => AllScreens::ArtifactDependencies(ArtifactDependenciesScreen::new(name)),
                                AppState::ViewingArtifactEndpoints(name) => AllScreens::ArtifactEndpoints(ArtifactEndpointsScreen::new(name)),
                                AppState::ViewingDiagrams => AllScreens::Diagrams(DiagramsScreen::new()),
                                AppState::ViewingMiscellany => AllScreens::Miscellany(MiscellanyScreen::new()),
                                AppState::ViewingBills => AllScreens::Bills(BillsScreen::new()),
                                AppState::ViewingInvestments => AllScreens::Investments(InvestmentsScreen::new()),
                                AppState::Quit => AllScreens::Home(HomeScreen::new()?),
                            };
                        },
                        ScreenOutcome::Quit => {
                            self.state = AppState::Quit;
                        },
                        ScreenOutcome::LaunchHelix(dir) => {
                            std::env::set_var("CIFRE_HELIX_DIR", dir.to_string_lossy().to_string());
                            self.state = AppState::Quit;
                        },
                    }
                }
            }

            if self.last_tick.elapsed() >= tick_rate {
                self.current_datetime = Local::now();
                self.last_tick = Instant::now();
            }

            if let AppState::Quit = self.state {
                break;
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;
        disable_raw_mode().context("Failed to disable raw mode")?;

        Ok(())
    }
}

// setup_terminal y restore_terminal van aquí
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("Failed to create terminal")
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;
    disable_raw_mode().context("Failed to disable raw mode")?;
    Ok(())
}
