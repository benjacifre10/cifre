// src/main.rs
mod domain;
mod infrastructure;
mod presentation;
mod application;

use std::io::Write;

// Hacemos AppState público para que sea accesible desde las pantallas
pub use presentation::tui::AppState;

fn main() -> anyhow::Result<()> {
    loop {
        let mut terminal = presentation::tui::setup_terminal()?;
        let mut app = presentation::tui::App::new()?;
        let res = app.run(&mut terminal);
        presentation::tui::restore_terminal(&mut terminal)?;

        if let Err(err) = res {
            eprintln!("{err:?}");
        }

        // Verificar si hay un directorio de helix para ejecutar
        if let Ok(helix_dir) = std::env::var("CIFRE_HELIX_DIR") {
            std::env::remove_var("CIFRE_HELIX_DIR");
            
            // Ejecutar helix en el directorio
            let _ = std::process::Command::new("hx")
                .arg(".")
                .current_dir(&helix_dir)
                .status();
            
            // Forzar reset del terminal después de helix
            print!("\x1Bc");
            std::io::stdout().flush()?;
            
            // Continuar el loop para volver a cifre
            continue;
        }

        // Verificar si hay un perfil de harlequin para ejecutar
        if let Ok(profile) = std::env::var("CIFRE_HARLEQUIN_PROFILE") {
            std::env::remove_var("CIFRE_HARLEQUIN_PROFILE");
            
            // Ejecutar harlequin
            let _ = std::process::Command::new("harlequin")
                .arg("--profile")
                .arg(&profile)
                .status();
            
            // Forzar reset del terminal después de harlequin
            print!("\x1Bc"); // Reset completo del terminal
            std::io::stdout().flush()?;
            
            // Continuar el loop para volver a cifre
            continue;
        }
        
        // Si no hay perfil, salir
        break;
    }

    Ok(())
}
