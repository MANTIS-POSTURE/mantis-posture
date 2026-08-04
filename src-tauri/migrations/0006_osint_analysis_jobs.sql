-- Automatic/background local analysis queue.

CREATE TABLE IF NOT EXISTS osint_analysis_jobs (
    id TEXT PRIMARY KEY,
    origin TEXT NOT NULL CHECK(origin IN ('scan_manuel', 'routine', 'relance_manuelle')),
    signal_ids_json TEXT NOT NULL,
    signal_count INTEGER NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('en_attente', 'en_cours', 'termine', 'fallback', 'erreur', 'interrompu')),
    estimated_seconds INTEGER NOT NULL,
    run_id TEXT,
    result_mode TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_osint_analysis_jobs_status ON osint_analysis_jobs(status, created_at);

INSERT OR IGNORE INTO app_settings (key, value) VALUES ('local_ai_manual_scan_mode', 'automatic');
