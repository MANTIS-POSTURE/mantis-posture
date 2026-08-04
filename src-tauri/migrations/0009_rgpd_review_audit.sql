-- Phase 7: versioned deterministic drafts, explicit human review and local-use audit.
CREATE TABLE IF NOT EXISTS rgpd_draft_versions (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    contract_version TEXT NOT NULL,
    draft_text TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    source_url TEXT,
    source_signal_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS rgpd_user_reviews (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    draft_version_id TEXT NOT NULL,
    source_checked INTEGER NOT NULL CHECK(source_checked IN (0,1)),
    identity_checked INTEGER NOT NULL CHECK(identity_checked IN (0,1)),
    recipient_checked INTEGER NOT NULL CHECK(recipient_checked IN (0,1)),
    content_checked INTEGER NOT NULL CHECK(content_checked IN (0,1)),
    legal_notice_accepted INTEGER NOT NULL CHECK(legal_notice_accepted IN (0,1)),
    decision TEXT NOT NULL CHECK(decision IN ('valide','revoque')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS rgpd_draft_uses (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    draft_version_id TEXT NOT NULL,
    use_type TEXT NOT NULL CHECK(use_type IN ('copie','export_texte')),
    relative_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_rgpd_versions_request ON rgpd_draft_versions(request_id,created_at DESC);
CREATE INDEX IF NOT EXISTS idx_rgpd_reviews_request ON rgpd_user_reviews(request_id,created_at DESC);
CREATE INDEX IF NOT EXISTS idx_rgpd_uses_request ON rgpd_draft_uses(request_id,created_at DESC);

INSERT INTO rgpd_draft_versions (id,request_id,contract_version,draft_text,content_sha256,source_url,created_at)
SELECT 'legacy-'||id,id,'legacy-import',draft_preview,'legacy-unverified',source_url,created_at
FROM rgpd_requests r
WHERE NOT EXISTS (SELECT 1 FROM rgpd_draft_versions v WHERE v.request_id=r.id);
