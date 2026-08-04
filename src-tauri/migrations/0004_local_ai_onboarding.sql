-- Phase 3: local AI onboarding, consent and prevalidated model choices.

CREATE TABLE IF NOT EXISTS local_ai_preferences (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0, 1)),
    selected_model_id TEXT,
    onboarding_status TEXT NOT NULL DEFAULT 'a_proposer'
        CHECK(onboarding_status IN ('a_proposer', 'sans_ia', 'configure')),
    consented_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO local_ai_preferences (id) VALUES (1);

INSERT OR IGNORE INTO local_ai_components (
    component_id, component_type, version, platform, architecture, sha256, byte_size, status, diagnostic
) VALUES
    ('qwen3-0.6b-q8', 'model', 'ef4088322893040952513f532f736ddeab518403', 'windows', 'x86_64',
     '9465e63a22add5354d9bb4b99e90117043c7124007664907259bd16d043bb031',
     639446688, 'non_installe', 'Modèle léger officiel, facultatif et inutilisé avant la Phase 4.'),
    ('qwen3-1.7b-q8', 'model', '90862c4b9d2787eaed51d12237eafdfe7c5f6077', 'windows', 'x86_64',
     '061b54daade076b5d3362dac252678d17da8c68f07560be70818cace6590cb1a',
     1834426016, 'non_installe', 'Modèle recommandé officiel, facultatif et inutilisé avant la Phase 4.');
