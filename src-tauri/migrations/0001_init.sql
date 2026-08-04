-- 0001_init.sql

-- Application settings (for future use)
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Folders - containers for organizing data
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    context TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Identities - personal identifiers
CREATE TABLE IF NOT EXISTS identities (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('nom', 'email', 'telephone', 'pseudo', 'domaine', 'url', 'adresse')),
    value TEXT NOT NULL,
    folder_id TEXT REFERENCES folders(id),
    notes TEXT,
    address_line1 TEXT,
    address_line2 TEXT,
    city TEXT,
    postal_code TEXT,
    country TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Exposures - public data traces
CREATE TABLE IF NOT EXISTS exposures (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('profil_public', 'fuite', 'annuaire', 'mention')),
    severity TEXT NOT NULL CHECK(severity IN ('faible', 'moderee', 'elevee', 'critique')),
    status TEXT NOT NULL CHECK(status IN ('nouvelle', 'en_suivi', 'acceptee', 'reduite')),
    discovered_at TEXT NOT NULL,
    source TEXT NOT NULL,
    what TEXT NOT NULL,
    why TEXT NOT NULL,
    folder_id TEXT REFERENCES folders(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Incident categories for better classification
CREATE TABLE IF NOT EXISTS incident_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Incidents - exposures requiring action
CREATE TABLE IF NOT EXISTS incidents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    severity TEXT NOT NULL CHECK(severity IN ('faible', 'moderee', 'elevee', 'critique')),
    discovered_at TEXT NOT NULL,
    what TEXT NOT NULL,
    why TEXT NOT NULL,
    impact TEXT NOT NULL,
    confidence TEXT NOT NULL,
    next_step TEXT NOT NULL,
    folder_id TEXT REFERENCES folders(id),
    category_id TEXT REFERENCES incident_categories(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Action priorities and difficulties
CREATE TABLE IF NOT EXISTS action_metadata (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK(type IN ('priority', 'difficulty')),
    value TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Actions - remediation steps
CREATE TABLE IF NOT EXISTS actions (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    priority_id TEXT NOT NULL REFERENCES action_metadata(id),
    difficulty_id TEXT NOT NULL REFERENCES action_metadata(id),
    deadline TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('a_faire', 'en_cours', 'faite')),
    guidance TEXT NOT NULL,  -- JSON array of steps
    proof_expected TEXT NOT NULL,
    folder_id TEXT REFERENCES folders(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Types of RGPD requests
CREATE TABLE IF NOT EXISTS rgpd_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL CHECK(name IN ('acces', 'rectification', 'effacement', 'opposition', 'dereferencement')),
    label TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- RGPD request statuses
CREATE TABLE IF NOT EXISTS rgpd_statuses (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL CHECK(name IN ('brouillon', 'prete', 'envoyee', 'repondue')),
    label TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Data protection requests (RGPD)
CREATE TABLE IF NOT EXISTS rgpd_requests (
    id TEXT PRIMARY KEY,
    type_id TEXT NOT NULL REFERENCES rgpd_types(id),
    target TEXT NOT NULL,
    dpo_contact TEXT NOT NULL,
    status_id TEXT NOT NULL REFERENCES rgpd_statuses(id),
    data_summary TEXT NOT NULL,
    draft_preview TEXT NOT NULL,
    source_url TEXT,
    contact_source_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Timeline entries
CREATE TABLE IF NOT EXISTS timeline_entries (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- OSINT Modules - definitions of surveillance routines
CREATE TABLE IF NOT EXISTS osint_modules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    target_kind TEXT NOT NULL, -- e.g., 'email', 'nom', 'adresse', 'domaine'
    frequency TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('planifie', 'actif', 'erreur', 'desactive')),
    last_run TEXT,
    next_run TEXT,
    script_path TEXT, -- Path to executable or script (e.g., python, bash, ./tools/holehe)
    script_args TEXT, -- Arguments for the script (e.g., scripts/check_breaches.py)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Relationships between entities
CREATE TABLE IF NOT EXISTS exposure_incident (
    exposure_id TEXT REFERENCES exposures(id),
    incident_id TEXT REFERENCES incidents(id),
    PRIMARY KEY (exposure_id, incident_id)
);

CREATE TABLE IF NOT EXISTS incident_action (
    incident_id TEXT REFERENCES incidents(id),
    action_id TEXT REFERENCES actions(id),
    PRIMARY KEY (incident_id, action_id)
);

CREATE TABLE IF NOT EXISTS action_rgpd (
    action_id TEXT REFERENCES actions(id),
    rgpd_id TEXT REFERENCES rgpd_requests(id),
    PRIMARY KEY (action_id, rgpd_id)
);

CREATE TABLE IF NOT EXISTS incident_rgpd (
    incident_id TEXT REFERENCES incidents(id),
    rgpd_id TEXT REFERENCES rgpd_requests(id),
    PRIMARY KEY (incident_id, rgpd_id)
);

CREATE TABLE IF NOT EXISTS folder_identity (
    folder_id TEXT REFERENCES folders(id),
    identity_id TEXT REFERENCES identities(id),
    PRIMARY KEY (folder_id, identity_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_identities_folder ON identities(folder_id);
CREATE INDEX IF NOT EXISTS idx_exposures_folder ON exposures(folder_id);
CREATE INDEX IF NOT EXISTS idx_incidents_folder ON incidents(folder_id);
CREATE INDEX IF NOT EXISTS idx_actions_folder ON actions(folder_id);
CREATE INDEX IF NOT EXISTS idx_rgpd_requests ON rgpd_requests(status_id);

-- Catalogue initial OSINT -- DONNEES DE REFERENCE, PAS DE TEST
INSERT OR IGNORE INTO osint_modules (id, name, description, target_kind, frequency, status, last_run, next_run, script_path, script_args) VALUES
('osint-email-intel', 'XposedOrNot (fuites)', 'Verifie les fuites connues associees a un e-mail autorise. Les resultats restent a confirmer.', 'email', 'Manuel', 'actif', NULL, NULL, NULL, NULL),
('osint-email-platforms', 'User Scanner (comptes)', 'Recherche des comptes potentiels associes a un e-mail ou pseudo autorise. Les resultats restent a verifier.', 'e-mail ou pseudo', 'Hebdomadaire', 'planifie', NULL, NULL, NULL, NULL),
('osint-web-footprint', 'Empreinte Web (DDGS)', 'Recherche des mentions publiques d''un nom, pseudo ou e-mail autorise. Resultats indicatifs, a verifier avant toute action.', 'nom, pseudo ou e-mail', 'Manuel', 'planifie', NULL, NULL, NULL, NULL),
('osint-username-profiles', 'Profils publics (Maigret)', 'Recherche des profils publics potentiels associes a un pseudo autorise. Chaque correspondance doit etre verifiee.', 'pseudo', 'Hebdomadaire', 'planifie', NULL, NULL, NULL, NULL),
('osint-gmail-profile', 'GHunt (profil Google)', 'Prevu pour une configuration Google explicite. Pas encore executable.', 'email', 'Manuel', 'desactive', NULL, NULL, NULL, NULL);
