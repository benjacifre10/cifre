use crate::domain::models::{ArtifactHpaDoc, HpaCountryData, HpaStageData, Country};
use anyhow::Result;
use std::path::Path;
use std::fs;
use serde_yaml::Value;

pub struct HpaDocsService;

impl HpaDocsService {
    pub fn generate_hpa_docs_for_artifact(artifact_name: &str) -> Result<()> {
        let config_repo_path = "/Users/u631568/Documents/Development/Teco/Others/configurations-region";
        
        let countries = Self::load_countries()?;
        let stages = vec!["dev".to_string(), "qa".to_string(), "beta".to_string(), "prod".to_string()];
        
        let mut hpa_countries = Vec::new();
        
        for country in &countries {
            let mut country_stages = Vec::new();
            
            for stage in &stages {
                let artifact_path = format!("{}/{}/{}/auth/microservicios/{}/manifest", 
                    config_repo_path, country.code, stage, artifact_name);
                if Path::new(&artifact_path).exists() {
                    if let Ok(stage_data) = Self::extract_stage_data(&artifact_path, stage) {
                        country_stages.push(stage_data);
                    }
                }
            }
            
            if !country_stages.is_empty() {
                hpa_countries.push(HpaCountryData {
                    country: country.code.clone(),
                    stages: country_stages,
                });
            }
        }
        
        // Load existing docs or create new
        let mut all_docs = Self::load_hpa_docs().unwrap_or_default();
        
        // Remove existing entry for this artifact
        all_docs.retain(|doc| doc.name != artifact_name);
        
        // Add new entry if we have data
        if !hpa_countries.is_empty() {
            all_docs.push(ArtifactHpaDoc {
                id: artifact_name.to_string(),
                name: artifact_name.to_string(),
                hpa: hpa_countries,
            });
        }
        
        let json_content = serde_json::to_string_pretty(&all_docs)?;
        
        // Create data directory if it doesn't exist
        std::fs::create_dir_all("data")?;
        fs::write("data/artifact_docs_hpa.json", json_content)?;
        
        Ok(())
    }
    
    fn load_countries() -> Result<Vec<Country>> {
        let countries = vec![
            Country {
                id: "1".to_string(),
                name: "Argentina".to_string(),
                code: "arg".to_string(),
            },
            Country {
                id: "2".to_string(),
                name: "Paraguay".to_string(),
                code: "py".to_string(),
            },
        ];
        Ok(countries)
    }
    
    fn extract_stage_data(artifact_path: &str, stage: &str) -> Result<HpaStageData> {
        let hpa_path = format!("{}/hpa.yaml", artifact_path);
        let deployment_path = format!("{}/deployment.yaml", artifact_path);
        
        let mut min_replicas = 1;
        let mut max_replicas = 1;
        let mut deploy_replicas = 1;
        
        // Extract from hpa.yaml
        if Path::new(&hpa_path).exists() {
            if let Ok(content) = fs::read_to_string(&hpa_path) {
                if let Ok(yaml) = serde_yaml::from_str::<Value>(&content) {
                    if let Some(spec) = yaml.get("spec") {
                        if let Some(min) = spec.get("minReplicas") {
                            if let Some(min_val) = min.as_u64() {
                                min_replicas = min_val as u32;
                            }
                        }
                        if let Some(max) = spec.get("maxReplicas") {
                            if let Some(max_val) = max.as_u64() {
                                max_replicas = max_val as u32;
                            }
                        }
                    }
                }
            }
        }
        
        // Extract from deployment.yaml
        if Path::new(&deployment_path).exists() {
            if let Ok(content) = fs::read_to_string(&deployment_path) {
                if let Ok(yaml) = serde_yaml::from_str::<Value>(&content) {
                    if let Some(spec) = yaml.get("spec") {
                        if let Some(replicas) = spec.get("replicas") {
                            if let Some(replicas_val) = replicas.as_u64() {
                                deploy_replicas = replicas_val as u32;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(HpaStageData {
            stage: stage.to_string(),
            min_replicas,
            max_replicas,
            deploy_replicas,
        })
    }
    
    pub fn load_hpa_docs() -> Result<Vec<ArtifactHpaDoc>> {
        if !Path::new("data/artifact_docs_hpa.json").exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string("data/artifact_docs_hpa.json")?;
        let docs: Vec<ArtifactHpaDoc> = serde_json::from_str(&content)?;
        Ok(docs)
    }
}
