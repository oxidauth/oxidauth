# OXA-000016 — update_authority overwrites client_key/status with NULL; row corrupted before the decode error

**Original ID:** DATA-3 · **Severity:** P1 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · REJECTED — **wontfix by owner: by design, same ruling as OXA-000015 — repository layer is trusted raw SQL; NULL-overwrite contract is caller-managed, not SQL-guarded**. Claims verified intact at HEAD (HTTP path masked by service backfills; exposure limited to direct repo callers; `client_key: None` commits an unreadable row). Ops note kept live: if a deployment ever reports `WHERE client_key IS NULL` rows, the §3 repair query + IdP/redirect re-sync caveats apply. Service keep-on-omit fix stays in OXA-000003 (already SCHEDULED).


## Locations

Register line (`BUGS_AND_NOTES.md:35`): `oxidauth-postgres/src/authorities/update_authority/:124,151,182` — "Same unconditional overwrite: `status: None` writes NULL; `client_key: None` writes NULL too — the row is corrupted, then the read path cannot decode NULL→`Uuid` so the call errors **after** the damage is committed."

Known drift pattern, confirmed here: the three quoted lines are `BUG(pinned)` comment markers inside the test module (`mod.rs:124`, `:151`, `:182`), not the product defect; the crate tree also moved under `src/oxidauth/` (plan-02). The defect is the bind block in `call()` plus `update_authority.sql`.

Verified current locations:

| What | Path |
|---|---|
| Repository binds the `Option`s straight through | `src/oxidauth/oxidauth-postgres/src/authorities/update_authority/mod.rs:12-26` — `client_key: Option<Uuid>` at `:15`; `status` → `Option<String>` (`None` ⇒ SQL NULL) at `:16-21`; `strategy`/`settings`/`params` unconditional at `:22-24`; `fetch_one` on the **write pool** at `:25-26`; decode `try_into` at `:28` |
| Full-row `SET` overwrite | `src/oxidauth/oxidauth-postgres/src/authorities/update_authority/update_authority.sql:2-9` (`name` `:3`, `client_key = $3` `:4`, `status = $4` `:5`, `strategy` `:6`, `settings` `:7`, `params` `:8`); `WHERE id = $1` `:10`; `RETURNING *` `:11` |
| Decode target is non-`Option` | `PgAuthority` struct `src/oxidauth/oxidauth-postgres/src/authorities/mod.rs:30-41` — `client_key: Uuid` `:34`, `status: String` `:35`; domain `Authority.client_key: Uuid` at `src/oxidauth/oxidauth-kernel/src/authorities/mod.rs:21` |
| Pinned repository tests | same `update_authority/mod.rs`: `it_should_reject_null_status_overwrite` `:122-147` (marker `:124`), `it_should_write_null_client_key_that_the_read_path_cannot_decode` `:149-185` (markers `:151`, `:182`) |
| Column DDL | `src/oxidauth/oxidauth-postgres/migrations/20221020180349_create_authorities.sql:4` — `client_key UUID UNIQUE DEFAULT uuid_generate_v4()` (**nullable**, default does not apply to an explicitly-listed column); `:5` `status VARCHAR(32) NOT NULL`; `uuid_generate_v4()` comes from uuid-ossp (`20221019185650_create_uuid_ext.sql`) |
| HTTP-path mask (service always sends `Some`) | `src/oxidauth/oxidauth-services/src/authorities/update_authority.rs:61-64` (client_key ← `Uuid::new_v4()` — that's SEC-3/OXA-000003), `:66-69` (status ← `current.status` backfill) |
| Only route that reaches the repo | `PUT /api/v1/authorities/{authority_id}` — router `src/oxidauth/oxidauth-api/src/server/api/v1/authorities/mod.rs:26`; handler maps repository `Err` to HTTP 400 with the raw DB error string (`oxidauth-api/src/server/api/v1/authorities/update_authority.rs:61-68`) |
| Same hole on create | `insert_authority/mod.rs:15` binds `params.client_key` (`Option<Uuid>`) directly; `insert_authority.sql:2` lists `client_key` explicitly (only `id` is COALESCE-guarded, `:3`) |
| Provenance | `missing-tests.md:34, 59, 324, 325` (`:324` cites the pin as `mod.rs:153-186` — 4-line drift from the current `:149-185`) |

## Problem

At the repository layer, `UpdateAuthority` is a full-row write with no keep-on-omit semantics for its two `Option` fields (`src/oxidauth/oxidauth-kernel/src/authorities/update_authority.rs:24-25`: `client_key: Option<Uuid>`, `status: Option<AuthorityStatus>`):

1. **`status: None`** — `mod.rs:16-21` binds SQL NULL into `status = $4`; the `NOT NULL` constraint rejects the statement (SQLSTATE 23502). The call fails with a raw Postgres error string (pinned: `format!("{err:?}").contains("not-null constraint")`, `mod.rs:143-146`). No corruption, but "field not supplied" is indistinguishable from "clear the column", so a keep-semantics update is impossible at this layer.
2. **`client_key: None`** — `client_key = $3` writes NULL (the column's `DEFAULT uuid_generate_v4()` does **not** apply because the column is listed explicitly in `SET`). The statement succeeds and **commits** (single statement on the write pool, implicit transaction). Then sqlx decodes `RETURNING *` client-side into `PgAuthority`, whose `client_key` is a non-`Option` `Uuid` (`mod.rs:34`), so decoding NULL fails with `Error::Decode` — the caller receives an error **after the row is already corrupted**. This is exactly what the test proves with a raw `SELECT client_key …` after the failed call: `assert!(stored.is_none(), "pinned: client_key overwritten with NULL")` (`mod.rs:176-184`).

Register wording nuance: "status: None writes NULL" is true of the bind, but the constraint prevents corruption — the commit-then-error corruption is specific to `client_key`.

## Analysis

**Why the damage is committed before the error.** `sqlx::query_as(...).fetch_one(&self.db.write_pool())` runs the `UPDATE … RETURNING *` as one statement outside any explicit transaction; Postgres commits it on completion and returns the new row. The type mismatch (`NULL` → `Uuid`) is caught only afterwards, in the Rust `FromRow` decode, and wrapped into the call's `BoxedError`. No Rust-side error handling can roll it back. `mod.rs:28`'s `result.try_into()?` is downstream and irrelevant — decode already failed inside `fetch_one`.

**A NULL-client_key row is unreadable through every repository read path.** All readers decode into the same non-`Option` `PgAuthority`: `select_authority_by_id/mod.rs:12-17` (`fetch_one`), `select_authority_by_client_key/mod.rs:17-22` (`fetch_one`, and `WHERE client_key = $1` can never match a NULL row in the first place), `select_authority_by_strategy/mod.rs:16-22` (`fetch_optional`), `select_all_authorities/mod.rs:12-18` (`fetch_all` — a single poisoned row fails the whole list, so `GET /authorities` becomes permanently 400 [INFERENCE: standard sqlx per-row decode semantics; not exercised in a test]). `delete_authority/mod.rs:12-14` also decodes `RETURNING *`, so even DELETE via the API errors (while deleting the row, CASCADEing `user_authorities` per `20221020204155_create_user_authorities.sql:11` [INFERENCE, same commit-before-decode mechanism]) — destructive, and still reported as a failure. The row can only be repaired by direct SQL.

**Blast radius if reached:** the poisoned authority takes down every by-client_key flow (authenticate, register/AOR, OAuth2 redirect+callback, TOTP validate, update_password, create_user_authority — consumer list verified in OXA-000003) and, via `FindAuthorityByStrategy`, the bootstrap path (`oxidauth-services/src/bootstrap/mod.rs`), which then cannot see the default authority.

**Who can reach it today.** Nobody over HTTP: the service use case is the sole product caller of `UpdateAuthorityQuery` (verified by grep across `src/oxidauth` — every other non-test hit is a DTO re-export, the provider wiring, or the `oxidauth-rs` HTTP client SDK; `oxidauth-import-export` and `oxidauth-cli` never reference `UpdateAuthority`), and it unconditionally backfills both columns to `Some` (`:61-64`, `:66-69`). The hole is **latent**: direct `PgAuthorityRepository` callers — embedders, admin tooling, future refactors that bypass the use case, tests reaching for "keep the current value" semantics — corrupt the row with one call. The sibling register item DATA-2 (`update_user` silently NULLs omitted names, and there the columns *are* nullable so the corruption lands silently and permanently) shows the same defect class already has a live victim.

**Same hole, create path.** `insert_authority` binds `Option<Uuid>` client_key directly (`mod.rs:15`) into an explicitly-listed column, so a direct-repo create with `client_key: None` commits a NULL-client_key row and then errors on the same decode — identical commit-before-damage shape, currently untested (every insert test passes `Some(...)`; only missing *status* is pinned, `insert_authority/mod.rs:186`). The service create masks it by minting the key (`oxidauth-services/src/authorities/create_authority.rs:29-32`, correct there per OXA-000003).

**Mixed `None` semantics within one struct.** `id: Option<Uuid>` binds into the `WHERE` (`mod.rs:13`): `id: None` means "match nothing" → `RowNotFound` (pinned, `mod.rs:187-207`), while the SET-side `None`s above mean "write NULL". The fix below gives the SET-side fields a third, explicit meaning: **keep**. `name`/`strategy`/`settings`/`params` are non-`Option` on the DTO — unconditional overwrite is correct full-replace PUT semantics and stays.

**Interaction with OXA-000003 (SEC-3) — split VOIDED by the wontfix ruling.** Two layers mishandle `client_key: None`; the former split had OXA-000003 §Resolution 3 defer the SQL work here while this ticket deferred the service fix back. **OXA-000003 closed wontfix 2026-09-29: the service's regenerate-on-omit is BY DESIGN — no service fix may be re-minted.** This ticket's `COALESCE` SQL work stands on its own (null-overwrite of the other fields), independent of client_key:

- The service fix (`req.client_key.replace(current.client_key)`) does **not** close this hole — any future/direct caller still reaches the raw SQL.
- This ticket's `COALESCE` fix does **not** fix SEC-3 — today's service mints a new UUID before the repository ever sees `None` (OXA-000003 §Analysis).
- Landing order is independent: with only this fix, HTTP behavior is unchanged because the service always binds non-NULL `Some`, and `COALESCE($3, client_key)` is a no-op for non-NULL bindings. `it_should_overwrite_every_column` (`:79-120`) keeps proving supplied values still overwrite.

**Explicit-null policy.** There is no representable "no client_key" (`Authority.client_key` is `Uuid`) and no representable "no status" (`NOT NULL` column, non-`Option` enum in the domain). Therefore `None` on these two fields cannot mean "clear"; keep-on-omit is the only coherent repository-layer policy. If an explicit "rotate the key" operation is ever wanted, it ships as `Some(new_key)` (already supported — verified by OXA-000003's `supplied_client_key_is_preserved_verbatim`) or a future dedicated endpoint, not as NULL. If a tri-state (`Keep`/`Set`/`Clear`) is ever genuinely needed, that is a DTO change, not a SQL NULL.

## Impact

- **Today, over HTTP:** none observable — fully masked by the service backfills. That is why this is latent, not live, but the register ranks it **P1** and the failure mode is the worst kind: silent persistence of unreadable data, discovered only as an unrecoverable read error.
- **Who is exposed:** any direct `PgAuthorityRepository` caller (embedders, future code paths, tests/tools) — one `UpdateAuthority { client_key: None, .. }` call turns the authority into a row no service can read, list, find, or safely delete; recovery requires DBA-level raw SQL. Deployments running custom wiring that calls the repository without the use case can already contain `client_key IS NULL` rows: their `GET /authorities` is permanently broken [INFERENCE], by-strategy lookups fail (bootstrap, `GET /by_strategy`), and every client_key login flow 400s with a raw `Decode`/lookup error string.
- **status variant:** loud rather than silent — a keep-style update ("change name only") cannot be expressed at the repository layer at all, and any error surfaced over HTTP becomes a 400 with raw Postgres text (`update_authority.rs:61-68`).
- **Not affected:** `create`/`find`/`list`/`delete` on healthy rows; the HTTP `PUT` path as currently wired; rows are recoverable via SQL (no data beyond the key itself is lost — `user_authorities` links by `authority_id`).

## Proposed resolution

This ticket owns exactly what OXA-000003 §Resolution 3 assigned to it: the repository/SQL change and the two repository test flips. The service keep-on-omit stays in OXA-000003.

1. **COALESCE-guarded UPDATE** — `src/oxidauth/oxidauth-postgres/src/authorities/update_authority/update_authority.sql`:

   ```sql
   UPDATE authorities
   SET
       name = $2,
       client_key = COALESCE($3, client_key),
       status = COALESCE($4, status),
       strategy = $5,
       settings = $6,
       params = $7,
       updated_at = NOW()
   WHERE id = $1
   RETURNING *
   ```

   `COALESCE` is the established pattern in this crate (every `insert_*.sql` uses `COALESCE($1, uuid_generate_v4())` for `id`). No Rust change is needed in `update_authority/mod.rs` — the binds already pass the `Option`s; the SQL now interprets `NULL` as "keep". Add a brief comment above the `SET` block stating the keep-on-omit contract for the two `Option` columns.

2. **Explicit-null policy recorded** in that comment: `None` ⇒ keep stored value; clearing either column is not expressible and not intended (see Analysis). No wire/DTO change, so no compat decision is forced on API consumers.

3. **Repair query for rows already corrupted** (ops one-off, run via psql; not a schema migration — no in-tree HTTP path creates them):

   ```sql
   -- detect
   SELECT id, strategy, params FROM authorities WHERE client_key IS NULL;
   -- repair (uuid-ossp is installed by migration 20221019185650; on PG13+ gen_random_uuid() also works)
   UPDATE authorities SET client_key = uuid_generate_v4() WHERE client_key IS NULL;
   ```

   Caveats to carry with it: `client_key` is `UNIQUE` (collision negligible for v4); a regenerated key will not match the callback URL baked into the authority's `params.redirect_uri`/`redirect_url` nor the IdP console registration (OXA-000003 §Edge cases) — OAuth2 authorities found by the detect query additionally need `params` and the IdP console updated, and `OXIDAUTH_DEFAULT_CLIENT_KEY` re-synced if the default authority is affected. Add as a guarded data-fix migration only if a target deployment reports affected rows.

4. **Adjacent create-path hardening (same defect, same ticket family):** `insert_authority.sql:3` → `VALUES (COALESCE($1, uuid_generate_v4()), $2, COALESCE($3, uuid_generate_v4()), $4, ...)`, mirroring the existing id pattern — on insert, minting is the correct meaning of `None` (the service already mints, so this is a no-op for the current caller and closes the direct-repo corruption path). Leave insert `status` alone (`it_should_reject_missing_status`, `insert_authority/mod.rs:186`, deliberately pins the NOT NULL violation there).

5. **Pinned-test flips owned by this ticket** (marker grep: `grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-postgres/src/authorities` — the other three markers in that directory belong to other register items):

   - `it_should_reject_null_status_overwrite` (`update_authority/mod.rs:122-147`, marker `:124`): rename to `it_should_keep_status_when_none`; delete the `BUG(pinned)` block `:124-126`; expect `Ok`, assert the returned **and** reread (`FindAuthorityById`) status still equals the seeded `Disabled`, and drop the `"not-null constraint"` assertion.
   - `it_should_write_null_client_key_that_the_read_path_cannot_decode` (`:149-185`, markers `:151-154`, `:182-183`): rename to `it_should_keep_client_key_when_none`; expect `Ok`; assert `updated.client_key == seeded.client_key`, a `FindAuthorityById` reread succeeds, and the raw `SELECT` (`:176-181`) now asserts `stored == Some(seeded.client_key)`.
   - Leave green as-is: `it_should_overwrite_every_column` (`:79-120`) — proves supplied `Some` values still overwrite through COALESCE; `it_should_error_on_missing_authority` (`:187-207`) — WHERE-side `id` semantics untouched.
   - If step 4 lands: add one insert test — `create_params(name, None)` succeeds and the returned/stored `client_key` is non-nil (the `it_should_reject_duplicate_client_key` shape at `insert_authority/mod.rs:164-184` shows the pattern).
   - Register/docs hygiene once green: drop `BUGS_AND_NOTES.md:35`; update `missing-tests.md:59` ("SQL-level overwrite is pinned here") and the review-log note at `:324` that records the corruption as current pinned behavior; `missing-tests.md:34` headline list mentions the class.
   - **No hurl changes here:** HTTP behavior is unchanged by this fix (the service always sends `Some`), so `authorities.hurl` assertions stay under OXA-000003's ownership.
   - Keep the service backfills (`oxidauth-services/src/authorities/update_authority.rs:61-69` — as fixed by OXA-000003) as defense-in-depth; this ticket does not remove them.

6. **Compat/migration.** No schema change, no DTO/wire change, no data migration shipped by default. The only behavior delta is for direct repository callers passing `None` on the guarded columns: previously `23502` error (status) or commit-then-`Decode` corruption (client_key); now keep. That delta is the entire point and cannot regress the in-tree caller, which never passes `None`. `COALESCE` is a no-op for every non-NULL binding.

## Verification

- **Targeted DB tests (the proof):** `cargo test -p oxidauth-postgres authorities::update_authority` (needs the compose DB, as `./src/oxidauth/database_test.sh` runs with `.env` + `MIGRATIONS_ENABLED=true`). Expect: both flipped tests green, `it_should_overwrite_every_column` and `it_should_error_on_missing_authority` still green.
- **Negative check that the flip is real:** temporarily restore `client_key = $3` / `status = $4`; both flipped tests must go red (status test: NOT NULL violation; client_key test: `Decode` error + stored NULL), then restore the fix. This mirrors the pre/post behavior pinned today at `:122-185`, so the flipped pair is the permanent regression test for the fix.
- **Create-path step (if taken):** `cargo test -p oxidauth-postgres authorities::insert_authority` — new `client_key: None` test green.
- **Direct-repo smoke (pre-fix repro / post-fix proof):** a throwaway `#[sqlx::test]` (or the flipped tests themselves) calling `PgAuthorityRepository::new(...).call(&UpdateAuthority { id: Some(seeded), client_key: None, status: None, .. })` — pre-fix this is exactly the pinned `Decode` error + `stored.is_none()`; post-fix it returns `Ok` with the seeded key and status intact.
- **Full suites once, after sibling tickets land:** `./src/oxidauth/database_test.sh` (postgres crate) and `bin/unit_test.sh` (unaffected — no service-layer change here). `./src/oxidauth/hurl.sh` should produce byte-identical authority results from *this* change alone (service still sends `Some`); any `PUT /authorities` omitted-field change there belongs to OXA-000003.
- **Repair rehearsal (ops path):** in psql against a test stack, `UPDATE authorities SET client_key = NULL WHERE id = <test-authority>;` → observe `GET /authorities`/`GET /{id}` fail with the decode error; run the repair `UPDATE` from step 3 → `GET /authorities` decodes and a login with the repaired key (read back via `GET /authorities`) succeeds.
