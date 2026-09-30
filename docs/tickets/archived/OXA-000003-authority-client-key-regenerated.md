# OXA-000003 — PUT /authorities regenerates client_key when the field is omitted

**Original ID:** SEC-3 · **Severity:** P1 · **Type:** bug · **Status:** closed (wontfix — owner ruled behavior intended by design)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #19 of 27 · reviewed 2026-09-29 · REJECTED — by-design ruling; hygiene follow-ups only


## Locations

Register line (`BUGS_AND_NOTES.md:17`) points at `oxidauth-services/src/authorities/update_authority.rs:350`. Both halves have drifted / need reading carefully:

- The crate tree moved under `src/oxidauth/` (plan-02), and **:350 is the `BUG(pinned)` comment inside the test module, not the defect**.
- Defect: `src/oxidauth/oxidauth-services/src/authorities/update_authority.rs:61-64` (`UpdateAuthorityUseCase::update_authority`).

Verified current locations:

| What | Path |
|---|---|
| Regeneration on omit | `src/oxidauth/oxidauth-services/src/authorities/update_authority.rs:61-64` |
| Correct `status` backfill right next to it (pattern to copy) | same file, `:66-69`; `current` fetched at `:56-59` |
| Copy-paste origin (regeneration is *correct* there) | `src/oxidauth/oxidauth-services/src/authorities/create_authority.rs:29-32` |
| Pinned unit tests | `src/oxidauth/oxidauth-services/src/authorities/update_authority.rs:348-381` (`omitted_client_key_is_regenerated`, marker `:350`), `:383-406` (`every_omission_gets_a_fresh_client_key`) |
| SQL full-row overwrite | `src/oxidauth/oxidauth-postgres/src/authorities/update_authority/update_authority.sql:4` (`client_key = $3`) |
| Repository binds `Option<Uuid>` straight through | `src/oxidauth/oxidauth-postgres/src/authorities/update_authority/mod.rs:15` |
| DATA-3 markers (repository-level `None`→NULL) | `src/oxidauth/oxidauth-postgres/src/authorities/update_authority/mod.rs:124`, `:151`, `:182` (tests `:122-147`, `:149-185`) |
| Column DDL: `client_key UUID UNIQUE DEFAULT uuid_generate_v4()` (nullable, unique) | `src/oxidauth/oxidauth-postgres/migrations/20221020180349_create_authorities.sql:4` |
| e2e assert of the buggy behavior | `src/oxidauth/hurl/tests/authorities.hurl:127-130` (comment + `new_client_key` capture), `:144` |
| Only product caller of `UpdateAuthorityQuery` | the use case above (verified by grep for `UpdateAuthority` across `src/`; every other hit is a DTO re-export, mock, or test) |
| Provenance | `missing-tests.md:34, 59, 215 (B5.1, P0), 289, 526`; `docs/migration-plan/11-scripts-and-tests.md:120, 182-184` |

Key consumers of `client_key` (all verified as callers of `FindAuthorityByClientKey` / `SelectAuthorityByClientKeyQuery`): `oxidauth-services/src/auth/authenticate.rs:107-111`, `auth/register.rs`, `auth/authenticate_or_register.rs:131-157`, `auth/strategies/oauth2/redirect.rs:51-60`, `auth/strategies/username_password/update_password.rs`, `totp/validate.rs:88-91`, `user_authorities/create_user_authority.rs:55-58`; plus the callback route that embeds the key in a URL: `oxidauth-api/src/server/api/v1/auth/oauth2/mod.rs:14` (`/callback/{client_key}`), and the client SDK constructor `oxidauth-rs/src/client/mod.rs:106` (`Client::new(base_url, client_key)`).

## Problem

`PUT /api/v1/authorities/{authority_id}` with a body that omits `client_key` silently replaces the authority's client key with a brand-new random UUIDv4 and answers **HTTP 200 / `success: true`**.

```rust
// update_authority.rs:61-64
if req.client_key.is_none() {
    req.client_key
        .replace(Uuid::new_v4());
}
```

`client_key` is `Option<Uuid>` on both the wire DTO and the domain DTO (`oxidauth-kernel/src/authorities/update_authority.rs:24`; the HTTP layer just wraps it — `oxidauth-http/src/authorities/update_authority.rs:10-13`), so "field absent from the JSON body" is indistinguishable from "caller wants a new key". An operator editing only `jwt_ttl` therefore rotates the credential every authenticated flow is built on. (The absent-field→`None` behavior is exercised by the hurl request at `authorities.hurl:108-126`, whose body has no `client_key` and gets a 200; the underlying serde-derive defaulting itself I did not test in isolation.)

`oxidauth/hurl/tests/authorities.hurl:127-130` asserts this end-to-end on purpose, and the service test `omitted_client_key_is_regenerated` pins it with a `BUG(pinned)` marker. `docs/migration-plan/11-scripts-and-tests.md:182-184` records the review's position: "honest pin of the flagged product bug." The register (`BUGS_AND_NOTES.md:17`) and `missing-tests.md:215` (B5.1) rank it **P0/P1**.

## Analysis

**Where it came from.** The `if … is_none() { replace(new_v4) }` shape is copied verbatim from `create_authority.rs:29-32`, where minting a key for a fresh row is exactly right. In the update use case the copy-paste is wrong by construction, and the file already contains the correct shape two lines below: `status` is backfilled from the freshly-loaded `current` row (`:66-69`). `current.client_key` is available in the same scope and simply never used — an oversight, not a design.

**Interaction with DATA-3 (repository NULL-overwrite).** Two independent layers mishandle `client_key: None`:

1. *Service* (`:61-64`): mints a key → SQL never sees NULL **on this path**. So today's HTTP behavior is "silently rotated", not "silently corrupted".
2. *Repository/SQL* (`mod.rs:15` binds the `Option` directly; `update_authority.sql:4` assigns it unconditionally): `client_key = NULL` is written. The column is nullable with a `DEFAULT` that does not apply to an explicitly-listed column, the write commits, and then `RETURNING *` cannot decode NULL into the non-`Option` `Uuid` of `Authority` (`oxidauth-kernel/src/authorities/mod.rs:21`), so the call *errors after the damage is committed*. Pinned at `update_authority/mod.rs:149-185` (markers `:151`, `:182`).

Consequences for ordering: the service currently *masks* DATA-3 for `client_key` on the HTTP path, and **a SQL-only fix (`COALESCE`) would not fix SEC-3** — the service mints the UUID before the repository ever sees `None`. Conversely, a service-only fix leaves the repository free to corrupt the column for any future direct caller (today the use case is the sole product caller). Fix the service here; harden the SQL under DATA-3.

**Capability check.** Nothing is lost by keep-on-omit: deliberate rotation is already expressible as `Some(new_key)` in the body (`supplied_client_key_is_preserved_verbatim`, `:323-346`, proves a supplied key reaches the repository untouched). The authorities router (`oxidauth-api/src/server/api/v1/authorities/mod.rs:19-28`) has no rotation endpoint, so today's *only* rotation mechanism is this accident — fixing it should be paired with documenting the explicit-key route.

**Edge cases.**

- Every omission rotates: two consecutive key-less updates produce two distinct keys (`every_omission_gets_a_fresh_client_key`, `:383-406`). An admin poking at an authority a few times churns the key repeatedly, so "reconnect the old key" is not a reliable recovery story.
- The new key *is* returned in `payload.authority.client_key` (`RETURNING *` → response), so a CLI operator who reads the payload sees the change — but a UI/automation that discards the payload sees a plain success, and the key was never in the request to begin with.
- **OAuth2 authorities break three ways at once.** The authority `params` bake the key into the callback URL: `docs/OAUTH.md:87-88` shows `redirect_uri`/`redirect_url` containing `/api/v1/auth/oauth2/callback/<client_key>`, and `oauth2/redirect.rs:51-60` appends `state` to that stored URL. That same URL is what the operator registers in the IdP console. After rotation: (a) the IdP's registered redirect URI hits the old callback path → authority lookup fails; (b) even after fixing the console, the stored `params.redirect_url` still carries the old callback path → the IdP rejects with `redirect_uri_mismatch`; (c) `state` is an argon2 hash bound to the key's bytes (`redirect.rs:59`, verified in AOR at `authenticate_or_register.rs:141-157`), so in-flight flows minted pre-rotation stop verifying. Recovery needs both a params rewrite and an IdP-console edit.
- The failure is reported confusingly: `authenticate.rs:107-111` maps a missing authority to `AuthorityNotFoundError::ClientKey` ("authority not found by client_key: …"), but per DATA-8 the repository uses `fetch_one` (`oxidauth-postgres/src/authorities/select_authority_by_client_key/mod.rs:16-23`), so a stale key raises `RowNotFound` first — callers get a raw DB error string as HTTP 400 (`docs/migration-plan/11-scripts-and-tests.md:112-118` pins that not-found-is-400, never-404 convention), not a clean domain error.
- **Nothing self-heals.** `bootstrap/mod.rs:382-427` only *creates* the default authority when absent (find-by-strategy short-circuits) and reads `OXIDAUTH_DEFAULT_CLIENT_KEY` at creation time only. Once rotated, the env-var-pinned key is permanently out of sync with the row; restarts, `bin/reset-db.sh` aside, never restore it.
- Worst victim: the default `username_password` authority seeded with the operator-pinned `OXIDAUTH_DEFAULT_CLIENT_KEY`. That key is the login credential for the whole `.env`/hurl/devops toolchain — 16 of the suite's 19 `.hurl` files send `"client_key": "{{admin_client_key}}"` in a body (`variables-local` only defines the variable) — so one careless PUT against it locks the admin out of the API that would be used to repair it (the repair still works via `GET /authorities` + a re-PUT, but only with a token minted before the rotation, or via direct SQL).

## Impact

- **Who:** every holder of an authority's `client_key` — `oxidauth-rs` clients (`Client::new(base_url, client_key)`), any service/mobile/web integration storing the key, deploy configs (`OXIDAUTH_DEFAULT_CLIENT_KEY` in `.env`, helm/compose), OAuth2 IdP console registrations, and end users logging in through username/password, TOTP, or SSO.
- **What breaks:** `authenticate`, `register`, authenticate-or-register, OAuth2 redirect + callback, TOTP validate, password recovery (`update_password`), and `create_user_authority` — all resolve the authority *by client_key*. A rotated key turns every one of them into a lookup miss.
- **Severity:** P1 — auth availability loss, silent (200 OK + `success: true`), and self-perpetuating (each further edit re-rotates; bootstrap never repairs). No rows are lost: `user_authorities` links by `authority_id`, and refresh tokens/permission grants are untouched, so the damage is credential-level and reversible **only by an operator who knows the key changed**.
- **Trigger surface:** any admin with `oxidauth:authorities:manage` — i.e. the bootstrap admin role — editing name/settings/params via the API, the Rust client, or any UI wrapping it. No malformed input needed; the ordinary "rename my authority" call does it.
- **Not affected:** `create` (minting a key for a new row is intended) and `delete`/`find`/`list`.

## Proposed resolution

1. **Keep-on-omit in the use case** (the actual SEC-3 fix) — mirror the adjacent `status` backfill at `oxidauth-services/src/authorities/update_authority.rs:61-64`:

   ```rust
   if req.client_key.is_none() {
       req.client_key.replace(current.client_key);
   }
   ```

   Semantics: omitted ⇒ unchanged; supplied ⇒ set (deliberate rotation). Then `use uuid::Uuid;` (`:14`) has no product-code use left — move it into `mod tests` (tests reference bare `Uuid` at `:101, :106, :147, :399`) and confirm with `cargo test --no-run -p oxidauth-services` (register T-3: `cargo check` does not compile `#[cfg(test)]`).

2. **Rotate explicitly, not accidentally.** Keep the API shape; document in `docs/AUTHORITIES.md` (which currently never mentions `client_key`) that a key rotation is a `PUT` with a freshly minted `client_key`, and that OAuth2 authorities additionally need `params.redirect_uri`/`redirect_url` and the IdP console updated in the same change. Do NOT encode "rotate" as a sentinel string in a UUID field. If server-generated rotation must stay on the table, it is a separate product slice (`POST /authorities/{id}/rotate_client_key`) — out of scope here.

3. **Close the repository hole under DATA-3, not here.** `update_authority.sql` should preserve-on-omit for the two columns whose DTO fields are `Option` and whose domain types are non-`Option`:

   ```sql
   client_key = COALESCE($3, client_key),
   status     = COALESCE($4, status),
   ```

   (name/strategy/settings/params are non-`Option` and stay unconditional.) Neither column has a legitimate "clear it" request — `status` is `NOT NULL` and `client_key` cannot decode as NULL — so COALESCE costs nothing. Coordinate with the DATA-3 ticket (same file, three markers) so one owner lands the SQL change and flips `it_should_reject_null_status_overwrite` / `it_should_write_null_client_key_that_the_read_path_cannot_decode`.

4. **Pinned tests to flip** — the marker grep is `grep -rn 'BUG(pinned)' src --include='*.rs'`; SEC-3 owns exactly the two service tests, and the SQL/marker pair belongs to DATA-3:

   - `oxidauth-services/src/authorities/update_authority.rs:348-381`: rename `omitted_client_key_is_regenerated` → `omitted_client_key_is_kept`, delete the `BUG(pinned)` block `:350-354`, and replace `assert_ne!(regenerated, current_client_key())` + the `get_version() == Some(Version::Random)` pin (`:371-380`) with `assert_eq!(captured[0].client_key, Some(current_client_key()))`. This is the regression test: it asserts the *stored* row's key reaches the repository.
   - `oxidauth-services/src/authorities/update_authority.rs:383-406`: rename `every_omission_gets_a_fresh_client_key` → `repeated_omissions_keep_the_same_key`; `assert_eq!(keys[0], keys[1])` (keep the `expect("generated")`/non-`None` shape so an accidental `None`-pass-through still fails loudly).
   - Leave `supplied_client_key_is_preserved_verbatim` (`:323-346`) and every other test in the module untouched — they already describe intended behavior.
   - `oxidauth/hurl/tests/authorities.hurl`: delete the `new_client_key` capture (`:129-130`) and rewrite the comment (`:127-128`); the create-time capture `authority_client_key` already exists at `:74` and is currently unused, so assert `$.payload.authority.client_key == "{{authority_client_key}}"` on both the `PUT` response and the follow-up `GET` (`:144`).
   - Register/docs hygiene once green: drop `BUGS_AND_NOTES.md:17`; update `missing-tests.md:34, 59, 215, 289, 526` and `docs/migration-plan/11-scripts-and-tests.md:120, 182-184`, which all record regeneration as *current, asserted* behavior.

5. **Compat / migration.** No wire-format, DTO, or schema change; no migration required for the fix itself. The behavior change is strictly toward "PUT without a field keeps it", and the only observable difference is that `payload.authority.client_key` now equals the create-time key — a caller that *relied* on regeneration reads the returned payload or sends an explicit key. **Data repair is a separate question:** the column is nullable, so any deployment that hit the DATA-3 repository path has unreadable rows (`client_key IS NULL` — the row decodes as an error forever). If a target deployment has any, a one-off repair (`UPDATE authorities SET client_key = uuid_generate_v4() WHERE client_key IS NULL`, reusing the `uuid-ossp` extension from migration `20221019185650`) restores readability but hands the operator a brand-new credential to distribute; decide per deployment rather than shipping it blind. Verify existence first with `SELECT count(*) FROM authorities WHERE client_key IS NULL;` — I could not check any live database from here, so whether such rows exist anywhere is unverified.

## Verification

- **Service unit tests (the proof of the fix):** `cargo test -p oxidauth-services authorities::update_authority` — the two flipped tests must now fail if `Uuid::new_v4()` is restored at `:61-64`. Full non-DB suite: `bin/unit_test.sh` (`cargo test --workspace --exclude '*-postgres' --exclude postgres`).
- **Negative check that the pin is real:** temporarily restore the regeneration line and confirm `omitted_client_key_is_kept` *and* `repeated_omissions_keep_the_same_key` both go red (a single-flip test would pass under both behaviors); remove the temporary change.
- **e2e / live API:** `./src/oxidauth/hurl.sh` (requires the compose stack up, or `OXIDAUTH_HURL_HOST`, plus repo-root `.env` with `OXIDAUTH_DEFAULT_CLIENT_KEY` / `OXIDAUTH_DEFAULT_ADMIN_PASSWORD`). Expect the `PUT /authorities` at `authorities.hurl:108` (body with no `client_key`) to echo the create-time key, and the `GET` at `:138` to still report it. Every one of the 16 authenticating `.hurl` files re-logs in with `admin_client_key` as its first request, so the suite's second pass is the standing canary that no test rotated the bootstrap key.
- **If the DATA-3 `COALESCE` lands alongside:** `./src/oxidauth/database_test.sh` (sources `.env`, `MIGRATIONS_ENABLED=true`), or targeted `cargo test -p oxidauth-postgres authorities::update_authority`; `it_should_overwrite_every_column` must stay green (supplied keys still overwrite) while `:149-185` flips to "stored client_key preserved + call decodes successfully".
- **Manual end-to-end credential check (worth one run):** create an authority, log in with its key, `PUT` it with only a renamed `name`, log in again with the *original* key and expect 200 — before the fix this is the 400/`RowNotFound` path, since DATA-8's `fetch_one` error outranks `AuthorityNotFoundError::ClientKey`.

## Decision (2026-09-29) — REJECTED; owner ruled the behavior is by design; no code change
- Reviewed jointly (scout verification + owner ruling 2026-09-29). The mechanism is exactly as documented in this ticket (omitted `client_key` on update ⇒ fresh UUIDv4, `update_authority.rs:61-64`), and the owner rules it **intended design** — the proposed keep-on-omit fix is rejected. PUT semantics stand: omission rotates.
- Facts that survive the ruling and need hygiene, not a fix (implement in any convenient commit):
  1. **Relabel, don't delete, the pins:** `omitted_client_key_is_regenerated` / `every_omission_gets_a_fresh_client_key` and their `BUG(pinned)` markers (`:350` block, `:383`) pin *intended* behavior — rename the markers to design-pins so the suite stops advertising this as a known bug; same for the "assert of the buggy behavior" comment at `hurl/tests/authorities.hurl:127-128`.
  2. **Register reclassify:** `BUGS_AND_NOTES.md:17` (SEC-3, P1) should be struck or rewritten as a design note — leaving a P1 bug line for ruled-intended behavior misdirects every future audit, exactly the failure mode this campaign exists to kill.
  3. **The documentation gap is now the actual risk:** `docs/AUTHORITIES.md` never mentions `client_key` at all (verified), so the rotation-on-omission semantics — plus the OAuth2 consequence (stored `redirect_uri` embeds the old key path; IdP console must be updated in the same change) — exist nowhere operator-readable. A design that silently rotates login credentials MUST be written down; recommend this as a follow-up doc slice.
- Scope boundaries that remain valid regardless of the ruling: the repository-level `client_key = $3` NULL-overwrite (`update_authority/mod.rs:15` + its three markers) is **OXA-000016/DATA-3's** problem and is unaffected — this ruling concerns the service layer only.
- Cross-reference update: OXA-000021's decision scheduled itself "before OXA-000003" so the rotation repro would read as the clean domain error — that ordering rationale is now **moot** (000003 is closed); 000021 stands on its own merits, order-agnostic.
- Acceptance: hygiene only — no `BUG(pinned)`/register/hurl-comment text describes the regeneration as a bug; AUTHORITIES.md documents omit⇒rotate + the OAuth2 console ritual (follow-up slice).
