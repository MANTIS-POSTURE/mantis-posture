-- Phase 2: local AI component and recoverable download state.

CREATE TABLE IF NOT EXISTS local_ai_components (
    component_id TEXT PRIMARY KEY,
    component_type TEXT NOT NULL CHECK(component_type IN ('runtime', 'model')),
    version TEXT NOT NULL,
    platform TEXT NOT NULL,
    architecture TEXT NOT NULL,
    install_path TEXT,
    sha256 TEXT NOT NULL,
    byte_size INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('non_installe', 'telechargement', 'pret', 'erreur', 'supprime')),
    diagnostic TEXT,
    installed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS local_ai_downloads (
    id TEXT PRIMARY KEY,
    component_id TEXT NOT NULL,
    version TEXT NOT NULL,
    source_url TEXT NOT NULL,
    partial_path TEXT NOT NULL,
    expected_sha256 TEXT NOT NULL,
    expected_bytes INTEGER NOT NULL,
    downloaded_bytes INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK(status IN ('en_attente', 'en_cours', 'interrompu', 'verifie', 'erreur')),
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_local_ai_download_component ON local_ai_downloads(component_id, updated_at DESC);

INSERT OR IGNORE INTO local_ai_components (
    component_id, component_type, version, platform, architecture, sha256, byte_size, status, diagnostic
) VALUES (
    'llama-cpp-cpu', 'runtime', 'b10203', 'windows', 'x86_64',
    'd340e273cfb76a704b3badfc8a0f4ea55b4966b9a53c33fe36f7a025502802ef',
    18351018, 'non_installe', 'Runtime local facultatif ; aucun modèle n’est installé en Phase 2.'
);
