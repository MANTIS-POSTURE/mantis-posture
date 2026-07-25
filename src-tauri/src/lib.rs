use rusqlite::{Connection, params};
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct Folder {
    id: String,
    name: String,
    context: String,
}

#[derive(Serialize)]
struct Incident {
    id: String,
    title: String,
    severity: String,
    discovered_at: String,
    what: String,
    why: String,
    impact: String,
    confidence: String,
    next_step: String,
    folder_id: Option<String>,
}

#[derive(Serialize)]
struct Action {
    id: String,
    title: String,
    priority_id: String,
    difficulty_id: String,
    deadline: String,
    status: String,
    guidance: String,
    proof_expected: String,
    folder_id: Option<String>,
    incident_id: Option<String>,
}

#[derive(Serialize)]
struct Identity {
    id: String,
    label: String,
    kind: String,
    value: String,
    folder_id: Option<String>,
    notes: Option<String>,
}

#[derive(Serialize)]
struct Exposure {
    id: String,
    title: String,
    kind: String,
    severity: String,
    status: String,
    discovered_at: String,
    source: String,
    what: String,
    why: String,
    folder_id: Option<String>,
}

#[derive(Serialize)]
struct RgpdRequest {
    id: String,
    type_id: String,
    target: String,
    dpo_contact: String,
    status_id: String,
    data_summary: String,
    draft_preview: String,
}

#[derive(Serialize)]
struct TimelineEntry {
    id: String,
    event_type: String,
    description: String,
    created_at: String,
}

#[derive(Serialize)]
struct PostureScore {
    score: i32,
    open_incidents: i32,
    high_exposures: i32,
    completed_actions: i32,
}

const MIGRATION_SQL: &str = include_str!("../migrations/0001_init.sql");

fn init_database(app: &tauri::AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let db_path = app_dir.join("mantis.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;

    // Check schema version
    let user_version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap_or(0);
    
    if user_version < 1 {
        // Drop all existing tables to ensure clean state for this migration
        conn.execute_batch("
            DROP TABLE IF EXISTS folder_identity;
            DROP TABLE IF EXISTS incident_rgpd;
            DROP TABLE IF EXISTS action_rgpd;
            DROP TABLE IF EXISTS incident_action;
            DROP TABLE IF EXISTS exposure_incident;
            DROP TABLE IF EXISTS timeline_entries;
            DROP TABLE IF EXISTS rgpd_requests;
            DROP TABLE IF EXISTS rgpd_statuses;
            DROP TABLE IF EXISTS rgpd_types;
            DROP TABLE IF EXISTS actions;
            DROP TABLE IF EXISTS action_metadata;
            DROP TABLE IF EXISTS incidents;
            DROP TABLE IF EXISTS incident_categories;
            DROP TABLE IF EXISTS exposures;
            DROP TABLE IF EXISTS identities;
            DROP TABLE IF EXISTS folders;
            DROP TABLE IF EXISTS app_settings;
        ").map_err(|e| e.to_string())?;

        // Apply migrations
        conn.execute_batch(MIGRATION_SQL).map_err(|e| e.to_string())?;
        
        // Update schema version
        conn.execute("PRAGMA user_version = 1", []).map_err(|e| e.to_string())?;
    }

    // Seed data if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if count == 0 {
        // Seed reference data
        conn.execute_batch("
            INSERT OR IGNORE INTO action_metadata (id, type, value, label) VALUES
            ('prio_001', 'priority', 'basse', 'Basse'),
            ('prio_002', 'priority', 'moyenne', 'Moyenne'),
            ('prio_003', 'priority', 'haute', 'Haute'),
            ('prio_004', 'priority', 'critique', 'Critique');

            INSERT OR IGNORE INTO action_metadata (id, type, value, label) VALUES
            ('diff_001', 'difficulty', 'facile', 'Facile'),
            ('diff_002', 'difficulty', 'moyenne', 'Moyenne'),
            ('diff_003', 'difficulty', 'difficile', 'Difficile');

            INSERT OR IGNORE INTO rgpd_types (id, name, label) VALUES
            ('type_001', 'acces', 'Accès'),
            ('type_002', 'rectification', 'Rectification'),
            ('type_003', 'effacement', 'Effacement'),
            ('type_004', 'opposition', 'Opposition'),
            ('type_005', 'dereferencement', 'Déréférencement');

            INSERT OR IGNORE INTO rgpd_statuses (id, name, label) VALUES
            ('status_001', 'brouillon', 'Brouillon'),
            ('status_002', 'prete', 'Prête à envoyer'),
            ('status_003', 'envoyee', 'Envoyée'),
            ('status_004', 'repondue', 'Répondue');
        ").map_err(|e| e.to_string())?;

        // Seed Folders
        conn.execute(
            "INSERT OR IGNORE INTO folders (id, name, context) VALUES (?1, ?2, ?3)",
            params!["folder-perso", "Personnel", "Identités et traces hors travail"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO folders (id, name, context) VALUES (?1, ?2, ?3)",
            params!["folder-job", "Emploi actuel", "Présence professionnelle et comptes liés au travail"]
        ).map_err(|e| e.to_string())?;

        // Seed Identities
        conn.execute(
            "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["id-name", "Nom complet", "nom", "Alex Martin", "folder-perso", ""]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["id-email-perso", "E-mail personnel", "email", "alex.martin.perso@example.com", "folder-perso", "Utilisé pour comptes grand public"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["id-email-pro", "E-mail professionnel", "email", "a.martin@entreprise-exemple.example", "folder-job", ""]
        ).map_err(|e| e.to_string())?;

        // Seed Exposures
        conn.execute(
            "INSERT OR IGNORE INTO exposures (id, title, kind, severity, status, discovered_at, source, what, why, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params!["exp-linkedin-email", "E-mail pro sur profil LinkedIn", "profil_public", "modérée", "en_suivi", "2026-07-20", "Profil public", "L'adresse e-mail professionnelle apparaît dans la section contact du profil.", "Facilite phishing ciblé et corrélation d'identités.", "folder-job"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO exposures (id, title, kind, severity, status, discovered_at, source, what, why, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params!["exp-leak-2019", "Occurrence e-mail — fuite 2019", "fuite", "élevée", "en_suivi", "2026-07-18", "Breach intelligence", "E-mail personnel signalé dans une fuite datée 2019.", "Risque si réutilisation d'identifiants sur des comptes encore actifs.", "folder-perso"]
        ).map_err(|e| e.to_string())?;

        // Seed Incidents
        conn.execute(
            "INSERT OR IGNORE INTO incidents (id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params!["inc-linkedin", "E-mail professionnel visible sur LinkedIn", "modérée", "2026-07-20", "Le profil LinkedIn affiche l'adresse e-mail professionnelle en clair.", "Cette adresse facilite le phishing ciblé, le scraping et la corrélation avec d'autres traces publiques.", "Phishing, usurpation légère, augmentation du volume de spam ciblé.", "Élevée — observation directe du profil public.", "Masquer ou retirer l'e-mail du profil, puis vérifier la page publique.", "folder-job"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO incidents (id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params!["inc-leaks", "Fuites anciennes d'identifiants", "élevée", "2026-07-18", "Deux occurrences d'un e-mail personnel dans des bases de fuites datées (2019 et 2021).", "Si un mot de passe a été réutilisé, des comptes encore actifs peuvent être exposés.", "Compromission de comptes, reprise d'accès non autorisée.", "Moyenne — correspondance e-mail, contexte de fuite partiel.", "Passer en revue les services concernés et confirmer rotation MFA / mots de passe (hors MANTIS).", "folder-perso"]
        ).map_err(|e| e.to_string())?;

        // Seed Actions
        conn.execute(
            "INSERT OR IGNORE INTO actions (id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params!["act-linkedin-email", "Masquer l'e-mail sur LinkedIn", "prio_003", "diff_001", "2026-07-28", "a_faire", "[\"Ouvrir les paramètres de confidentialité du profil LinkedIn.\",\"Retirer ou masquer l'adresse e-mail professionnelle de la vue publique.\"]", "Capture ou note : e-mail plus visible en mode public.", "folder-job"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO actions (id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params!["act-review-leaks", "Revue des comptes liés aux fuites", "prio_004", "diff_002", "2026-08-01", "en_cours", "[\"Lister les services où l'e-mail fuité était utilisé (hors MANTIS).\",\"Pour chaque compte encore actif : changer le mot de passe et activer MFA.\"]", "Liste des services revus (noms uniquement).", "folder-perso"]
        ).map_err(|e| e.to_string())?;

        // Seed Incident-Action Relations
        conn.execute(
            "INSERT OR IGNORE INTO incident_action (incident_id, action_id) VALUES (?1, ?2)",
            params!["inc-linkedin", "act-linkedin-email"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO incident_action (incident_id, action_id) VALUES (?1, ?2)",
            params!["inc-leaks", "act-review-leaks"]
        ).map_err(|e| e.to_string())?;

        // Seed RGPD Requests
        conn.execute(
            "INSERT OR IGNORE INTO rgpd_requests (id, type_id, target, dpo_contact, status_id, data_summary, draft_preview) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["rgpd-1", "type_003", "Annuaire Web Exemple", "privacy@annuaire-exemple.example", "status_002", "Adresse postale ancienne publiée sur une fiche nominative.", "Objet : Demande d'effacement..."]
        ).map_err(|e| e.to_string())?;

        // Seed Timeline
        conn.execute(
            "INSERT OR IGNORE INTO timeline_entries (id, event_type, description, created_at) VALUES (?1, ?2, ?3, ?4)",
            params!["tl-1", "Détection", "Fuite d'email détectée dans Collection #1", "2023-10-15T12:00:00Z"]
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO timeline_entries (id, event_type, description, created_at) VALUES (?1, ?2, ?3, ?4)",
            params!["tl-2", "Action", "Création d'une action pour changer le mot de passe", "2023-10-15T12:05:00Z"]
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = app_dir.join("mantis.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
    Ok(conn)
}

#[tauri::command]
fn list_folders(app: tauri::AppHandle) -> Result<Vec<Folder>, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, name, context FROM folders").map_err(|e| e.to_string())?;
    let folder_iter = stmt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            name: row.get(1)?,
            context: row.get(2)?,
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
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id FROM incidents").map_err(|e| e.to_string())?;
    let incident_iter = stmt.query_map([], |row| {
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
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("
        SELECT a.id, a.title, a.priority_id, a.difficulty_id, a.deadline, a.status, a.guidance, a.proof_expected, a.folder_id, ia.incident_id 
        FROM actions a 
        LEFT JOIN incident_action ia ON a.id = ia.action_id
    ").map_err(|e| e.to_string())?;
    let action_iter = stmt.query_map([], |row| {
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
            incident_id: row.get(9)?,
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
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, label, kind, value, folder_id, notes FROM identities").map_err(|e| e.to_string())?;
    let identity_iter = stmt.query_map([], |row| {
        Ok(Identity {
            id: row.get(0)?,
            label: row.get(1)?,
            kind: row.get(2)?,
            value: row.get(3)?,
            folder_id: row.get(4)?,
            notes: row.get(5)?,
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
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, title, kind, severity, status, discovered_at, source, what, why, folder_id FROM exposures").map_err(|e| e.to_string())?;
    let exposure_iter = stmt.query_map([], |row| {
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
        })
    }).map_err(|e| e.to_string())?;

    let mut exposures = Vec::new();
    for exposure in exposure_iter {
        exposures.push(exposure.map_err(|e| e.to_string())?);
    }

    Ok(exposures)
}

#[tauri::command]
fn list_rgpd_requests(app: tauri::AppHandle) -> Result<Vec<RgpdRequest>, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, type_id, target, dpo_contact, status_id, data_summary, draft_preview FROM rgpd_requests").map_err(|e| e.to_string())?;
    let rgpd_iter = stmt.query_map([], |row| {
        Ok(RgpdRequest {
            id: row.get(0)?,
            type_id: row.get(1)?,
            target: row.get(2)?,
            dpo_contact: row.get(3)?,
            status_id: row.get(4)?,
            data_summary: row.get(5)?,
            draft_preview: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut requests = Vec::new();
    for req in rgpd_iter {
        requests.push(req.map_err(|e| e.to_string())?);
    }

    Ok(requests)
}

#[tauri::command]
fn list_timeline_entries(app: tauri::AppHandle) -> Result<Vec<TimelineEntry>, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, event_type, description, created_at FROM timeline_entries").map_err(|e| e.to_string())?;
    let timeline_iter = stmt.query_map([], |row| {
        Ok(TimelineEntry {
            id: row.get(0)?,
            event_type: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in timeline_iter {
        entries.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(entries)
}

#[tauri::command]
fn get_posture_score(app: tauri::AppHandle) -> Result<PostureScore, String> {
    let conn = get_db_connection(&app)?;

    let mut score = 100;
    
    // Pénalités pour les incidents
    let mut stmt_inc = conn.prepare("SELECT severity FROM incidents").map_err(|e| e.to_string())?;
    let inc_iter = stmt_inc.query_map([], |row| {
        let sev: String = row.get(0)?;
        Ok(sev)
    }).map_err(|e| e.to_string())?;

    let mut open_incidents = 0;
    for sev_res in inc_iter {
        let sev = sev_res.map_err(|e| e.to_string())?;
        open_incidents += 1;
        match sev.as_str() {
            "critique" => score -= 20,
            "élevée" => score -= 10,
            "modérée" => score -= 5,
            "faible" => score -= 2,
            _ => {}
        }
    }

    // Pénalités pour les expositions
    let mut stmt_exp = conn.prepare("SELECT severity FROM exposures").map_err(|e| e.to_string())?;
    let exp_iter = stmt_exp.query_map([], |row| {
        let sev: String = row.get(0)?;
        Ok(sev)
    }).map_err(|e| e.to_string())?;

    let mut high_exposures = 0;
    for sev_res in exp_iter {
        let sev = sev_res.map_err(|e| e.to_string())?;
        if sev == "élevée" || sev == "critique" {
            high_exposures += 1;
        }
        match sev.as_str() {
            "critique" => score -= 5,
            "élevée" => score -= 5,
            "modérée" => score -= 3,
            "faible" => score -= 1,
            _ => {}
        }
    }

    // Bonus pour les actions terminées
    let completed: i64 = conn.query_row("SELECT COUNT(*) FROM actions WHERE status = 'faite'", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    let bonus = (completed * 2).min(10); // Max +10 points
    score += bonus as i32;

    if score < 0 { score = 0; }
    if score > 100 { score = 100; }

    Ok(PostureScore {
        score,
        open_incidents,
        high_exposures,
        completed_actions: completed as i32,
    })
}

#[tauri::command]
fn update_action_status(app: tauri::AppHandle, action_id: String, status: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    conn.execute(
        "UPDATE actions SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, action_id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_rgpd_request_status(app: tauri::AppHandle, request_id: String, status_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    conn.execute(
        "UPDATE rgpd_requests SET status_id = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status_id, request_id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            init_database(&app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_folders, 
            list_incidents, 
            list_actions, 
            list_identities, 
            list_exposures,
            list_rgpd_requests,
            list_timeline_entries,
            get_posture_score,
            update_action_status,
            update_rgpd_request_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
