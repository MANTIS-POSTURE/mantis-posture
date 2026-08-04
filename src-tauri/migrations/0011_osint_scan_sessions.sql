-- Aggregate one human-target scan while preserving every collector execution.
-- Columns on existing tables are added by `ensure_scan_session_columns`.

CREATE TABLE IF NOT EXISTS osint_scan_sessions (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL REFERENCES identities(id),
    origin TEXT NOT NULL CHECK(origin IN ('scan_manuel','routine')),
    status TEXT NOT NULL CHECK(status IN ('en_cours','termine','partiel','erreur')),
    planned_checks INTEGER NOT NULL DEFAULT 0,
    completed_checks INTEGER NOT NULL DEFAULT 0,
    failed_checks INTEGER NOT NULL DEFAULT 0,
    skipped_checks INTEGER NOT NULL DEFAULT 0,
    signal_count INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    summary TEXT
);

CREATE INDEX IF NOT EXISTS idx_osint_sessions_identity
    ON osint_scan_sessions(identity_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_osint_scans_session
    ON osint_scans(session_id, started_at);
CREATE INDEX IF NOT EXISTS idx_osint_scans_identity_value
    ON osint_scans(identity_value_id, started_at DESC);

-- Make pre-session scans visible in the identity history without combining
-- executions that may have been launched independently.
INSERT OR IGNORE INTO osint_scan_sessions (
    id,identity_id,origin,status,planned_checks,completed_checks,failed_checks,
    skipped_checks,signal_count,started_at,completed_at,summary
)
SELECT
    'legacy-'||s.id,s.identity_id,'scan_manuel',
    CASE WHEN s.status='termine' THEN 'termine' WHEN s.status='erreur' THEN 'erreur' ELSE 'partiel' END,
    1,CASE WHEN s.status='termine' THEN 1 ELSE 0 END,CASE WHEN s.status='erreur' THEN 1 ELSE 0 END,0,
    (SELECT COUNT(*) FROM osint_signals sig WHERE sig.scan_id=s.id),s.started_at,s.completed_at,
    'Exécution historique importée sans modification.'
FROM osint_scans s
WHERE s.session_id IS NULL;

UPDATE osint_scans SET session_id='legacy-'||id WHERE session_id IS NULL;

-- Best-effort lossless link for historical executions. The target snapshot is
-- matched only inside the same identity; ambiguous values are never merged.
UPDATE osint_scans
SET identity_value_id=(
        SELECT v.id FROM identity_values v
        WHERE v.identity_id=osint_scans.identity_id
          AND v.normalized_value=lower(trim(osint_scans.target))
        ORDER BY v.sort_order,v.created_at LIMIT 1
    ),
    target_kind_snapshot=COALESCE(target_kind_snapshot,(
        SELECT v.kind FROM identity_values v
        WHERE v.identity_id=osint_scans.identity_id
          AND v.normalized_value=lower(trim(osint_scans.target))
        ORDER BY v.sort_order,v.created_at LIMIT 1
    ))
WHERE identity_value_id IS NULL;

UPDATE osint_signals
SET identity_value_id=(SELECT identity_value_id FROM osint_scans WHERE id=osint_signals.scan_id)
WHERE identity_value_id IS NULL;

UPDATE osint_observations
SET identity_value_id=(SELECT identity_value_id FROM osint_scans WHERE id=osint_observations.scan_id)
WHERE identity_value_id IS NULL;

-- Provenance is inherited at insert time so existing parsers do not each need a
-- parallel persistence path.
CREATE TRIGGER IF NOT EXISTS trg_osint_signal_identity_value
AFTER INSERT ON osint_signals
WHEN NEW.identity_value_id IS NULL
BEGIN
    UPDATE osint_signals
    SET identity_value_id=(SELECT identity_value_id FROM osint_scans WHERE id=NEW.scan_id)
    WHERE id=NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_osint_observation_identity_value
AFTER INSERT ON osint_observations
WHEN NEW.identity_value_id IS NULL
BEGIN
    UPDATE osint_observations
    SET identity_value_id=(SELECT identity_value_id FROM osint_scans WHERE id=NEW.scan_id)
    WHERE id=NEW.id;
END;
