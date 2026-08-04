-- Phase 1: additive OSINT provenance, normalized observations and human decisions.
-- This migration is intentionally idempotent so an interrupted startup can retry safely.

CREATE TABLE IF NOT EXISTS osint_raw_artifacts (
    id TEXT PRIMARY KEY,
    scan_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    media_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    byte_size INTEGER,
    sha256 TEXT,
    collector_id TEXT NOT NULL,
    collector_version TEXT,
    collected_at TEXT NOT NULL,
    retention_status TEXT NOT NULL DEFAULT 'conserver'
        CHECK(retention_status IN ('conserver', 'suppression_demandee', 'supprime')),
    UNIQUE(scan_id, relative_path)
);

CREATE TABLE IF NOT EXISTS osint_observations (
    id TEXT PRIMARY KEY,
    scan_id TEXT NOT NULL,
    signal_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    observation_type TEXT NOT NULL,
    canonical_key TEXT NOT NULL,
    display_value TEXT NOT NULL,
    source TEXT NOT NULL,
    source_url TEXT,
    observed_at TEXT NOT NULL,
    relevance_status TEXT NOT NULL DEFAULT 'a_verifier'
        CHECK(relevance_status IN ('a_verifier', 'confirmee', 'pas_moi', 'ignoree', 'suivie')),
    UNIQUE(scan_id, canonical_key)
);

CREATE TABLE IF NOT EXISTS osint_evidence_links (
    id TEXT PRIMARY KEY,
    observation_id TEXT NOT NULL,
    artifact_id TEXT,
    source_url TEXT,
    evidence_label TEXT NOT NULL,
    excerpt TEXT,
    locator TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(observation_id, evidence_label, source_url)
);

CREATE TABLE IF NOT EXISTS osint_user_decisions (
    id TEXT PRIMARY KEY,
    target_type TEXT NOT NULL CHECK(target_type IN ('signal', 'observation')),
    target_id TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('confirmer', 'pas_moi', 'ignorer', 'suivre', 'corriger')),
    reason TEXT,
    previous_status TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_osint_artifacts_scan ON osint_raw_artifacts(scan_id);
CREATE INDEX IF NOT EXISTS idx_osint_observations_scan ON osint_observations(scan_id);
CREATE INDEX IF NOT EXISTS idx_osint_observations_signal ON osint_observations(signal_id);
CREATE INDEX IF NOT EXISTS idx_osint_evidence_observation ON osint_evidence_links(observation_id);
CREATE INDEX IF NOT EXISTS idx_osint_decisions_target ON osint_user_decisions(target_type, target_id, created_at DESC);

-- Backfill existing signals without changing or deleting the historical rows.
INSERT OR IGNORE INTO osint_raw_artifacts (
    id, scan_id, relative_path, collector_id, collected_at
)
SELECT 'artifact-' || id, id, raw_result_path, module_id, COALESCE(completed_at, started_at)
FROM osint_scans
WHERE raw_result_path IS NOT NULL AND trim(raw_result_path) <> '';

INSERT OR IGNORE INTO osint_observations (
    id, scan_id, signal_id, identity_id, observation_type, canonical_key,
    display_value, source, source_url, observed_at, relevance_status
)
SELECT
    'observation-' || s.id,
    s.scan_id,
    s.id,
    s.identity_id,
    s.signal_type,
    lower(trim(s.signal_type || '|' || s.source || '|' || COALESCE(s.source_url, s.evidence_ref, s.title))),
    s.title,
    s.source,
    s.source_url,
    s.discovered_at,
    CASE s.review_status
        WHEN 'Confirmé' THEN 'confirmee'
        WHEN 'Ce n''est pas moi' THEN 'pas_moi'
        WHEN 'Ignoré' THEN 'ignoree'
        WHEN 'Suivi' THEN 'suivie'
        ELSE 'a_verifier'
    END
FROM osint_signals s;

INSERT OR IGNORE INTO osint_evidence_links (
    id, observation_id, artifact_id, source_url, evidence_label, excerpt, locator, created_at
)
SELECT
    'evidence-' || s.id,
    'observation-' || s.id,
    CASE WHEN s.raw_result_path IS NOT NULL THEN 'artifact-' || s.scan_id ELSE NULL END,
    s.source_url,
    COALESCE(NULLIF(trim(s.evidence_ref), ''), s.source),
    s.explanation,
    s.evidence_ref,
    s.discovered_at
FROM osint_signals s;
