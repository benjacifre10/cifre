use crate::domain::models::ArtifactDescDoc;
use anyhow::Result;
use std::path::Path;
use std::fs;

pub struct DescDocsService;

impl DescDocsService {
    pub fn generate_desc_docs_for_artifact(artifact_name: &str) -> Result<()> {
        // Generate description and purpose based on artifact name
        let (description, purpose) = Self::generate_description_for_artifact(artifact_name);
        
        // Load existing docs or create new
        let mut all_docs = Self::load_desc_docs().unwrap_or_default();
        
        // Remove existing entry for this artifact
        all_docs.retain(|doc| doc.name != artifact_name);
        
        // Add new entry
        all_docs.push(ArtifactDescDoc {
            id: artifact_name.to_string(),
            name: artifact_name.to_string(),
            description,
            purpose,
        });
        
        let json_content = serde_json::to_string_pretty(&all_docs)?;
        
        // Create data directory if it doesn't exist
        std::fs::create_dir_all("data")?;
        fs::write("data/artifact_docs_desc.json", json_content)?;
        
        Ok(())
    }
    
    fn generate_description_for_artifact(artifact_name: &str) -> (String, String) {
        match artifact_name {
            "bff-auth" => (
                "Backend For Frontend service that handles authentication flows and user session management.".to_string(),
                "Provides a unified authentication interface for frontend applications, managing login, logout, and session validation processes.".to_string()
            ),
            "auth-service" => (
                "Core authentication service responsible for user credential validation and token management.".to_string(),
                "Centralizes authentication logic, handles user login/logout, token generation, and credential verification across the platform.".to_string()
            ),
            "authority-service" => (
                "Authorization service that manages user permissions and access control policies.".to_string(),
                "Enforces security policies by controlling user access to resources based on roles and permissions within the system.".to_string()
            ),
            "bff-mfa" => (
                "Backend For Frontend service specialized in Multi-Factor Authentication flows.".to_string(),
                "Manages MFA processes including SMS, email, and authenticator app verification for enhanced security.".to_string()
            ),
            "mfa-auth-service" => (
                "Multi-Factor Authentication service that handles secondary authentication methods.".to_string(),
                "Provides secure MFA verification through various channels like SMS, email, and TOTP authenticators.".to_string()
            ),
            "audit-auth-service" => (
                "Audit service that logs and monitors authentication-related activities and security events.".to_string(),
                "Tracks authentication attempts, security events, and user activities for compliance and security monitoring.".to_string()
            ),
            "bff-face-recognition" => (
                "Backend For Frontend service that handles facial recognition authentication flows.".to_string(),
                "Manages biometric authentication using facial recognition technology for secure user identification.".to_string()
            ),
            "bff-unlock" => (
                "Backend For Frontend service that manages account unlock and recovery processes.".to_string(),
                "Handles user account unlock procedures, password recovery, and account restoration workflows.".to_string()
            ),
            "unlock-service" => (
                "Core service responsible for account unlock operations and user recovery processes.".to_string(),
                "Manages the backend logic for unlocking user accounts, handling recovery requests, and restoring access.".to_string()
            ),
            "authorizer-lambda" => (
                "AWS Lambda function that provides authorization decisions for API Gateway requests.".to_string(),
                "Acts as a custom authorizer for API Gateway, validating tokens and making authorization decisions for incoming requests.".to_string()
            ),
            "core-fraud-service" => (
                "Fraud detection service that analyzes transactions and user behavior for suspicious activities.".to_string(),
                "Implements fraud detection algorithms to identify and prevent fraudulent activities across the platform.".to_string()
            ),
            "mobile-app-react-native" => (
                "React Native mobile application providing user interface for authentication and banking services.".to_string(),
                "Delivers a cross-platform mobile experience for users to access banking services, authentication, and account management.".to_string()
            ),
            _ => (
                format!("Microservice {} that provides specific functionality within the authentication and banking ecosystem.", artifact_name),
                format!("Serves as a specialized component in the overall system architecture, handling specific business logic and operations for {}.", artifact_name)
            )
        }
    }
    
    pub fn load_desc_docs() -> Result<Vec<ArtifactDescDoc>> {
        if !Path::new("data/artifact_docs_desc.json").exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string("data/artifact_docs_desc.json")?;
        let docs: Vec<ArtifactDescDoc> = serde_json::from_str(&content)?;
        Ok(docs)
    }
}
