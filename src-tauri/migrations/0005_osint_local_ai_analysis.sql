-- Phase 4: versioned, auditable local analysis runs and validated outputs.

CREATE TABLE IF NOT EXISTS osint_analysis_runs (
    id TEXT PRIMARY KEY,
    task TEXT NOT NULL,
    contract_version TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    runtime_component_id TEXT,
    runtime_version TEXT,
    model_component_id TEXT,
    model_version TEXT,
    model_sha256 TEXT,
    status TEXT NOT NULL CHECK(status IN ('en_cours', 'valide', 'fallback', 'echec')),
    fallback_used INTEGER NOT NULL DEFAULT 0 CHECK(fallback_used IN (0, 1)),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    duration_ms INTEGER,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS osint_analysis_inputs (
    run_id TEXT NOT NULL,
    observation_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY(run_id, observation_id)
);

CREATE TABLE IF NOT EXISTS osint_analysis_outputs (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL UNIQUE,
    schema_version INTEGER NOT NULL,
    output_json TEXT NOT NULL,
    overview TEXT NOT NULL,
    needs_human_review INTEGER NOT NULL CHECK(needs_human_review IN (0, 1)),
    validated INTEGER NOT NULL CHECK(validated IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_osint_analysis_input_observation ON osint_analysis_inputs(observation_id);
CREATE INDEX IF NOT EXISTS idx_osint_analysis_runs_hash ON osint_analysis_runs(task, contract_version, input_hash, status);
