# OXA-000017 — public_keys has no key-material uniqueness, and insert_public_key commits before its response-side UTF-8 decode

**Original ID:** DATA-4 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 5 (hard — SQL migration + dedup + unique index + CHECK) · unreviewed


## Locations

All paths relative to `src/oxidauth/`. Register line verified against current code; both cited line numbers are pinned-test markers, not the defect (drift below).

- **Defect 1 — no uniqueness (schema):** `oxidauth-postgres/migrations/20221021180809_create_public_keys.sql:1-7` — `public_keys` has only the UUID primary key. The public key material lives in `public_key BYTEA NOT NULL` (line 3; it holds *base64(PEM) as raw bytes*, not a VARCHAR — so "UNIQUE on the b64 column" means UNIQUE on a bytea expression, see resolution §1). No later migration touches `public_keys` (verified: `20221021180809` is the only file in `oxidauth-postgres/migrations/` mentioning the table).
- **Defect 2 — post-commit decode (repo):** `oxidauth-postgres/src/public_keys/insert_public_key/mod.rs:13-24`. The `INSERT … RETURNING *` runs with `fetch_one(&self.db.write_pool())` (`:14-19`, auto-committed statement), and *only then* the response is built by `result.try_into()?` (`:21`) → `String::from_utf8(value.public_key)?` in `TryFrom<PgPublicKey> for PublicKey` (`oxidauth-postgres/src/public_keys/mod.rs:30-43`, decode at `:34`). Error type is `BoxedError` (`mod.rs:9`), so the raw `std::string::FromUtf8Error` escapes unmapped. The write is durable before the conversion that can fail — the DATA-3/OXA-000016 class ("errors *after* the damage is committed").
- **Register drift (confirmed, matches the pattern across the register):** the register cites `insert_public_key/:96,117`. Those lines are the two `BUG(pinned)` comment headers inside the test module — `:96-97` in `it_should_accept_identical_key_material_twice` (`:95-113`) and `:117-119` in `it_should_persist_the_row_but_error_when_public_bytes_are_not_utf8` (`:115-139`). The product defects are `insert_public_key/mod.rs:14-23` + `public_keys/mod.rs:34` + the migration DDL. These are the only `BUG(pinned)` markers for this item; neighboring markers belong elsewhere: `select_all_public_keys/mod.rs:52-53` (no `ORDER BY` → DATA-13), `select_public_key_by_user_id/mod.rs:41-43` (lying module name → DATA-11), `oxidauth-services/src/public_keys/list_all_public_keys.rs:160,174` (jwks `unwrap()` panics → SEC-12 / OXA-000012).
  - *(Implementation-time note 2026-09-29: the `select_public_key_by_user_id` module was renamed to `select_public_key_by_id` by OXA-000024 and its marker deleted; the path above is kept as review-time history.)*
- **The insert SQL:** `oxidauth-postgres/src/public_keys/insert_public_key/insert_public_key.sql:1-4` — `INSERT INTO public_keys (id, private_key, public_key, created_at, updated_at) VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, NOW(), NOW()) RETURNING *`. No `ON CONFLICT`, no timestamp guard.
- **The only production writer:** `oxidauth-services/src/public_keys/create_public_key.rs:37-49` — every call mints a *fresh* `KeyPair::new()?.base64_encode()` (`:38`) and inserts it (`:46-48`). Mounted as authenticated `POST /api/v1/public_keys` (`oxidauth-api/src/server/api/v1/public_keys/mod.rs:16`; handler `create_public_key.rs:18-58`, Err path → `Response::bad_request().error(err.into_error())` at `:56`). Wire DTO `PublicKey.public_key` is `String` (`oxidauth-kernel/src/public_keys/mod.rs:11-13`) — the response contract *is* the UTF-8 decode that runs after the commit.
- **Bootstrap writer (race):** `oxidauth-services/src/bootstrap/mod.rs:231-246` `first_or_create_public_key` = `list_all_public_keys` → if empty → `create_public_key` — a check-then-act with no lock, itself gated by the check-then-act `bootstrap` setting (`:93-105`, `:109-119`). Two instances booting against a cold database both see "no keys" and both create.
- **Duplicate-capable raw writers:** `oxidauth-postgres/src/test_fixtures.rs:195-207` (`seed_public_key` inserts arbitrary bytes straight into the table — calling it twice with the same slice produces duplicate rows at the SQL level), any operator `psql`/restore, and the hurl suite's `hurl/public_keys_create.hurl` (creates an extra keypair per suite run; self-cleaning per its `:1-4` comment, driven by `hurl.sh:74-75`). `oxidauth-import-export` and `src/seedz` contain no `public_keys` handling (verified by grep).
- **Same-table consumers (cross-refs):** `oxidauth-postgres/src/private_keys/select_most_recent_private_key/select_most_recent_private_key.sql` (jwt signing-key pick — **OXA-000019**, whose proposed fix edits the very same `insert_public_key.sql:4` for `NOW()` → `statement_timestamp()`), `oxidauth-postgres/src/public_keys/select_all_public_keys/` + `oxidauth-services/src/public_keys/list_all_public_keys.rs` (jwks listing / `ExtractJwt` — **OXA-000012**), `oxidauth-postgres/src/public_keys/select_public_key_by_id/` (PK lookup — DATA-11, renamed by **OXA-000024**). Precedent for the missing-UNIQUE class and its cleanup-migration shape: **OXA-000006** (`totp_secrets.user_id`); precedent for post-commit damage: **OXA-000016** (DATA-3, `update_authority`).

## Problem

`insert_public_key` has two independent write-path defects, both pinned by tests:

1. **No uniqueness anywhere in `public_keys`.** Re-inserting identical key material persists a second, third, … row; nothing — schema, SQL, or Rust — detects it. The pinned test asserts `COUNT(*) = 2` after two inserts of `b"same-pub"` (`insert_public_key/mod.rs:105-112`).
2. **The insert commits before the response-side decode.** `fetch_one` executes and auto-commits the statement; `String::from_utf8` then runs on the *returned* bytes (`public_keys/mod.rs:34`). When the stored public bytes are not UTF-8, the caller correctly gets `Err(FromUtf8Error)` — after the row is already durable. A naive retry of the same failing call appends another row every time: the error response is what makes the accumulation self-amplifying. The pinned test asserts the error *and* `COUNT(*) = 1` "despite the Err" (`insert_public_key/mod.rs:120-138`).

## Analysis

**Mechanism of the post-commit decode.** The response contract (`PublicKey.public_key: String`) forces a UTF-8 interpretation of the bytea column, but the code derives it from the DB round-trip (`:21` on `result`) instead of validating the *input* before writing. bytea is lossless, so the decode outcome is fully determined by the caller's bytes — yet the code only discovers it post-commit. `sqlx`'s auto-commit on `write_pool()` means there is no transaction to unwind; the row survives. This is exactly the DATA-3 shape recorded in OXA-000016: mutation lands, *then* the read-back path errors, so callers see failure with side effects persisted.

**Can the UTF-8 leg fire through the app today?** No — and that is precisely the trap. The only production writer (`create_public_key.rs:38`) passes base64 output (ASCII ⇒ always valid UTF-8). The failing conversion is reachable today only through the repo `Service` directly (test fixtures, future import/rotation code, any tool built on `oxidauth-postgres`) or through out-of-band SQL writes that this schema cannot reject. The defect is therefore *latent-but-structural*: the database and the repo layer both accept bytes their own read path cannot represent, and every future writer inherits the landmine.

**Duplicate sources, ranked by realism (verified in code):**

1. **Retry of a failing insert (same material).** The register's scenario: any caller passing constant bytes (import tool, seed re-run, fixture loop) plus a retry-on-Err loop. The non-UTF-8 path makes each retry a *guaranteed* Err + a committed row — a perfect accumulation machine. With valid UTF-8, a post-insert network failure *after* `RETURNING` (client dies before reading the response) plus retry leaves an extra identical row with no error at all.
2. **Bootstrap race (different material).** Two instances racing the cold-DB bootstrap (`bootstrap/mod.rs:235-241` list-then-create, with the `bootstrap` setting gate at `:93-105` also unlocked) both create a keypair. Note the distinction: these are *two different* keypairs, so material-uniqueness does **not** stop them — that race belongs to bootstrap locking (unregistered adjacent finding; OXA-000019 already documents the "at most one key when empty" intent). This ticket fixes the *identical-material* axis; §5 notes the residual.
3. **Out-of-band re-seed / restore re-runs.** `seed_public_key`-style raw inserts (`test_fixtures.rs:195-207`) or a restore script replayed twice produce byte-identical rows the schema shrugs at.
4. **HTTP-level retries of `POST /api/v1/public_keys`** do *not* produce identical duplicates (`KeyPair::new()` at `:38` runs per call) but do produce unbounded distinct-row growth; there is no cap or rotation policy in the repo. Out of scope here; noted so nobody "fixes" retries here expecting uniqueness to help.

**What uniqueness must cover.** The read paths key off the *public* half: jwks serves `public_key` (OXA-000012), verification brute-forces public keys, and `select_all_public_keys` would serve duplicate entries for duplicate rows (the `oxidauth-rs` client tolerates them, the listing just grows). So the invariant is: *one row per distinct public key material*. A `(public_key, private_key)` composite would miss the real-world case of the same public bytes replayed with dummy/different private bytes (`seed_public_key` inserts exactly dummy private material; `InsertPublicKeyParams` carries the two halves independently). Partial index is pointless — `public_key` is `NOT NULL`.

**BYTEA + UNIQUE mechanics (answers the register's "b64 column UNIQUE" question).** The stored material is base64(PEM) *bytes*, so a plain `UNIQUE (public_key)`/`CREATE UNIQUE INDEX … (public_key)` works (btree compares bytea byte-wise). RSA-2048 base64(PEM) SPKI is ~460 B and RSA-4096 ~800 B — inside the btree entry cap (~1/3 of an 8 KB page) — but a fixed-width **expression index on `sha256(public_key)`** (core `sha256(bytea)` since PostgreSQL 11; *not* pgcrypto, which is not installed — only `uuid-ossp`, per `migrations/20221019185650_create_uuid_ext.sql`) removes the size ceiling entirely at 32 B per entry. Hash-collision risk (≈2⁻¹²⁸ adversarial-free) is not a practical concern for key material; verify the deployment's PG major ≥ 11 at implementation time.

**Interactions and ordering with sibling tickets (cross-refs, not duplicates):**

- **OXA-000012 (SEC-12)** fixes the *read* side (skip+warn malformed rows in the jwks use case). This ticket closes the *write* side: pre-validation + schema constraints stop new bad/duplicate rows from landing at all. The two audits complement: SEC-12's reports malformed material, this ticket's dedups identical material. A deployment running both cleanups converges; running only one does not undo the other.
- **OXA-000019 (DATA-6)** edits the same file, `insert_public_key.sql:4` (`NOW()` → `statement_timestamp()`). Coordinate the single line; the unique index and dedup live in a *new* migration file, so there is no file conflict beyond that one line. The two bugs compound: duplicate rows written in one operator batch share both material *and* `NOW()`-stamp, deepening the "which key is signing" fog OXA-000019 describes; and `DELETE /public_keys/{id}` cleanup on identical rows is guesswork — different ids, indistinguishable payload.
- **OXA-000006 (SEC-6)** is the in-repo template for this exact shape (missing UNIQUE → cleanup migration → `assert_sql_state(err, "23505")` pin flips), and the repo's duplicate-error idiom is surfacing `sqlx::Error::Database` 23505, e.g. `oxidauth-postgres/src/permissions/insert_permission/mod.rs:92` via `test_fixtures::assert_sql_state` (`test_fixtures.rs:89-102`). No repo query maps 23505 to a domain type today; keep that convention for the duplicate leg.

**Edge cases:**
- Same `public_key`, *different* `private_key`: unique on material rejects the second insert (correct — the public key is already registered). Existing rows of this shape must not be silently deleted (each private half may be the only copy that signs); §1's migration splits auto-clean vs. report-only precisely for this.
- Empty `bytea` (`''`): valid UTF-8, would dedupe as one value — still junk key material; a `CHECK (octet_length(public_key) > 0)` hardening belongs with the same migration.
- The 23505 that a duplicate now produces *precedes* `RETURNING`, so no row is written and the post-commit window for the duplicate leg closes for free; the UTF-8 pre-validation closes the post-commit window for the encoding leg. Both legs fixed ⇒ `result.try_into()?` at `:21` becomes unreachable-fallible (keep it as defense-in-depth).
- Bootstrap race remains after this fix (different material) — unique on material cannot see it; do not claim otherwise in tests.
- Migration on a DB that already holds duplicates must *not* blindly `CREATE UNIQUE INDEX` (it aborts, which is a good thing — it forces cleanup first); ordering within the migration is audit → auto-clean safe class → index (fails loudly if the report-only class was ignored).

**Who is affected:** every deployment that seeds, restores, or scripts `public_keys` outside the API (the repo's own fixture helper proves this is the norm for key material); any future import/rotation feature built on `oxidauth-postgres` (there is none today — `oxidauth-import-export` and seedz verified negative — which is why P2 is honest); operators deduping jwks noise or deleting "one of" two identical rows; indirectly the auth stack, since duplicate/accumulated rows widen OXA-000019's ambiguity surface.

## Impact

- **Integrity/UX (P2 as registered):** duplicate key rows silently accumulate on any retry/replay loop; after a post-commit decode failure the *error itself* is the accumulation trigger. jwks listings serve duplicate entries; rotation and deletion decisions operate on a table whose row set no longer mirrors "one row per key".
- **Latent severity:** both legs are unreachable through today's single writer (fresh keypair, base64/ASCII), so this is not an active production incident — it is a missing invariant. The moment any second writer appears (the import-export crate is the obvious candidate), it becomes live with no further code change.
- **No confidentiality impact:** `private_key` is never served by any public_keys read path (`PgPublicSanitizedKey` projection, `public_keys/mod.rs:45-51`; verified by the sanitization assertion in `select_all_public_keys/mod.rs:77-80`); duplicates and the decode error move rows and error types, not secrets.

## Proposed resolution

**1. Schema: dedup + unique index on public key material** — new migration `oxidauth-postgres/migrations/<ts>_unique_public_keys_material.sql`, sequenced *after* any rows exist in the wild (sqlx runs migrations in one tx per file, so skip `CONCURRENTLY`; the table holds a handful of rows):

```sql
-- (a) auto-remove byte-identical duplicates (both halves equal): keeping any one
-- row preserves every verification/signing capability, since the payload is equal
DELETE FROM public_keys p
USING (
  SELECT id, min(id) OVER (PARTITION BY public_key, private_key
                           ORDER BY created_at, id) AS keep_id
  FROM public_keys
) d
WHERE p.id = d.id AND d.id <> d.keep_id;

-- (b) auto-remove public-only duplicates when the kept row's private half is
-- byte-equal? NO — public-dupes with *differing* private halves are NOT deleted
-- (each private half may be the only signing copy). Report them instead:
CREATE OR REPLACE FUNCTION public_keys_audit_public_dupes()
RETURNS TABLE (public_key BYTEA, ids UUID[]) AS $$
  SELECT public_key, array_agg(id ORDER BY id)
  FROM public_keys
  GROUP BY public_key
  HAVING count(DISTINCT private_key) > 1;   -- same pub, >1 distinct priv
$$ LANGUAGE sql;
-- run in the migration and RAISE NOTICE with results (report-only; never delete)

-- (c) the invariant itself
CREATE UNIQUE INDEX public_keys_public_key_key ON public_keys (sha256(public_key));
ALTER TABLE public_keys ADD CONSTRAINT public_keys_nonempty
  CHECK (octet_length(public_key) > 0);
```

If (b) finds rows, operators resolve them (delete via `DELETE /api/v1/public_keys/{id}` or accept one row and prune manually) before (c) will succeed; step (c) failing the migration is the enforced gate. Fallback if the target PG is < 11 (`sha256` unavailable): `CREATE UNIQUE INDEX … ON public_keys (public_key)` — bytea btree, fine for RSA-4096-class material. (`digest()`+pgcrypto would work but pgcrypto is not installed; don't add an extension for this.)

**2. Repo: decode-before-write in `PgPublicKeyRepository::call`** (`oxidauth-postgres/src/public_keys/insert_public_key/mod.rs:13-24`) — validate the *input* before touching the database, so a rejected insert is a zero-side-effect failure:

```rust
async fn call(&self, params: &'a InsertPublicKeyParams) -> Result<Self::Response, Self::Error> {
    std::str::from_utf8(&params.public_key)
        .map_err(|_| InvalidPublicKeyMaterial)?;  // borrow, no clone; write never happens on Err

    let result = sqlx::query_as::<_, PgPublicKey>(include_str!("./insert_public_key.sql"))
        /* unchanged */ .fetch_one(&self.db.write_pool()).await?;
    let public_key = result.try_into()?;          // now unreachable-fallible; keep as defense
    Ok(public_key)
}
```

Pre-validation over a `begin()/commit` wrapper: the decode outcome depends only on the caller's bytes (bytea is lossless), so checking the input is strictly stronger than the transaction trick — it also saves the round-trip and makes the duplicate leg (23505) moot for ordering. No transaction plumbing needed.

**3. Typed error for the encoding leg.** Add to `oxidauth-kernel/src/public_keys/mod.rs` (next to `PublicKey`, `:11-15`) a unit error, e.g. `#[derive(Debug, thiserror::Error /* or manual impl */)] #[error("public key material is not valid UTF-8")] pub struct InvalidPublicKeyMaterial;` — `std::error::Error + Send + Sync`, carried in the existing `BoxedError` so the trait signatures (`oxidauth-repository/src/public_keys/insert_public_key.rs:5-13`) don't churn; callers downcast instead of matching on `std::string::FromUtf8Error`. Wire shape is unchanged: the Err path already answers `Response::bad_request().error(err.into_error())` (`oxidauth-api/src/server/api/v1/public_keys/create_public_key.rs:56`). The *duplicate* leg deliberately stays the raw `sqlx::Error::Database` 23505, matching the repo-wide convention (roles/permissions/grants); no mapping layer.

**4. Retry-idempotency variant (decision point, not both):** if insert callers should get retry-safe no-ops instead of 23505, make the SQL an upsert on the new index — `ON CONFLICT (sha256(public_key)) DO UPDATE SET updated_at = public_keys.updated_at RETURNING *`. This converts the entire retry-accumulation class into a benign re-read at the cost of making "created" vs "already existed" indistinguishable to callers. The repo's established idiom is explicit 23505 (`assert_sql_state` used across eight insert test modules, all the grant tables plus permissions, user_authorities, refresh_tokens, totp_secrets), so **recommendation: plain INSERT + 23505** (steps 1-3); adopt the upsert only if a concrete import use case demands idempotent writes.

**5. Out of scope, note-don't-fix here:** the bootstrap cold-start race (list-then-create with unlocked setting gate, `bootstrap/mod.rs:93-105, 235-241`) produces *distinct*-material duplicates no unique constraint can catch; it needs a bootstrap advisory lock or `ON CONFLICT` on the `bootstrap` setting row, and is not in the register — flag it to Main as an unregistered adjacent finding rather than silently absorbing it.

**Pin flips** (both in `oxidauth-postgres/src/public_keys/insert_public_key/mod.rs`; these are the item's only `BUG(pinned)` markers):
- `it_should_accept_identical_key_material_twice` (`:95-113`): delete the marker (`:96-97`), rename to `it_should_reject_duplicate_public_key_material`; flip `repo.call(&params).await.expect("duplicate insert is accepted")` (`:106`) to capture the error and `assert_sql_state(err, "23505")` (helper: `test_fixtures.rs:89-102`), and flip the count assertion `assert_eq!(count, 2)` (`:112`) to `assert_eq!(count, 1, "only the original row persists")`.
- `it_should_persist_the_row_but_error_when_public_bytes_are_not_utf8` (`:115-139`): delete the marker (`:117-119`), rename to `it_should_reject_non_utf8_public_bytes_without_writing`; keep the error leg but flip the downcast at `:130` from `std::string::FromUtf8Error` to `InvalidPublicKeyMaterial`, and flip `assert_eq!(count, 1, "the row is persisted despite the Err")` (`:138`) to `assert_eq!(count, 0, "a rejected insert writes nothing")`.
- Add two tests (not pins): *distinct material still inserts twice* — `assert_eq!(count, 2)` for two different payloads, guarding that the index constrains duplicates, not the table; *same public bytes, different private bytes is rejected* — the public-only duplicate leg of the invariant.

**Compat/migration concerns:** no API contract change — the sole production writer always generates fresh valid base64, so 23505 and the UTF-8 error are unreachable through `POST /api/v1/public_keys` (success and Err wire shapes byte-identical). Existing databases holding byte-identical duplicates converge automatically at migration time; public-dupes-with-differing-private rows block the index until operator action (intentional — deleting the wrong one strands a signing key). Coordinate the one-line touch to `insert_public_key.sql:4` with OXA-000019's `statement_timestamp()` change; whichever ticket lands second rebases on it. No import-export/seedz surface to migrate (verified negative); the `oxidauth-rs` client never inserts keys.

## Verification

- `cargo test -p oxidauth-postgres public_keys::insert_public_key` (requires `DATABASE_URL` per the `#[sqlx::test(migrator = "crate::MIGRATOR")]` convention): both flipped tests + the two new tests pass; run the flipped tests **before** the code change to confirm they fail against today's behavior (regression proof — the old pins would fail the new assertions).
- Post-commit-window repro (the acceptance scenario), against a live DB: call the repo with `public_key: vec![0xFF, 0xFE]` via a throwaway `#[sqlx::test]` or psql-side wrapper; before: `Err` + `SELECT COUNT(*) FROM public_keys` grows by 1 per attempt; after: `Err(InvalidPublicKeyMaterial)` + count unchanged across three retries.
- Unique-index repro: `INSERT INTO public_keys (public_key, private_key) VALUES ('dup','p1'); INSERT … VALUES ('dup','p1');` — second statement raises `unique_violation` (23505); then the `(dup, p2)` shape is also rejected by the index while the audit function names it.
- Migration on seeded data: a repo test that pre-inserts byte-identical duplicates via raw SQL (bypassing the constraint by running inside the migration test transaction *before* index creation is not possible — so: run migrations on a DB seeded through `sqlx::query` between migrate steps, mirroring OXA-000006's cleanup-migration test shape) asserts duplicates gone + index present; a second case seeds a `(same pub, different priv)` row and asserts the migration aborts (or reports, if the index step is split per §1).
- `cargo test -p oxidauth-services public_keys::create_public_key bootstrap` — mock-based use-case tests untouched (they never run SQL), proving no consumer signature churn from the typed error.
- `cargo test -p oxidauth-postgres private_keys::select_most_recent_private_key public_keys` — neighboring same-table tests (`RowNotFound`, newest-wins, sanitized-projection) unaffected by the index.
- `hurl --test src/oxidauth/hurl/tests/public_keys.hurl` and `hurl/public_keys_create.hurl` against a running server — create/list/find/delete contract and bootstrap key behavior unchanged (create still mints fresh material per call).
- Confirm marker removal: `grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-postgres/src/public_keys/insert_public_key/` returns nothing (the remaining `public_keys/` markers belong to DATA-11/DATA-13 and must stay).
