-- Phase 6: immutable report snapshots and local export audit trail.
CREATE TABLE IF NOT EXISTS osint_report_snapshots (
    id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL DEFAULT 1,
    content_json TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS osint_report_exports (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    export_format TEXT NOT NULL CHECK(export_format IN ('markdown','pdf')),
    relative_path TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_osint_report_exports_snapshot ON osint_report_exports(snapshot_id,created_at DESC);
