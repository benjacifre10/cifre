use crate::domain::models::{ArtifactDoc, ArtifactCountryStage, VersionsArtifact, Country, Stage};
use std::{fs, path::PathBuf, collections::HashMap};
use serde_json;

pub struct ArtifactsDocsService {
    pub configs_repo_path: PathBuf,
    pub output_file: PathBuf,
}

impl ArtifactsDocsService {
    pub fn new(configs_repo_path: &str) -> Self {
        Self {
            configs_repo_path: PathBuf::from(configs_repo_path),
            output_file: PathBuf::from("data/artifact_docs.json"),
        }
    }

    fn extract_namespace_from_yaml(&self, yaml_path: &PathBuf) -> Option<String> {
        if let Ok(content) = fs::read_to_string(yaml_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("namespace:") {
                    if let Some(ns) = trimmed.split(':').nth(1) {
                        return Some(ns.trim().trim_matches('"').to_string());
                    }
                }
            }
        }
        None
    }

    fn find_namespace(&self, country_code: &str, stage_name: &str, namespace_folder: &str, subpath: &str) -> String {
        let base_path = self.configs_repo_path
            .join(country_code)
            .join(stage_name)
            .join(namespace_folder);
        
        if !subpath.is_empty() {
            // Buscar en manifest/deployment.yaml
            let manifest_path = base_path.join(subpath).join("manifest").join("deployment.yaml");
            if let Some(ns) = self.extract_namespace_from_yaml(&manifest_path) {
                return ns;
            }
            
            // Buscar en deployment.yaml directo
            let deployment_path = base_path.join(subpath).join("deployment.yaml");
            if let Some(ns) = self.extract_namespace_from_yaml(&deployment_path) {
                return ns;
            }
        }
        
        // Buscar en manifest/deployment.yaml en base
        let manifest_path = base_path.join("manifest").join("deployment.yaml");
        if let Some(ns) = self.extract_namespace_from_yaml(&manifest_path) {
            return ns;
        }
        
        // Buscar en deployment.yaml directo en base
        let deployment_path = base_path.join("deployment.yaml");
        if let Some(ns) = self.extract_namespace_from_yaml(&deployment_path) {
            return ns;
        }
        
        namespace_folder.to_string()
    }

    pub fn generate_artifacts_docs_with_progress<F>(
        &self,
        versions: &[VersionsArtifact],
        countries: &[Country],
        stages: &[Stage],
        progress_callback: F,
    ) -> Result<Vec<ArtifactDoc>, String>
    where
        F: Fn(usize, usize, String) + Send + 'static,
    {
        let mut artifact_docs = Vec::new();

        let artifact_locations: HashMap<&str, (&str, &str)> = [
            ("bff-auth", ("auth", "microservicios/bff-auth")),
            ("bff-mfa", ("auth", "microservicios/bff-mfa")),
            ("bff-unlock", ("auth", "microservicios/bff-unlock")),
            ("bff-face-recognition", ("auth", "microservicios/bff-face-recognition")),
            ("unlock-service", ("auth", "microservicios/unlock-service")),
            ("bussines-flow-authenticator", ("auth", "microservicios/bussines-flow-authenticator")),
            ("authority-service", ("auth", "microservicios/authority-service")),
            ("audit-auth-service", ("auth", "microservicios/audit-auth-service")),
            ("mfa-auth-service", ("auth", "microservicios/mfa-auth-service")),
            ("auth-service", ("auth", "microservicios/auth-service")),
            ("authorizer-lambda", ("auth", "microservicios")),
            ("core-fraud-service", ("fraude", "microservicios/core-fraud-service")),
            ("mobile-app-react-native", ("mobile", "")),
        ].iter().cloned().collect();

        let total = versions.iter().filter(|v| artifact_locations.contains_key(v.name.as_str())).count();
        let mut current = 0;

        for version in versions {
            if let Some((namespace_folder, subpath)) = artifact_locations.get(version.name.as_str()) {
                current += 1;
                progress_callback(current, total, format!("Procesando: {}", version.name));

                let mut countries_data = Vec::new();
                let mut namespace_value = String::new();

                for country in countries {
                    let country_path = self.configs_repo_path.join(&country.code);
                    if !country_path.exists() {
                        continue;
                    }

                    let mut stage_ids = Vec::new();

                    for stage in stages {
                        let stage_folder = stage.name.to_lowercase();
                        let stage_path = country_path.join(&stage_folder).join(namespace_folder);
                        if stage_path.exists() {
                            stage_ids.push(stage.id.clone());
                            
                            if namespace_value.is_empty() {
                                namespace_value = self.find_namespace(&country.code, &stage_folder, namespace_folder, subpath);
                            }
                        }
                    }

                    if !stage_ids.is_empty() {
                        countries_data.push(ArtifactCountryStage {
                            country_id: country.id.clone(),
                            stages: stage_ids,
                        });
                    }
                }

                if !countries_data.is_empty() {
                    artifact_docs.push(ArtifactDoc {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: version.name.clone(),
                        namespace: namespace_value,
                        countries: countries_data,
                    });
                }
            }
        }

        progress_callback(total, total, "Guardando...".to_string());
        self.save_to_json(&artifact_docs)?;
        Ok(artifact_docs)
    }

    #[allow(dead_code)]
    pub fn generate_artifacts_docs(
        &self,
        versions: &[VersionsArtifact],
        countries: &[Country],
        stages: &[Stage],
    ) -> Result<Vec<ArtifactDoc>, String> {
        self.generate_artifacts_docs_with_progress(versions, countries, stages, |_, _, _| {})
    }

    pub fn load_from_json(&self) -> Result<Vec<ArtifactDoc>, String> {
        if !self.output_file.exists() {
            return Err("Archivo artifact_docs.json no existe".to_string());
        }

        let content = fs::read_to_string(&self.output_file)
            .map_err(|e| format!("Error al leer archivo: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Error al deserializar: {}", e))
    }

    fn save_to_json(&self, docs: &[ArtifactDoc]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(docs)
            .map_err(|e| format!("Error al serializar: {}", e))?;

        fs::write(&self.output_file, json)
            .map_err(|e| format!("Error al escribir archivo: {}", e))?;

        Ok(())
    }
}
