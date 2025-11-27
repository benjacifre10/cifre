// src/main.rs
mod domain;
mod infrastructure;
mod presentation;
mod application;

// Hacemos AppState público para que sea accesible desde las pantallas
pub use presentation::tui::AppState;

fn main() -> anyhow::Result<()> {
    let mut terminal = presentation::tui::setup_terminal()?;
    let mut app = presentation::tui::App::new()?;
    let res = app.run(&mut terminal);
    presentation::tui::restore_terminal(&mut terminal)?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}
