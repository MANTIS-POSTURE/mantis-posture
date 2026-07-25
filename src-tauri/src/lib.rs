use rusqlite::{Connection, params};
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct Folder {
    id: i64,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Incident {
    id: i64,
    title: String,
    severity: String,
    status: String,
    description: String,
}

#[derive(Serialize)]
struct Action {
    id: i64,
    title: String,
    priority: String,
    status: String,
    incident_id: Option<i64>,
}

#[derive(Serialize)]
struct Identity {
    id: i64,
    folder_id: Option<i64>,
    identity_type: String,
    value: String,
    label: Option<String>,
}

#[derive(Serialize)]
struct Exposure {
    id: i64,
    identity_id: Option<i64>,
    source: String,
    severity: String,
    description: String,
    detected_at: String,
}

fn init_database(app: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let db_path = app_dir.join("mantis.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS incidents (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            severity TEXT NOT NULL,
            status TEXT NOT NULL,
            description TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS actions (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            priority TEXT NOT NULL,
            status TEXT NOT NULL,
            incident_id INTEGER,
            FOREIGN KEY(incident_id) REFERENCES incidents(id)
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS identities (
            id INTEGER PRIMARY KEY,
            folder_id INTEGER,
            identity_type TEXT NOT NULL,
            value TEXT NOT NULL,
            label TEXT,
            FOREIGN KEY(folder_id) REFERENCES folders(id)
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS exposures (
            id INTEGER PRIMARY KEY,
            identity_id INTEGER,
            source TEXT NOT NULL,
            severity TEXT NOT NULL,
            description TEXT,
            detected_at TEXT NOT NULL,
            FOREIGN KEY(identity_id) REFERENCES identities(id)
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Seed data if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if count == 0 {
        conn.execute(
            "INSERT INTO folders (name, description) VALUES (?1, ?2)",
            params!["Personnel", "Dossier personnel principal"]
        ).map_err(|e| e.to_string())?;
    }

    let inc_count: i64 = conn.query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if inc_count == 0 {
        conn.execute(
            "INSERT INTO incidents (title, severity, status, description) VALUES (?1, ?2, ?3, ?4)",
            params!["Fuite d'email détectée", "Élevée", "Ouvert", "Adresse email trouvée dans une base de données publique."]
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO actions (title, priority, status, incident_id) VALUES (?1, ?2, ?3, ?4)",
            params!["Changer le mot de passe", "Haute", "À faire", 1]
        ).map_err(|e| e.to_string())?;
    }

    let id_count: i64 = conn.query_row("SELECT COUNT(*) FROM identities", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if id_count == 0 {
        conn.execute(
            "INSERT INTO identities (folder_id, identity_type, value, label) VALUES (?1, ?2, ?3, ?4)",
            params![1, "email", "jean.dupont@example.com", "Email principal"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO identities (folder_id, identity_type, value, label) VALUES (?1, ?2, ?3, ?4)",
            params![1, "pseudo", "jdupont", "Pseudo générique"]
        ).map_err(|e| e.to_string())?;
    }

    let exp_count: i64 = conn.query_row("SELECT COUNT(*) FROM exposures", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if exp_count == 0 {
        conn.execute(
            "INSERT INTO exposures (identity_id, source, severity, description, detected_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![1, "Collection #1", "Élevée", "Email et mot de passe haché exposés.", "2023-10-15T12:00:00Z"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO exposures (identity_id, source, severity, description, detected_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![2, "Forum public", "Faible", "Pseudo lié à un profil sur un forum public.", "2024-01-20T08:30:00Z"]
        ).map_err(|e| e.to_string())?;
    }

    Ok(conn)
}

#[tauri::command]
fn list_folders(app: tauri::AppHandle) -> Result<Vec<Folder>, String> {
    let conn = init_database(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, name, description FROM folders").map_err(|e| e.to_string())?;
    let folder_iter = stmt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder.map_err(|e| e.to_string())?);
    }

    Ok(folders)
}

#[tauri::command]
fn list_incidents(app: tauri::AppHandle) -> Result<Vec<Incident>, String> {
    let conn = init_database(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, title, severity, status, description FROM incidents").map_err(|e| e.to_string())?;
    let incident_iter = stmt.query_map([], |row| {
        Ok(Incident {
            id: row.get(0)?,
            title: row.get(1)?,
            severity: row.get(2)?,
            status: row.get(3)?,
            description: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut incidents = Vec::new();
    for incident in incident_iter {
        incidents.push(incident.map_err(|e| e.to_string())?);
    }

    Ok(incidents)
}

#[tauri::command]
fn list_actions(app: tauri::AppHandle) -> Result<Vec<Action>, String> {
    let conn = init_database(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, title, priority, status, incident_id FROM actions").map_err(|e| e.to_string())?;
    let action_iter = stmt.query_map([], |row| {
        Ok(Action {
            id: row.get(0)?,
            title: row.get(1)?,
            priority: row.get(2)?,
            status: row.get(3)?,
            incident_id: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut actions = Vec::new();
    for action in action_iter {
        actions.push(action.map_err(|e| e.to_string())?);
    }

    Ok(actions)
}

#[tauri::command]
fn list_identities(app: tauri::AppHandle) -> Result<Vec<Identity>, String> {
    let conn = init_database(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, folder_id, identity_type, value, label FROM identities").map_err(|e| e.to_string())?;
    let identity_iter = stmt.query_map([], |row| {
        Ok(Identity {
            id: row.get(0)?,
            folder_id: row.get(1)?,
            identity_type: row.get(2)?,
            value: row.get(3)?,
            label: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut identities = Vec::new();
    for identity in identity_iter {
        identities.push(identity.map_err(|e| e.to_string())?);
    }

    Ok(identities)
}

#[tauri::command]
fn list_exposures(app: tauri::AppHandle) -> Result<Vec<Exposure>, String> {
    let conn = init_database(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, identity_id, source, severity, description, detected_at FROM exposures").map_err(|e| e.to_string())?;
    let exposure_iter = stmt.query_map([], |row| {
        Ok(Exposure {
            id: row.get(0)?,
            identity_id: row.get(1)?,
            source: row.get(2)?,
            severity: row.get(3)?,
            description: row.get(4)?,
            detected_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut exposures = Vec::new();
    for exposure in exposure_iter {
        exposures.push(exposure.map_err(|e| e.to_string())?);
    }

    Ok(exposures)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![list_folders, list_incidents, list_actions, list_identities, list_exposures])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
