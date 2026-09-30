-- OXA-000033: `TotpSettings` now carries `#[serde(rename_all = "snake_case")]`,
-- so backfill the at-rest `authorities.settings` JSONB to the emitted tokens.
-- The kernel keeps decode aliases for the old PascalCase tags, so this can run
-- before or after the binary ships; both statements are idempotent (second run
-- matches zero rows). Scalar case: `"totp": "Disabled"` -> `"disabled"`.
UPDATE authorities
SET
    settings = jsonb_set (
        settings,
        '{totp}',
        '"disabled"'
    )
WHERE settings ->> 'totp' = 'Disabled';
-- Object case: the externally-tagged `{"Enabled": {…}}` sits under `totp`, so
-- rename the nested key — strip `Enabled` from the `totp` object and re-`jsonb_set`
-- its inner object under `enabled`. Parentheses around `->` operands are
-- mandatory (Postgres operator lexing), and the `::text` casts pin the
-- jsonb-operator overloads against the untyped literals.
UPDATE authorities
SET
    settings = jsonb_set (
        settings,
        '{totp}',
        jsonb_set (
            (settings -> 'totp') - 'Enabled'::text,
            '{enabled}',
            settings -> 'totp' -> 'Enabled'
        )
    )
WHERE (settings -> 'totp') ? 'Enabled'::text;
