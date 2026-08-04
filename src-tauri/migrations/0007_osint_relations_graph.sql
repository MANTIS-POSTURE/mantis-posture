-- Phase 5: bounded, explainable graph projections. Source observations remain authoritative.
CREATE TABLE IF NOT EXISTS osint_entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('identite','observation','source')),
    label TEXT NOT NULL,
    canonical_value TEXT NOT NULL,
    identity_id TEXT,
    first_seen_at TEXT,
    last_seen_at TEXT,
    UNIQUE(entity_type, canonical_value, identity_id)
);

CREATE TABLE IF NOT EXISTS osint_relations (
    id TEXT PRIMARY KEY,
    from_entity_id TEXT NOT NULL,
    to_entity_id TEXT NOT NULL,
    relation_type TEXT NOT NULL CHECK(relation_type IN ('observe','collecte_par','correspondance_multi_source','contredit')),
    evidence_level TEXT NOT NULL CHECK(evidence_level IN ('observe','possible','probable','corroboree','contradiction')),
    justification TEXT NOT NULL,
    review_status TEXT NOT NULL DEFAULT 'proposee' CHECK(review_status IN ('proposee','validee','rejetee')),
    first_seen_at TEXT,
    last_seen_at TEXT,
    UNIQUE(from_entity_id,to_entity_id,relation_type)
);

CREATE TABLE IF NOT EXISTS osint_relation_evidence (
    relation_id TEXT NOT NULL,
    observation_id TEXT NOT NULL,
    evidence_role TEXT NOT NULL CHECK(evidence_role IN ('favorable','contradictoire')),
    PRIMARY KEY(relation_id,observation_id,evidence_role)
);

CREATE INDEX IF NOT EXISTS idx_osint_relations_from ON osint_relations(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_osint_relations_to ON osint_relations(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_osint_relation_evidence_observation ON osint_relation_evidence(observation_id);
