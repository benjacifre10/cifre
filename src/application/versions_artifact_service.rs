// application/versions_artifact_service.rs
use std::{
    fs,
    path::{PathBuf, Path},
    process::Command,
};
use crate::domain::models::VersionsArtifact;
use serde_json::Value;

#[allow(dead_code)]
pub type ProgressCallback = dyn Fn(usize, Option<String>, Option<String>) + Send + Sync;

#[derive(Clone, Debug)]
pub struct VersionsArtifactService {
    pub base_path: PathBuf,
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
                    // Solo consideramos directorios que son repositorios Git (contienen una carpeta .git)
                    if path.is_dir() && path.join(".git").exists() {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        let num_repos = repos.len();
        // AHORA SON 3 ramas: dev, release, prod
        let num_branches = 3;
        let total_steps = num_repos * num_branches;

        if total_steps == 0 {
            progress_callback(0, Some("No se encontraron repositorios.".to_string()), None);
        }

        let mut completed_steps = 0;

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
    ) -> Result<(), String>
    where
        F: Fn(usize, Option<String>, Option<String>) + Send + Sync + 'static,
    {
        let original_dir = std::env::current_dir().map_err(|e| format!("Error al obtener el directorio actual: {}", e))?;
        std::env::set_current_dir(repo_path).map_err(|e| format!("Error al cambiar al directorio {}: {}", repo_path.display(), e))?;

        let current_branch = self.get_current_git_branch().unwrap_or_else(|_| "main".to_string());

        // --- Procesar rama 'dev' ---
        let dev_branch_name = "develop";
        let dev_version_str = {
            let message = format!("Repo: {} | Rama: {}", artifact.name, dev_branch_name);
            progress_callback(*completed_steps, Some(message), None);

            if self.checkout_and_pull(dev_branch_name).is_ok() {
                if let Some(version) = self.get_version_from_package_json(repo_path) {
                    artifact.dev_version = version.clone();
                    version
                } else {
                    artifact.dev_version = "N/A".to_string();
                    "N/A".to_string()
                }
            } else {
                artifact.dev_version = "Empty".to_string();
                "Empty".to_string()
            }
        };
        *completed_steps += 1;

        // --- Procesar rama 'prod' ---
        let prod_branch_name = "master";
        let _prod_version_str = {
            let message = format!("Repo: {} | Rama: {}", artifact.name, prod_branch_name);
            progress_callback(*completed_steps, Some(message), None);

            if self.checkout_and_pull(prod_branch_name).is_ok() {
                if let Some(version) = self.get_version_from_package_json(repo_path) {
                    artifact.prod_version = version.clone();
                    version
                } else {
                    artifact.prod_version = "N/A".to_string();
                    "N/A".to_string()
                }
            } else {
                artifact.prod_version = "Empty".to_string();
                "Empty".to_string()
            }
        };
        *completed_steps += 1;

        // --- Procesar rama 'release' (basado en dev_version, sin checkout ni package.json de release) ---
        let release_git_branch = format!("release/{}", dev_version_str);
        let message = format!("Repo: {} | Rama: {}", artifact.name, release_git_branch);
        progress_callback(*completed_steps, Some(message), None);

        if dev_version_str == "N/A" || dev_version_str == "Empty" {
            artifact.release_version = "not_applicable".to_string();
        } else {
            let output = self.run_git_command(&["remote", "show", "origin"]);
            if let Ok(output_str) = output {
                let mut status = "not_exists".to_string();
                for line in output_str.lines() {
                    if line.contains(&release_git_branch) {
                        if line.contains("tracked") {
                            status = "tracked".to_string();
                            break;
                        } else if line.contains("new (next fetch will store in remotes/origin)") {
                            status = "new".to_string();
                            break;
                        } else if line.contains("stale (use 'git remote prune' to remove)") {
                            status = "stale".to_string();
                            break;
                        } else {
                            status = "unknown_status".to_string();
                            break;
                        }
                    }
                }

                if status == "not_exists" || status == "unknown_status" {
                    artifact.release_version = status;
                } else {
                    // Concatenar la dev_version con el estado
                    artifact.release_version = format!("{} {}", dev_version_str, status);
                }
            } else {
                artifact.release_version = "error_checking_remote".to_string();
            }
        }
        *completed_steps += 1;

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
        // Primero, intenta hacer checkout a la rama
        let checkout_result = self.run_git_command(&["checkout", branch_name]);
        if checkout_result.is_err() {
            // Si el checkout falla, podría ser que la rama no existe localmente
            // Intenta crearla y rastrearla desde el origen
            let create_and_track_result = self.run_git_command(&["checkout", "-b", branch_name, &format!("origin/{}", branch_name)]);
            if create_and_track_result.is_err() {
                // Si tampoco se puede crear/rastrear, la rama realmente no existe o hay otro error
                return Err(format!("Error al hacer checkout/crear la rama '{}': {}", branch_name, create_and_track_result.unwrap_err()));
            }
        }

        // Si el checkout fue exitoso (o la rama fue creada), procede con el pull
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

    fn get_version_from_package_json(&self, repo_root_path: &Path) -> Option<String> {
        let possible_paths = vec![
            repo_root_path.join("package.json"),
            repo_root_path.join("backend/package.json"),
            repo_root_path.join("app/package.json"),
            repo_root_path.join("frontend/package.json"),
            repo_root_path.join("client/package.json"),
        ];

        for path in possible_paths {
            // Componer la ruta completa usando `repo_root_path` y el subdirectorio si existe
            let full_path = if path.is_absolute() {
                path
            } else {
                // Si la ruta no es absoluta, asumimos que es relativa al directorio actual (`repo_path` ya está seteado)
                // En este caso, sólo necesitamos la parte final `package.json` o `backend/package.json`
                // y buscar desde el directorio actual.
                // Sin embargo, `repo_root_path` ya es el directorio actual después de `set_current_dir`.
                // Por lo tanto, `path` como se construye con `repo_root_path.join(...)` ya es la ruta correcta.
                path
            };

            if full_path.exists() {
                if let Ok(contents) = fs::read_to_string(&full_path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                        if let Some(version) = json["version"].as_str() {
                            return Some(version.to_string());
                        }
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
