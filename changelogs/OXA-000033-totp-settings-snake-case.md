- [OXA-000033](https://www.pivotaltracker.com/story/show/OXA-000033) - `TotpSettings` wire tokens renamed to snake_case with permanent decode aliases (register CLI-7)
    - `TotpSettings` was the one externally-tagged enum in the
      `AuthoritySettings` family emitting PascalCase tags (`"Disabled"`,
      `{"Enabled":{…}}`) while `NbfOffset`/`AuthorityStatus`/
      `AuthorityStrategy`/`EntitlementsEncoding` all emit snake_case. The
      kernel enum now carries `#[serde(rename_all = "snake_case")]`, so
      responses, requests and freshly-persisted `authorities.settings` JSONB
      rows emit `"disabled"` / `{"enabled":{…}}`. Struct-variant fields
      (`totp_ttl`, `webhook`, `webhook_key`) are unchanged.
    - **Decode stays dual-accept, permanently:** `#[serde(alias = "Disabled")]`
      / `#[serde(alias = "Enabled")]` keep every old PascalCase token decoding
      — stored rows, old SDK versions, hand-authored JSON. Rollout/rollback
      order-free; no typed Rust API change (same type, same variants).
    - **Storage migration:** new
      `20260930120000_rename_totp_settings_tags_to_snake_case.sql` backfills
      `authorities.settings` — scalar `"Disabled"` → `"disabled"` via
      `jsonb_set`, and the nested object case key-renames
      `{"Enabled":{…}}` → `{"enabled":{…}}`; both token forms covered,
      idempotent (second run touches zero rows).
    - **BREAKING for external assertions on emitted tokens** (mirror of our
      own `authorities.hurl:86`): anything asserting the *response/request*
      carries `"Disabled"` or an `"Enabled"` key must flip to the snake_case
      token. Senders are unaffected (aliases accept both forever); emitters
      and verifier tests are not.
    - Repo artifacts flipped in the same commit: SDK canned contract payload
      (`contract.rs`, `BUG(pinned)` marker deleted), bootstrap default-
      authority test, postgres `insert_authority`/`select_all_authorities`
      round-trip tests, `authorities.hurl` create/update bodies + response
      jsonpath, `user_authorities.hurl` seed body, seedz raw-SQL fixture
      (`fixtures.rs` does NOT follow the attribute), `docs/OAUTH.md` example,
      migration-plan 11/13 casing notes; register CLI-7 retired
      (marker recount 49/37).
