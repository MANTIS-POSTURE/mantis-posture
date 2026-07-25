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
    kind TEXT NOT NULL CHECK(kind IN ('nom', 'email', 'telephone', 'pseudo', 'domaine', 'url')),
    value TEXT NOT NULL,
    folder_id TEXT REFERENCES folders(id),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Exposures - public data traces
CREATE TABLE IF NOT EXISTS exposures (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('profil_public', 'fuite', 'annuaire', 'mention')),
    severity TEXT NOT NULL CHECK(severity IN ('faible', 'modérée', 'élevée', 'critique')),
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
    severity TEXT NOT NULL CHECK(severity IN ('faible', 'modérée', 'élevée', 'critique')),
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
