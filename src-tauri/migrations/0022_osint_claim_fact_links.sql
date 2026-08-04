-- Traceability bridge: a displayed claim remains connected to the exact facts
-- it uses. Facts and raw observations are never rewritten by this projection.
CREATE TABLE IF NOT EXISTS osint_claim_fact_links (
    claim_id TEXT NOT NULL,
    fact_id TEXT NOT NULL,
    evidence_role TEXT NOT NULL CHECK(evidence_role IN ('favorable','contradictoire')),
    PRIMARY KEY(claim_id, fact_id, evidence_role),
    FOREIGN KEY(claim_id) REFERENCES osint_claims(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_osint_claim_fact_links_fact
    ON osint_claim_fact_links(fact_id);
