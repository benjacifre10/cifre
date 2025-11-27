// application/versions_artifact_service.rs
use std::{
    fs,
    path::PathBuf, // Eliminado 'Path'
    process::Command,
    // Eliminado 'io::Write'
};
use crate::domain::models::VersionsArtifact;
use serde_json::Value;

#[allow(dead_code)]
pub type ProgressCallback = dyn Fn(usize, Option<String>, Option<String>) + Send + Sync;

#[derive(Clone, Debug)]
pub struct VersionsArtifactService {
    pub base_path: PathBuf, // Hacemos base_path público para que el hilo lo pueda clonar
    pub output_dir: PathBuf,
    pub output_file_name: String,
}

impl VersionsArtifactService {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            output_dir: PathBuf::from("data"),
            output_file_name: "artifact_version.json".to_string(),
        }
    }

    pub fn new_with_output(base_path: &str, output_dir: &str, output_file_name: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            output_dir: PathBuf::from(output_dir),
            output_file_name: output_file_name.to_string(),
        }
    }

    pub fn read_versions_from_json(&self) -> Result<Vec<VersionsArtifact>, String> {
        let output_file_path = self.output_dir.join(&self.output_file_name);

        if !output_file_path.exists() {
            return Err(format!("El archivo de datos '{}' no existe. Por favor, genere los datos primero (presione G).", output_file_path.display()));
        }

        let contents = fs::read_to_string(&output_file_path)
            .map_err(|e| format!("Error al leer el archivo '{}': {}", output_file_path.display(), e))?;

        let versions: Vec<VersionsArtifact> = serde_json::from_str(&contents)
            .map_err(|e| format!("Error al deserializar los datos del archivo '{}': {}", output_file_path.display(), e))?;

        Ok(versions)
    }

    pub fn get_versions_artifact_data_with_progress<F>(
        &self,
        progress_callback: F,
    ) -> Result<Vec<VersionsArtifact>, String>
    where
        F: Fn(usize, Option<String>, Option<String>) + Send + Sync + 'static,
    {
        let mut versions_artifact_data = Vec::new();
        let base_path_display = self.base_path.display().to_string();

        if !self.base_path.exists() || !self.base_path.is_dir() {
            return Err(format!("El directorio '{}' no existe o no es un directorio.", base_path_display));
        }

        let repos: Vec<PathBuf> = fs::read_dir(&self.base_path)
            .map_err(|e| format!("Error al leer el directorio '{}': {}", base_path_display, e))?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        let num_repos = repos.len();
        let num_branches = 4;
        let total_steps = num_repos * num_branches;

        if total_steps == 0 {
            progress_callback(0, Some("No se encontraron repositorios.".to_string()), None);
        }

        let mut completed_steps = 0; // Se inicializa aquí

        for repo_path in repos {
            if let Some(repo_name) = repo_path.file_name().and_then(|n| n.to_str()) {
                progress_callback(completed_steps, Some(format!("Procesando repositorio: {}", repo_name)), None);

                let mut artifact = VersionsArtifact {
                    name: repo_name.to_string(),
                    ..Default::default()
                };

                if let Err(e) = self.process_repository_with_progress(
                    &repo_path,
                    &mut artifact,
                    &mut completed_steps,
                    &progress_callback,
                    num_repos,
                    num_branches,
                ) {
                    eprintln!("Error al procesar el repositorio '{}': {}", repo_name, e);
                    progress_callback(completed_steps, None, Some(format!("Error en {}: {}", repo_name, e)));
                }
                versions_artifact_data.push(artifact);
            }
        }

        progress_callback(total_steps, Some("Guardando datos...".to_string()), None);
        if let Err(e) = self.save_versions_to_json(&versions_artifact_data) {
            eprintln!("Error al guardar los datos de las versiones en JSON: {}", e);
            progress_callback(total_steps, None, Some(format!("Error al guardar: {}", e)));
        }
        progress_callback(total_steps, Some("Completado.".to_string()), None);

        Ok(versions_artifact_data)
    }

    fn process_repository_with_progress<F>(
        &self,
        repo_path: &PathBuf,
        artifact: &mut VersionsArtifact,
        completed_steps: &mut usize,
        progress_callback: &F,
        _num_repos: usize, // No se usan directamente aquí, pero estaban en la firma original.
        _num_branches: usize,
    ) -> Result<(), String>
    where
        F: Fn(usize, Option<String>, Option<String>) + Send + Sync + 'static,
    {
        let original_dir = std::env::current_dir().map_err(|e| format!("Error al obtener el directorio actual: {}", e))?;
        std::env::set_current_dir(repo_path).map_err(|e| format!("Error al cambiar al directorio {}: {}", repo_path.display(), e))?;

        let current_branch = self.get_current_git_branch().unwrap_or_else(|_| "main".to_string());

        let mut branches = [
            ("develop", &mut artifact.dev_version),
            ("qa", &mut artifact.qa_version),
            ("beta", &mut artifact.beta_version),
            ("master", &mut artifact.prod_version)
        ];

        for (branch_name, version_field) in branches.iter_mut() {
            let message = format!("Repo: {} | Rama: {}", artifact.name, branch_name);
            progress_callback(*completed_steps, Some(message), None);

            if self.checkout_and_pull(branch_name).is_ok() {
                if let Some(version) = self.get_version_from_package_json() {
                    **version_field = version; // CORREGIDO: doble desreferencia
                } else {
                    **version_field = "N/A".to_string(); // CORREGIDO: doble desreferencia
                }
            } else {
                **version_field = "Empty".to_string(); // CORREGIDO: doble desreferencia
            }
            *completed_steps += 1;
        }

        if let Err(e) = self.checkout_branch(&current_branch) {
            eprintln!("Advertencia: No se pudo restaurar la rama original '{}' en {}: {}", current_branch, repo_path.display(), e);
            progress_callback(*completed_steps, None, Some(format!("Advertencia en {}: {}", artifact.name, e)));
        }

        std::env::set_current_dir(&original_dir).map_err(|e| format!("Error al volver al directorio original: {}", e))?;
        Ok(())
    }

    fn save_versions_to_json(&self, versions: &Vec<VersionsArtifact>) -> Result<(), String> {
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)
                .map_err(|e| format!("Error al crear el directorio de salida '{}': {}", self.output_dir.display(), e))?;
        }

        let output_file_path = self.output_dir.join(&self.output_file_name);

        let json_string = serde_json::to_string_pretty(versions)
            .map_err(|e| format!("Error al serializar los datos a JSON: {}", e))?;

        fs::write(&output_file_path, json_string)
            .map_err(|e| format!("Error al escribir el archivo JSON en '{}': {}", output_file_path.display(), e))?;

        // println!("Datos de versiones guardados exitosamente en '{}'", output_file_path.display());
        Ok(())
    }

    fn run_git_command(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .args(args)
            .output()
            .map_err(|e| format!("Error al ejecutar comando git {:?}: {}", args, e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(format!("Error en comando git {:?}: {}", args, String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn get_current_git_branch(&self) -> Result<String, String> {
        self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    fn checkout_and_pull(&self, branch_name: &str) -> Result<(), String> {
        let checkout_result = self.run_git_command(&["checkout", branch_name]);
        if checkout_result.is_err() {
            return Err(format!("Error al hacer checkout a '{}': {}", branch_name, checkout_result.unwrap_err()));
        }
        let pull_result = self.run_git_command(&["pull", "origin", branch_name]);
        if pull_result.is_err() {
            return Err(format!("Error al hacer pull en '{}': {}", branch_name, pull_result.unwrap_err()));
        }
        Ok(())
    }

    fn checkout_branch(&self, branch_name: &str) -> Result<(), String> {
        self.run_git_command(&["checkout", branch_name])?;
        Ok(())
    }

    fn get_version_from_package_json(&self) -> Option<String> {
        let package_json_path = PathBuf::from("package.json");
        if package_json_path.exists() {
            if let Ok(contents) = fs::read_to_string(&package_json_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                    if let Some(version) = json["version"].as_str() {
                        return Some(version.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn get_base_path_display(&self) -> String {
        self.base_path.display().to_string()
    }
}


