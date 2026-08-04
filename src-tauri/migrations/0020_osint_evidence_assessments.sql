-- Deterministic evidence assessment used before claims and local AI.
-- Raw observations remain immutable; this table records why a signal is shown or hidden.
CREATE TABLE IF NOT EXISTS osint_evidence_assessments (
    signal_id TEXT PRIMARY KEY,
    observation_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    source_family TEXT NOT NULL,
    source_reliability TEXT NOT NULL
        CHECK(source_reliability IN ('structuree','plateforme_directe','indexee','agregateur','inconnue')),
    match_level TEXT NOT NULL
        CHECK(match_level IN ('identifiant_exact','pseudo_exact','nom_seul','non_verifie','rejete_utilisateur')),
    publication_status TEXT NOT NULL
        CHECK(publication_status IN ('visible','masque','rejete')),
    rationale TEXT NOT NULL,
    evidence_fingerprint TEXT NOT NULL,
    assessed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_osint_evidence_assessments_identity
    ON osint_evidence_assessments(identity_id, publication_status, assessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_osint_evidence_assessments_fingerprint
    ON osint_evidence_assessments(identity_id, evidence_fingerprint);
