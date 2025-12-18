use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub state: String,
    pub priority: String,
    pub tag: String,
    pub creation_date: String,
    pub finish_date: String,
    pub alert: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Country {
    pub id: String,
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub name: String,
    pub country_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactType {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub artifact_type_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub artifact_id: String,
    pub country_id: String,
    pub stage_id: String,
    pub version: String,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub name: String,
    pub year: u32,
    pub date_init: String,
    pub date_qa: String,
    pub date_finish: String,
    pub artifacts: Vec<ReleaseArtifact>, // Vector de objetos ReleaseArtifact
}

// #[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct VersionsArtifact {
//     pub name: String,
//     pub dev_version: String,
//     pub qa_version: String,
//     pub beta_version: String,
//     pub prod_version: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionsArtifact {
    pub name: String,
    pub dev_version: String,
    pub release_version: String,
    pub prod_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactCountryStage {
    pub country_id: String,
    pub stages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDoc {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub countries: Vec<ArtifactCountryStage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpaStageData {
    pub stage: String,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub deploy_replicas: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpaCountryData {
    pub country: String,
    pub stages: Vec<HpaStageData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactHpaDoc {
    pub id: String,
    pub name: String,
    pub hpa: Vec<HpaCountryData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDescDoc {
    pub id: String,
    pub name: String,
    pub description: String,
    pub purpose: String,
}
