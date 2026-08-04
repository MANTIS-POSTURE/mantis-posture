-- Phase 1: additive action tracking. Legacy `status` is retained for compatibility;
-- `workflow_status` is the user-facing lifecycle.
CREATE TABLE IF NOT EXISTS remediation_plans (
    id TEXT PRIMARY KEY,
    identity_id TEXT REFERENCES identities(id),
    folder_id TEXT REFERENCES folders(id),
    scan_id TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('propose','valide','en_cours','termine','archive')),
    priority TEXT NOT NULL CHECK(priority IN ('basse','moderee','haute','critique')),
    rationale TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS remediation_plan_items (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES remediation_plans(id) ON DELETE CASCADE,
    action_id TEXT REFERENCES actions(id),
    exposure_id TEXT REFERENCES exposures(id),
    incident_id TEXT REFERENCES incidents(id),
    sort_order INTEGER NOT NULL DEFAULT 0,
    expected_outcome TEXT NOT NULL,
    proof_expected TEXT NOT NULL,
    execution_mode TEXT NOT NULL CHECK(execution_mode IN ('manuel','assiste','semi_automatique')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS action_events (
    id TEXT PRIMARY KEY,
    action_id TEXT NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    from_status TEXT,
    to_status TEXT NOT NULL,
    actor TEXT,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS action_evidence (
    id TEXT PRIMARY KEY,
    action_id TEXT NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('url','fichier','note','hash')),
    locator TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_actions_workflow_status ON actions(workflow_status);
CREATE INDEX IF NOT EXISTS idx_action_events_action ON action_events(action_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_action_evidence_action ON action_evidence(action_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_remediation_plans_identity ON remediation_plans(identity_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_remediation_items_plan ON remediation_plan_items(plan_id, sort_order);
