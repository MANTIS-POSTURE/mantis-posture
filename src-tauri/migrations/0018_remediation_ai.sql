-- Phase 3: keep the local AI enrichment separate from the business plan.
CREATE TABLE IF NOT EXISTS remediation_plan_ai_enrichments (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES remediation_plans(id) ON DELETE CASCADE,
    contract_version TEXT NOT NULL,
    input_sha256 TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('ia_locale','deterministe')),
    status TEXT NOT NULL CHECK(status IN ('valide','fallback')),
    model_component_id TEXT,
    model_version TEXT,
    model_label TEXT,
    output_json TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_remediation_ai_plan ON remediation_plan_ai_enrichments(plan_id, created_at DESC);
