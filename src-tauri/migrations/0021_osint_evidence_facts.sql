-- Additive projection between a collected observation and a presentable claim.
-- Raw signals and evidence links remain the authoritative records.
CREATE TABLE IF NOT EXISTS osint_evidence_facts (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    signal_id TEXT NOT NULL,
    observation_id TEXT NOT NULL,
    fact_type TEXT NOT NULL
        CHECK(fact_type IN ('breach_event','public_profile','public_mention','external_proof','source_unavailable')),
    canonical_value TEXT NOT NULL,
    display_value TEXT NOT NULL,
    source_url TEXT,
    source_reliability TEXT NOT NULL
        CHECK(source_reliability IN ('structuree','plateforme_directe','indexee','agregateur','inconnue')),
    match_level TEXT NOT NULL
        CHECK(match_level IN ('identifiant_exact','pseudo_exact','nom_seul','non_verifie','rejete_utilisateur')),
    fact_status TEXT NOT NULL
        CHECK(fact_status IN ('retenu','avance','rejete')),
    rationale TEXT NOT NULL,
    evidence_fingerprint TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(signal_id, fact_type, canonical_value)
);

CREATE INDEX IF NOT EXISTS idx_osint_evidence_facts_identity
    ON osint_evidence_facts(identity_id, fact_status, fact_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_osint_evidence_facts_fingerprint
    ON osint_evidence_facts(identity_id, evidence_fingerprint);
