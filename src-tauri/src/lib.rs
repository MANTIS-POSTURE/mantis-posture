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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![list_folders, list_incidents, list_actions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
