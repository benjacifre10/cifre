// src/presentation/screens/mod.rs
pub mod screen;
pub mod home_screen;
pub mod versions_screen;
pub mod todo_screen;
pub mod releases_screen;
pub mod settings_screen;
pub mod flows_screen;
pub mod artifacts_screen;
pub mod artifact_description_screen;
pub mod artifact_hpa_cpu_screen;
pub mod artifact_dependencies_screen;
pub mod artifact_endpoints_screen;
pub mod diagrams_screen;
pub mod miscellany_screen;
pub mod bills_screen;
pub mod investments_screen;

use anyhow::Result; // Asegúrate de que Result esté importado
use crossterm::event::KeyEvent; // ¡Importar KeyEvent!
use ratatui::Frame; // ¡Importar Frame!

// Exportar el trait y el enum para fácil acceso
pub use screen::{Screen, ScreenContext, ScreenOutcome};

// Nuevo enum para manejar todas las pantallas
pub enum AllScreens {
    Home(home_screen::HomeScreen),
    Versions(versions_screen::VersionsScreen),
    Todo(todo_screen::TodoScreen),
    Releases(releases_screen::ReleasesScreen),
    Settings(settings_screen::SettingsScreen),
    Flows(flows_screen::FlowsScreen),
    Artifacts(artifacts_screen::ArtifactsScreen),
    ArtifactDescription(artifact_description_screen::ArtifactDescriptionScreen),
    ArtifactHpaCpu(artifact_hpa_cpu_screen::ArtifactHpaCpuScreen),
    ArtifactDependencies(artifact_dependencies_screen::ArtifactDependenciesScreen),
    ArtifactEndpoints(artifact_endpoints_screen::ArtifactEndpointsScreen),
    Diagrams(diagrams_screen::DiagramsScreen),
    Miscellany(miscellany_screen::MiscellanyScreen),
    Bills(bills_screen::BillsScreen),
    Investments(investments_screen::InvestmentsScreen),
}

impl Screen for AllScreens {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ScreenOutcome> {
        match self {
            AllScreens::Home(s) => s.handle_key_event(key),
            AllScreens::Versions(s) => s.handle_key_event(key),
            AllScreens::Todo(s) => s.handle_key_event(key),
            AllScreens::Releases(s) => s.handle_key_event(key),
            AllScreens::Settings(s) => s.handle_key_event(key),
            AllScreens::Flows(s) => s.handle_key_event(key),
            AllScreens::Artifacts(s) => s.handle_key_event(key),
            AllScreens::ArtifactDescription(s) => s.handle_key_event(key),
            AllScreens::ArtifactHpaCpu(s) => s.handle_key_event(key),
            AllScreens::ArtifactDependencies(s) => s.handle_key_event(key),
            AllScreens::ArtifactEndpoints(s) => s.handle_key_event(key),
            AllScreens::Diagrams(s) => s.handle_key_event(key),
            AllScreens::Miscellany(s) => s.handle_key_event(key),
            AllScreens::Bills(s) => s.handle_key_event(key),
            AllScreens::Investments(s) => s.handle_key_event(key),
        }
    }

    fn draw(&mut self, f: &mut Frame, context: &ScreenContext) {
        match self {
            AllScreens::Home(s) => s.draw(f, context),
            AllScreens::Versions(s) => s.draw(f, context),
            AllScreens::Todo(s) => s.draw(f, context),
            AllScreens::Releases(s) => s.draw(f, context),
            AllScreens::Settings(s) => s.draw(f, context),
            AllScreens::Flows(s) => s.draw(f, context),
            AllScreens::Artifacts(s) => s.draw(f, context),
            AllScreens::ArtifactDescription(s) => s.draw(f, context),
            AllScreens::ArtifactHpaCpu(s) => s.draw(f, context),
            AllScreens::ArtifactDependencies(s) => s.draw(f, context),
            AllScreens::ArtifactEndpoints(s) => s.draw(f, context),
            AllScreens::Diagrams(s) => s.draw(f, context),
            AllScreens::Miscellany(s) => s.draw(f, context),
            AllScreens::Bills(s) => s.draw(f, context),
            AllScreens::Investments(s) => s.draw(f, context),
        }
    }
}
