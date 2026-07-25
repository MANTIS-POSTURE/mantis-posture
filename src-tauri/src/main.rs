use tauri::async_runtime;
use tauri::Manager;
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use std::time::Instant;
use rusqlite::{Connection, Result as SqlResult, NO_PARAMS};
use serde::Serialize;
use std::sync::Mutex;

// Import data models from lib.rs
mod lib;
use lib::{
    Folder, Identity, Exposure, Incident, Action, RgpdRequest, Database, initialize_database
};

// Define API response structures
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: u64,
}

// Database initialization
fn create_db_connection() -> SqlResult<Database> {
    let start = Instant::now();
    
    // Create app data directory if it doesn't exist
    let app_data_dir = dirs::data_dir()
        .expect("Could not find data directory")
        .join("mantis-posture");
    
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).expect("Could not create data directory");
    }
    
    // Connect to database or create it
    let db_path = app_data_dir.join("mantis.db");
    let conn = Connection::open(&db_path)?;
    
    // Initialize database if empty
    if is_database_empty(&conn)? {
        initialize_database(&conn)?;
    }
    
    let duration = start.elapsed().as_millis();
    println!("Database connection established in {}ms", duration);
    
    Ok(Database::new(conn))
}

fn is_database_empty(conn: &Connection) -> SqlResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT IN ('sqlite_sequence', 'app_settings')")?;
    let count: i64 = stmt.query_row(NO_PARAMS, |row| row.get(0))?;
    Ok(count == 0)
}

// Generic wrapper for database operations with error handling
fn with_db<F, T>(db: &Database, handler: F) -> ApiResponse<T>
where
    F: FnOnce(&Connection) -> SqlResult<T>,
    T: Serialize,
{
    match handler(&db.connection.lock().unwrap()) {
        Ok(data) => ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            timestamp: now(),
        },
        Err(e) => {
            eprintln!("Database error: {}", e);
            ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                timestamp: now(),
            }
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u64
}

// Tauri Commands
#[tauri::command]
fn get_folders(db: tauri::State<Database>) -> ApiResponse<Vec<Folder>> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, name, context, created_at, updated_at FROM folders ORDER BY name")?;
        let folders = stmt.query_map(NO_PARAMS, |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                context: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        
        let mut results = Vec::new();
        for folder in folders {
            results.push(folder?);
        }
        Ok(results)
    })
}

#[tauri::command]
fn create_folder(db: tauri::State<Database>, name: String, context: String) -> ApiResponse<Folder> {
    with_db(&db, |conn| {
        let folder_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO folders (id, name, context, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            (&folder_id, &name, &context, &now)
        )?;
        
        Ok(Folder {
            id: folder_id,
            name,
            context,
            created_at: now.clone(),
            updated_at: now,
        })
    })
}

#[tauri::command]
fn get_folder(db: tauri::State<Database>, folder_id: String) -> ApiResponse<Folder> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, name, context, created_at, updated_at FROM folders WHERE id = ?1")?;
        let mut folder_iter = stmt.query_map((&folder_id,), |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                context: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        
        if let Some(folder_result) = folder_iter.next() {
            folder_result.map_err(|e| e.into())
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    })
}

#[tauri::command]
fn get_identities(db: tauri::State<Database>) -> ApiResponse<Vec<Identity>> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, label, kind, value, folder_id, notes, created_at, updated_at FROM identities ORDER BY folder_id, label")?;
        let identities = stmt.query_map(NO_PARAMS, |row| {
            Ok(Identity {
                id: row.get(0)?,
                label: row.get(1)?,
                kind: row.get(2)?,
                value: row.get(3)?,
                folder_id: row.get(4)?,
                notes: row.get(5).ok(),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        
        let mut results = Vec::new();
        for identity in identities {
            results.push(identity?);
        }
        Ok(results)
    })
}

#[tauri::command]
fn create_identity(
    db: tauri::State<Database>,
    label: String,
    kind: String,
    value: String,
    folder_id: String,
    notes: Option<String>
) -> ApiResponse<Identity> {
    with_db(&db, |conn| {
        let identity_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO identities (id, label, kind, value, folder_id, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            (&identity_id, &label, &kind, &value, &folder_id, &notes, &now)
        )?;
        
        Ok(Identity {
            id: identity_id,
            label,
            kind,
            value,
            folder_id,
            notes,
            created_at: now.clone(),
            updated_at: now,
        })
    })
}

#[tauri::command]
fn get_exposures(db: tauri::State<Database>) -> ApiResponse<Vec<Exposure>> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, title, kind, severity, status, discovered_at, source, what, why, folder_id, created_at, updated_at FROM exposures ORDER BY discovered_at DESC")?;
        let exposures = stmt.query_map(NO_PARAMS, |row| {
            Ok(Exposure {
                id: row.get(0)?,
                title: row.get(1)?,
                kind: row.get(2)?,
                severity: row.get(3)?,
                status: row.get(4)?,
                discovered_at: row.get(5)?,
                source: row.get(6)?,
                what: row.get(7)?,
                why: row.get(8)?,
                folder_id: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;
        
        let mut results = Vec::new();
        for exposure in exposures {
            results.push(exposure?);
        }
        Ok(results)
    })
}

#[tauri::command]
fn get_incidents(db: tauri::State<Database>) -> ApiResponse<Vec<Incident>> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id, category_id, created_at, updated_at FROM incidents ORDER BY severity DESC")?;
        let incidents = stmt.query_map(NO_PARAMS, |row| {
            Ok(Incident {
                id: row.get(0)?,
                title: row.get(1)?,
                severity: row.get(2)?,
                discovered_at: row.get(3)?,
                what: row.get(4)?,
                why: row.get(5)?,
                impact: row.get(6)?,
                confidence: row.get(7)?,
                next_step: row.get(8)?,
                folder_id: row.get(9)?,
                category_id: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;
        
        let mut results = Vec::new();
        for incident in incidents {
            results.push(incident?);
        }
        Ok(results)
    })
}

#[tauri::command]
fn get_actions(db: tauri::State<Database>) -> ApiResponse<Vec<Action>> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id, created_at, updated_at FROM actions ORDER BY deadline")?;
        let actions = stmt.query_map(NO_PARAMS, |row| {
            Ok(Action {
                id: row.get(0)?,
                title: row.get(1)?,
                priority_id: row.get(2)?,
                difficulty_id: row.get(3)?,
                deadline: row.get(4)?,
                status: row.get(5)?,
                guidance: row.get(6)?,
                proof_expected: row.get(7)?,
                folder_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        
        let mut results = Vec::new();
        for action in actions {
            results.push(action?);
        }
        Ok(results)
    })
}

#[tauri::command]
fn get_rgpd_requests(db: tauri::State<Database>) -> ApiResponse<Vec<RgpdRequest>> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare("SELECT id, type_id, target, dpo_contact, status_id, data_summary, draft_preview, created_at, updated_at FROM rgpd_requests ORDER BY created_at DESC")?;
        let requests = stmt.query_map(NO_PARAMS, |row| {
            Ok(RgpdRequest {
                id: row.get(0)?,
                type_id: row.get(1)?,
                target: row.get(2)?,
                dpo_contact: row.get(3)?,
                status_id: row.get(4)?,
                data_summary: row.get(5)?,
                draft_preview: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        let mut results = Vec::new();
        for request in requests {
            results.push(request?);
        }
        Ok(results)
    })
}

// Main function
fn main() {
    let db = match create_db_connection() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to create database connection: {}", e);
            panic!("Database initialization failed");
        }
    };
    
    let db = Arc::new(db);
    
    tauri::Builder::default()
        .manage(db.clone())
        .invoke_handler(tauri::generate_handler![
            get_folders,
            create_folder,
            get_folder,
            get_identities,
            create_identity,
            get_exposures,
            get_incidents,
            get_actions,
            get_rgpd_requests
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
