-- Phase 4: deterministic claims derived from authoritative observations.
-- User-entered identity values remain separate and are never inferred here.
CREATE TABLE IF NOT EXISTS osint_claims (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    claim_type TEXT NOT NULL,
    canonical_key TEXT NOT NULL,
    display_value TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('a_verifier','corroboree','confirmee','contradictoire','rejetee')),
    priority TEXT NOT NULL CHECK(priority IN ('faible','moyenne','haute','critique')),
    favorable_count INTEGER NOT NULL DEFAULT 0,
    contradictory_count INTEGER NOT NULL DEFAULT 0,
    source_count INTEGER NOT NULL DEFAULT 0,
    first_observed_at TEXT NOT NULL,
    last_observed_at TEXT NOT NULL,
    rationale TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(identity_id, canonical_key)
);

CREATE TABLE IF NOT EXISTS osint_claim_evidence (
    claim_id TEXT NOT NULL,
    observation_id TEXT NOT NULL,
    evidence_role TEXT NOT NULL CHECK(evidence_role IN ('favorable','contradictoire')),
    PRIMARY KEY(claim_id, observation_id, evidence_role),
    FOREIGN KEY(claim_id) REFERENCES osint_claims(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_osint_claims_identity_priority
    ON osint_claims(identity_id, priority, last_observed_at);
CREATE INDEX IF NOT EXISTS idx_osint_claim_evidence_observation
    ON osint_claim_evidence(observation_id);
