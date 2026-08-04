-- Human identity targets with multiple user-entered search values.
-- Columns on `identities` are added by `ensure_identity_target_columns` because
-- SQLite has no portable `ADD COLUMN IF NOT EXISTS`.

CREATE TABLE IF NOT EXISTS identity_values (
    id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('prenom','nom','pseudo','email','telephone','adresse','domaine','url')),
    value TEXT NOT NULL,
    normalized_value TEXT NOT NULL,
    label TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','inactive')),
    origin TEXT NOT NULL DEFAULT 'user' CHECK(origin = 'user'),
    address_line1 TEXT,
    address_line2 TEXT,
    city TEXT,
    postal_code TEXT,
    country TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(identity_id, kind, normalized_value)
);

CREATE INDEX IF NOT EXISTS idx_identity_values_identity
    ON identity_values(identity_id, status, sort_order);
CREATE INDEX IF NOT EXISTS idx_identity_values_lookup
    ON identity_values(kind, normalized_value);

-- Preserve every legacy identity as one distinct target. Similar labels or values
-- are deliberately not merged: only the user can decide that two rows are one person.
INSERT OR IGNORE INTO identity_values (
    id, identity_id, kind, value, normalized_value, label, status, origin,
    address_line1, address_line2, city, postal_code, country, sort_order,
    created_at, updated_at
)
SELECT
    'legacy-' || id, id, kind, value, lower(trim(value)), label, 'active', 'user',
    address_line1, address_line2, city, postal_code, country, 0,
    created_at, updated_at
FROM identities
WHERE NOT EXISTS (
    SELECT 1 FROM identity_values existing WHERE existing.identity_id=identities.id
);

-- Versions 20-22 could recreate the legacy compatibility value after the
-- multi-value editor had already stored the real first and last names. Keep
-- that historical row for scan provenance, but never use it as a live target
-- when it is demonstrably a duplicate of the active first name and a distinct
-- active surname exists.
UPDATE identity_values AS legacy
SET status='inactive', updated_at=datetime('now')
WHERE legacy.id='legacy-' || legacy.identity_id
  AND legacy.kind='nom'
  AND legacy.status='active'
  AND EXISTS (
      SELECT 1 FROM identity_values first_name
      WHERE first_name.identity_id=legacy.identity_id
        AND first_name.kind='prenom'
        AND first_name.status='active'
        AND first_name.normalized_value=legacy.normalized_value
  )
  AND EXISTS (
      SELECT 1 FROM identity_values surname
      WHERE surname.identity_id=legacy.identity_id
        AND surname.kind='nom'
        AND surname.status='active'
        AND surname.id<>legacy.id
        AND surname.normalized_value<>legacy.normalized_value
  );
