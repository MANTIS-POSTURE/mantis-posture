-- Deterministic consolidation of equivalent facts. The raw facts remain
-- immutable projections; a resolution only records their explainable outcome.
CREATE TABLE IF NOT EXISTS osint_fact_resolutions (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    fact_type TEXT NOT NULL,
    canonical_value TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('a_verifier','corroboree','contradictoire','rejete')),
    source_count INTEGER NOT NULL DEFAULT 0,
    favorable_count INTEGER NOT NULL DEFAULT 0,
    contradictory_count INTEGER NOT NULL DEFAULT 0,
    rationale TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(identity_id, fact_type, canonical_value)
);

CREATE TABLE IF NOT EXISTS osint_fact_resolution_evidence (
    resolution_id TEXT NOT NULL,
    fact_id TEXT NOT NULL,
    evidence_role TEXT NOT NULL CHECK(evidence_role IN ('favorable','contradictoire')),
    PRIMARY KEY(resolution_id, fact_id, evidence_role),
    FOREIGN KEY(resolution_id) REFERENCES osint_fact_resolutions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_osint_fact_resolutions_identity
    ON osint_fact_resolutions(identity_id, status, updated_at DESC);
