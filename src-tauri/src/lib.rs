use rusqlite::Connection;
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct Folder {
    id: i64,
    name: String,
    description: String,
}

#[tauri::command]
fn list_folders(app: tauri::AppHandle) -> Result<Vec<Folder>, String> {
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

    // Insérer un dossier fictif si la table est vide
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if count == 0 {
        conn.execute(
            "INSERT INTO folders (name, description) VALUES (?1, ?2)",
            ["Personnel", "Dossier personnel principal"]
        ).map_err(|e| e.to_string())?;
    }

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![list_folders])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
