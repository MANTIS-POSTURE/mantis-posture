use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use rusqlite::Connection;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub context: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identity {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub value: String,
    pub folder_id: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exposure {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub severity: String,
    pub status: String,
    pub discovered_at: String,
    pub source: String,
    pub what: String,
    pub why: String,
    pub folder_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncidentCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub discovered_at: String,
    pub what: String,
    pub why: String,
    pub impact: String,
    pub confidence: String,
    pub next_step: String,
    pub folder_id: String,
    pub category_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionMetadata {
    pub id: String,
    pub r#type: String, // type is a reserved keyword
    pub value: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub priority_id: String,
    pub difficulty_id: String,
    pub deadline: String,
    pub status: String,
    pub guidance: String, // JSON array
    pub proof_expected: String,
    pub folder_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RgpdType {
    pub id: String,
    pub name: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RgpdStatus {
    pub id: String,
    pub name: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RgpdRequest {
    pub id: String,
    pub type_id: String,
    pub target: String,
    pub dpo_contact: String,
    pub status_id: String,
    pub data_summary: String,
    pub draft_preview: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExposureIncident {
    pub exposure_id: String,
    pub incident_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncidentAction {
    pub incident_id: String,
    pub action_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionRgpd {
    pub action_id: String,
    pub rgpd_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncidentRgpd {
    pub incident_id: String,
    pub rgpd_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderIdentity {
    pub folder_id: String,
    pub identity_id: String,
}

// Database schema management
pub fn initialize_database(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Create tables
    conn.execute_batch(include_str!("../migrations/0001_init.sql"))?;
    
    // Initialize default metadata if needed
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
        ("db_version", "1")
    )?;
    
    Ok(())
}

// Connection management
pub struct Database {
    pub connection: Mutex<Connection>,
}

impl Database {
    pub fn new(connection: Connection) -> Self {
        Database {
            connection: Mutex::new(connection),
        }
    }
    
    // Add any common helper methods here
}
