-- Phase 6: auditable identity-level synthesis built from deterministic claims.

CREATE TABLE IF NOT EXISTS osint_synthesis_runs (
    run_id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    session_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS osint_analysis_claim_inputs (
    run_id TEXT NOT NULL,
    claim_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY(run_id, claim_id)
);

CREATE INDEX IF NOT EXISTS idx_osint_synthesis_identity
    ON osint_synthesis_runs(identity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_osint_analysis_claim_input
    ON osint_analysis_claim_inputs(claim_id);
