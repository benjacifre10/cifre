use crate::domain::models::{Country, Stage, Artifact, ArtifactType, Release};
use anyhow::{Result, Context};
use std::{fs, path::PathBuf};
use serde::{de::DeserializeOwned, Serialize, Deserialize};

pub struct JsonRepository<T> {
    file_path: PathBuf,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Default> JsonRepository<T> {
    pub fn new(file_name: &str) -> Self {
        JsonRepository {
            file_path: PathBuf::from(file_name),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn load(&self) -> Result<T> {
        if !self.file_path.exists() {
            let default_data = T::default();
            self.save(&default_data)?;
            return Ok(default_data);
        }
        let data = fs::read_to_string(&self.file_path)
            .context(format!("Failed to read file from {:?}", self.file_path))?;
        serde_json::from_str(&data)
            .context("Failed to deserialize data from JSON")
    }

    pub fn save(&self, data: &T) -> Result<()> {
        let json_string = serde_json::to_string_pretty(data)
            .context("Failed to serialize data to JSON")?;
        fs::write(&self.file_path, json_string)
            .context(format!("Failed to write data to {:?}", self.file_path))?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountryData {
    pub countries: Vec<Country>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageData {
    pub stages: Vec<Stage>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTypeData {
    pub artifact_types: Vec<ArtifactType>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactData {
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseData {
    pub releases: Vec<Release>,
}

pub struct CountryJsonRepository {
    json_repo: JsonRepository<CountryData>,
}

pub struct StageJsonRepository {
    json_repo: JsonRepository<StageData>,
}

pub struct ArtifactTypeJsonRepository {
    json_repo: JsonRepository<ArtifactTypeData>,
}

pub struct ArtifactJsonRepository {
    json_repo: JsonRepository<ArtifactData>,
}

pub struct ReleaseJsonRepository {
    json_repo: JsonRepository<ReleaseData>,
}

impl CountryJsonRepository {
    pub fn new() -> Self {
        CountryJsonRepository {
            json_repo: JsonRepository::new("data/country.json"),
        }
    }

    pub fn get_all_countries(&self) -> Result<Vec<Country>> {
        let data = self.json_repo.load()?;
        Ok(data.countries)
    }

    pub fn save_countries(&self, countries: &[Country]) -> Result<()> {
        let data = CountryData { countries: countries.to_vec() };
        self.json_repo.save(&data)
    }

    pub fn add_country(&self, name: String) -> Result<Country> {
        let mut countries = self.get_all_countries()?;
        let new_country = Country {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            code: String::new(),
        };
        countries.push(new_country.clone());
        self.save_countries(&countries)?;
        Ok(new_country)
    }

    pub fn update_country(&self, id: &str, name: String) -> Result<()> {
        let mut countries = self.get_all_countries()?;
        if let Some(country) = countries.iter_mut().find(|c| c.id == id) {
            country.name = name;
            self.save_countries(&countries)?;
        }
        Ok(())
    }

    pub fn delete_country(&self, id: &str) -> Result<()> {
        let mut countries = self.get_all_countries()?;
        countries.retain(|c| c.id != id);
        self.save_countries(&countries)?;
        Ok(())
    }
}

impl StageJsonRepository {
    pub fn new() -> Self {
        StageJsonRepository {
            json_repo: JsonRepository::new("data/stage.json"),
        }
    }

    pub fn get_all_stages(&self) -> Result<Vec<Stage>> {
        let data = self.json_repo.load()?;
        Ok(data.stages)
    }

    pub fn save_stages(&self, stages: &[Stage]) -> Result<()> {
        let data = StageData { stages: stages.to_vec() };
        self.json_repo.save(&data)
    }

    pub fn add_stage(&self, name: String, country_id: String) -> Result<Stage> {
        let mut stages = self.get_all_stages()?;
        let new_stage = Stage {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            country_id,
        };
        stages.push(new_stage.clone());
        self.save_stages(&stages)?;
        Ok(new_stage)
    }

    pub fn update_stage(&self, id: &str, name: String, country_id: String) -> Result<()> {
        let mut stages = self.get_all_stages()?;
        if let Some(stage) = stages.iter_mut().find(|s| s.id == id) {
            stage.name = name;
            stage.country_id = country_id;
            self.save_stages(&stages)?;
        }
        Ok(())
    }

    pub fn delete_stage(&self, id: &str) -> Result<()> {
        let mut stages = self.get_all_stages()?;
        stages.retain(|s| s.id != id);
        self.save_stages(&stages)?;
        Ok(())
    }
}

impl ArtifactTypeJsonRepository {
    pub fn new() -> Self {
        let repo = ArtifactTypeJsonRepository {
            json_repo: JsonRepository::new("data/artifact_type.json"),
        };
        let _ = repo.initialize_default_types();
        repo
    }

    fn initialize_default_types(&self) -> Result<()> {
        match self.get_all_artifact_types() {
            Ok(types) => {
                if types.is_empty() {
                    let default_types = vec![
                        ArtifactType { id: 1, name: "ms".to_string() },
                        ArtifactType { id: 2, name: "lambda".to_string() },
                        ArtifactType { id: 3, name: "db".to_string() },
                        ArtifactType { id: 4, name: "mobile".to_string() },
                        ArtifactType { id: 5, name: "bff".to_string() },
                    ];
                    self.save_artifact_types(&default_types)?;
                }
                Ok(())
            }
            Err(_) => {
                // Si hay error cargando, crear los tipos por defecto
                let default_types = vec![
                    ArtifactType { id: 1, name: "ms".to_string() },
                    ArtifactType { id: 2, name: "lambda".to_string() },
                    ArtifactType { id: 3, name: "db".to_string() },
                    ArtifactType { id: 4, name: "mobile".to_string() },
                    ArtifactType { id: 5, name: "bff".to_string() },
                ];
                self.save_artifact_types(&default_types)
            }
        }
    }

    pub fn get_all_artifact_types(&self) -> Result<Vec<ArtifactType>> {
        let data = self.json_repo.load()?;
        Ok(data.artifact_types)
    }

    pub fn save_artifact_types(&self, artifact_types: &[ArtifactType]) -> Result<()> {
        let data = ArtifactTypeData { artifact_types: artifact_types.to_vec() };
        self.json_repo.save(&data)
    }
}

impl ReleaseJsonRepository {
    pub fn new() -> Self {
        ReleaseJsonRepository {
            json_repo: JsonRepository::new("data/release.json"),
        }
    }

    pub fn get_all_releases(&self) -> Result<Vec<Release>> {
        let data = self.json_repo.load()?;
        Ok(data.releases)
    }

    pub fn save_releases(&self, releases: &[Release]) -> Result<()> {
        let data = ReleaseData { releases: releases.to_vec() };
        self.json_repo.save(&data)
    }

    pub fn add_release(&self, name: String, year: u32, date_init: String, date_qa: String, date_finish: String) -> Result<Release> {
        let mut releases = self.get_all_releases()?;
        let new_release = Release {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            year,
            date_init,
            date_qa,
            date_finish,
            artifacts: Vec::new(), // Vector vacío de artifacts
        };
        releases.push(new_release.clone());
        self.save_releases(&releases)?;
        Ok(new_release)
    }

    pub fn update_release(&self, id: &str, name: String, year: u32, date_init: String, date_qa: String, date_finish: String) -> Result<()> {
        let mut releases = self.get_all_releases()?;
        if let Some(release) = releases.iter_mut().find(|r| r.id == id) {
            release.name = name;
            release.year = year;
            release.date_init = date_init;
            release.date_qa = date_qa;
            release.date_finish = date_finish;
            self.save_releases(&releases)?;
        }
        Ok(())
    }

    pub fn update_release_complete(&self, updated_release: &Release) -> Result<()> {
        let mut releases = self.get_all_releases()?;
        if let Some(release) = releases.iter_mut().find(|r| r.id == updated_release.id) {
            *release = updated_release.clone();
            self.save_releases(&releases)?;
        }
        Ok(())
    }

    pub fn delete_release(&self, id: &str) -> Result<()> {
        let mut releases = self.get_all_releases()?;
        releases.retain(|r| r.id != id);
        self.save_releases(&releases)?;
        Ok(())
    }
}

impl ArtifactJsonRepository {
    pub fn new() -> Self {
        ArtifactJsonRepository {
            json_repo: JsonRepository::new("data/artifact.json"),
        }
    }

    pub fn get_all_artifacts(&self) -> Result<Vec<Artifact>> {
        let data = self.json_repo.load()?;
        Ok(data.artifacts)
    }

    pub fn save_artifacts(&self, artifacts: &[Artifact]) -> Result<()> {
        let data = ArtifactData { artifacts: artifacts.to_vec() };
        self.json_repo.save(&data)
    }

    pub fn add_artifact(&self, name: String, artifact_type_id: u32) -> Result<Artifact> {
        let mut artifacts = self.get_all_artifacts()?;
        let new_artifact = Artifact {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            artifact_type_id,
        };
        artifacts.push(new_artifact.clone());
        self.save_artifacts(&artifacts)?;
        Ok(new_artifact)
    }

    pub fn update_artifact(&self, id: &str, name: String, artifact_type_id: u32) -> Result<()> {
        let mut artifacts = self.get_all_artifacts()?;
        if let Some(artifact) = artifacts.iter_mut().find(|a| a.id == id) {
            artifact.name = name;
            artifact.artifact_type_id = artifact_type_id;
            self.save_artifacts(&artifacts)?;
        }
        Ok(())
    }

    pub fn delete_artifact(&self, id: &str) -> Result<()> {
        let mut artifacts = self.get_all_artifacts()?;
        artifacts.retain(|a| a.id != id);
        self.save_artifacts(&artifacts)?;
        Ok(())
    }
}
