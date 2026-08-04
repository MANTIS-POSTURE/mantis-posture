-- Phase 4: evidence and event trail for assisted DPO/RGPD workflows.
CREATE TABLE IF NOT EXISTS rgpd_request_evidence (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL REFERENCES rgpd_requests(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('source','identity','recipient','content','send','response')),
    locator TEXT NOT NULL,
    description TEXT,
    verified INTEGER NOT NULL DEFAULT 0 CHECK(verified IN (0,1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS rgpd_request_events (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL REFERENCES rgpd_requests(id) ON DELETE CASCADE,
    from_status TEXT,
    to_status TEXT,
    event_type TEXT NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_rgpd_evidence_request ON rgpd_request_evidence(request_id,created_at DESC);
CREATE INDEX IF NOT EXISTS idx_rgpd_events_request ON rgpd_request_events(request_id,created_at DESC);
