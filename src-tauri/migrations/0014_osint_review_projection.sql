-- Phase 7: durable claim reviews, session evolution and controlled projections.

CREATE TABLE IF NOT EXISTS osint_claim_reviews (
    id TEXT PRIMARY KEY,
    claim_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('confirmer','pas_moi','ignorer','suivre')),
    reason TEXT,
    previous_status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS osint_claim_session_presence (
    session_id TEXT NOT NULL,
    claim_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    source_count INTEGER NOT NULL DEFAULT 0,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(session_id, claim_id)
);

CREATE TABLE IF NOT EXISTS osint_projection_events (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    signal_id TEXT NOT NULL,
    projection_type TEXT NOT NULL CHECK(projection_type IN ('exposition','incident_action','dpo')),
    target_id TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK(outcome IN ('cree','reutilise')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(signal_id, projection_type, target_id)
);

CREATE INDEX IF NOT EXISTS idx_osint_claim_reviews_identity ON osint_claim_reviews(identity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_osint_claim_presence_identity ON osint_claim_session_presence(identity_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_osint_projection_identity ON osint_projection_events(identity_id, created_at DESC);

INSERT OR IGNORE INTO osint_claim_session_presence(session_id,claim_id,identity_id,source_count,recorded_at)
SELECT sc.session_id,ce.claim_id,c.identity_id,COUNT(DISTINCT o.source),COALESCE(ss.completed_at,ss.started_at)
FROM osint_claim_evidence ce
JOIN osint_claims c ON c.id=ce.claim_id
JOIN osint_observations o ON o.id=ce.observation_id
JOIN osint_scans sc ON sc.id=o.scan_id
JOIN osint_scan_sessions ss ON ss.id=sc.session_id
WHERE sc.session_id IS NOT NULL
GROUP BY sc.session_id,ce.claim_id,c.identity_id;
