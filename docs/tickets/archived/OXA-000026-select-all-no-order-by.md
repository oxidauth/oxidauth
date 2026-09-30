# OXA-000026 — list-all queries have no ORDER BY: row order in /users, /authorities, /public_keys responses is unspecified

**Original ID:** DATA-13 · **Severity:** P3 · **Type:** bug · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #20 of 27 · reviewed 2026-09-29 · SCHEDULED — scope FOLDED to all five SELECT-all queries


## Locations

All paths relative to `src/oxidauth/`.

- **Defect (all three SQL files are the whole query, no ORDER BY):**
  - `oxidauth-postgres/src/users/select_all_users_query/select_all_users_query.sql:1-2` — `SELECT * FROM users`
  - `oxidauth-postgres/src/authorities/select_all_authorities/select_all_authorities.sql:1-2` — `SELECT * FROM authorities`
  - `oxidauth-postgres/src/public_keys/select_all_public_keys/select_all_public_keys.sql:1-6` — `SELECT id, public_key, created_at, updated_at FROM public_keys`
- **Register drift (confirmed, same pattern as every prior item):** the cited lines `:49`, `:67`, `:52` are the first lines of the three `BUG(pinned)` comments inside the *test modules* of the sibling `mod.rs` files — `users/select_all_users_query/mod.rs:49-50`, `authorities/select_all_authorities/mod.rs:67`, `public_keys/select_all_public_keys/mod.rs:52-53` — not the product defect. The product defect is the three `.sql` files. (Verified: these are the only `BUG(pinned)` markers under the three query directories.)
- **Executors:** each `mod.rs` runs its `.sql` via `sqlx::query_as(...).fetch_all(&self.db.read_pool())` (users `mod.rs:13-15`, authorities `mod.rs:12-14`, public_keys `mod.rs:17-19`) and maps rows in iterator order. Read pool = `READ_DATABASE_URL` replicas when configured, falling back to the write pool (`oxidauth-postgres/src/lib.rs`, documented in OXA-000019).
- **Service layer is order-transparent by design:** `oxidauth-services/src/users/list_all_users.rs`, `.../authorities/list_all_authorities.rs`, `.../public_keys/list_all_public_keys.rs` are pure delegators (public_keys only base64-remaps each row). The users service test `the_stored_list_is_returned_in_query_order` pins this explicitly: "the service must not re-sort or filter the query result." So **the SQL is the only place row order is decided**, and it decides nothing.
- **HTTP surface:** `GET /api/v1/users`, `GET /api/v1/authorities`, `GET /api/v1/public_keys` (`oxidauth-api/src/server/api/v1/{users,authorities,public_keys}/mod.rs` router entries) serialize the `Vec` straight into `ListAllXRes { ...: Vec<_> }` (`oxidauth-http/src/*/list_all_*.rs`). **No pagination exists**: `ListAllUsers` and `ListAllPublicKeys` are unit structs and `ListAllAuthorities {}` is an empty struct (`oxidauth-kernel/src/*/list_all_*.rs`), aliased as the request types (`pub type ListAllXReq = ListAllX`); a repo-wide grep for offset/limit/page request params finds only `NbfOffset` noise. The register's "pagination" concern is therefore prospective, not current.
- **Public-key consumers (both order-insensitive):**
  - `oxidauth-api/src/middleware/permission_extractor.rs:36-41` — fetches all keys and iterates to verify the bearer JWT.
  - `oxidauth-rs/src/axum/extract/mod.rs:54-63` — `Jwt::decode_with_public_keys` brute-forces every served key (`oxidauth-kernel/src/jwt/mod.rs:68-81`; tokens carry no `kid`, per OXA-000019), so verification succeeds regardless of array order.
  - **There is no `/jwks` (or `/.well-known/*`) route anywhere in `oxidauth-api`** (grep: zero hits) — the "jwks endpoint" in the public_keys test comment refers to this client-side JWKS-style decode path, not a route.
- **No in-app export touches these tables:** `oxidauth-import-export` is an empty scaffold (`src/lib.rs` is 14 lines: a placeholder `add()`). The register's "export" concern is operator/DBA territory or future code.
- **Ordering columns exist:** all three tables have `id UUID PRIMARY KEY`, `created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP`, `updated_at` likewise (migrations `20221019185709_create_users.sql`, `20221020180349_create_authorities.sql`, `20221021180809_create_public_keys.sql`), and all three kernel entities expose `id` + `created_at` (`oxidauth-kernel/src/{users,authorities,public_keys}/mod.rs`).

## Problem

Every full-table read in the repository relies on Postgres's *unspecified* row order. Per the SQL standard and Postgres docs, without `ORDER BY` the database may return rows in any order, and which order it picks depends on the plan (seq scan vs index scan), page layout, `VACUUM`/page-reuse history, `ANALYZE` statistics, and — in this codebase, which reads through a separate `read_pool()` — *which replica* executed the query. Two identical `GET /api/v1/users` calls can legitimately return the JSON array in different orders with no write in between.

The three `BUG(pinned)` markers document that the authors knew this and downgraded their tests to set assertions (`sort()` before `assert_eq!`) instead of fixing the SQL. The defect is real but low-grade: no *correctness-sensitive* consumer depends on order today (JWT verification is set-like, hurl asserts are `count >=` only), so this is an API-determinism and future-proofing bug, matching its P3 rating.

## Analysis

**Why order leaks all the way to the wire:** SQL row order → `fetch_all` iterator order → `Vec` → serde JSON array order. Nothing in the chain re-sorts (and the service layer is *pinned* not to). So the JSON array order in three public/authenticated endpoints is nondeterministic by construction. Observable consequences: byte-unstable responses (breaks ETag/checksum/snapshot comparisons, response caching, HTTP-level diffs), admin UIs whose row order can shuffle between refreshes, and multi-replica deployments where different instances return different orders from identical data.

**The `created_at`-alone trap — this is OXA-000019's tie problem verbatim:** `CURRENT_TIMESTAMP`/`NOW()` is the *transaction start time*, constant within a transaction. So any multi-row seed or bulk INSERT in one transaction stamps every row with an identical `created_at` — including the `#[sqlx::test]` fixtures here (sqlx runs the test body, and thus all `seed_user`/`seed_authority`/`seed_public_key` inserts, inside one wrapper transaction, so test rows tie by construction). `ORDER BY created_at` alone would therefore still return an arbitrary permutation among ties, and the *tests could not even predict it* because the seed helpers let `uuid_generate_v4()` pick ids server-side. A total order requires the `id` tie-break; Postgres compares `UUID` byte-wise and Rust's `Uuid: Ord` uses the same byte order, so the tie-break is expressible on both sides. This is exactly the resolution convention established in **OXA-000019**: `(created_at, id)`, never `updated_at` (mutable by unrelated UPDATEs — also pinned there as a wrong orderer at `select_most_recent_private_key/mod.rs:84-95`).

**Direction is a per-query UX choice; totality is the invariant.** OXA-000019 orders `DESC` because "most recent" is its semantics. For full listings, `ASC` (oldest-created first) is the boring default and pairs naturally with keyset pagination later; pick one direction for all three endpoints and document it. What must *not* vary is the composite `(created_at, id)` pair.

**Why the `id` tie-break also matters for jwks ordering-independence:** verification ignores order, but *which key is newest* matters for rotation hygiene (OXA-000019); a stable `(created_at, id)` listing at least makes the served set's implied ordering deterministic for forensics, even though decode order is irrelevant.

**Prospective pagination is the sharper edge:** the register mentions pagination — none exists today (unit-struct requests), but the moment anyone adds `OFFSET/LIMIT` to these queries, an unordered base makes pages *skip and duplicate rows arbitrarily* across pages, and adding `ORDER BY created_at` without the `id` tie-break still breaks (ties straddle page boundaries nondeterministically). Fixing the total order now makes `(created_at, id)` the keyset cursor later. Note `ORDER BY` alone does *not* make `OFFSET` pagination correct under concurrent inserts — keyset pagination is the eventual design; `ORDER BY` is its prerequisite.

**Related register items (cross-refs, not duplicates):**
- **OXA-000019** (DATA-6) — same tables, same `NOW()`-tie mechanism, source of the `(created_at, id)` convention and the `statement_timestamp()` companion fix for the public_keys *writer*; if that fix lands, app-written keys get distinct `created_at`, but default-timestamp/out-of-band seeds still tie → `id` tie-break stays required here.
- **OXA-000022** (DATA-9) — `LIMIT 1` without `ORDER BY` in `select_authority_by_strategy`: same "arbitrary row" family, single-row symptom, P2 there because bootstrap attaches the admin to the wrong authority. Do not merge scopes.
- **OXA-000012** (SEC-12) — jwks listing panics on a malformed key row (`list_all_public_keys.rs` service decode, its own `BUG(pinned)` at `oxidauth-services/src/public_keys/list_all_public_keys.rs:160,174`); same consumer chain (`ListAllPublicKeys`), disjoint defect.

## Impact

- **Who is affected:** consumers of `GET /api/v1/users`, `/api/v1/authorities`, `/api/v1/public_keys` — today that includes the server's own JWT middleware and every `oxidauth-rs` client (via `ExtractJwt`), plus operator tooling. No *known* consumer breaks; the damage is nondeterminism, not failure.
- **What can go wrong concretely:** response bytes vary run-to-run (defeats caching/diffing/snapshot assertions); row order changes after routine `VACUUM`/`ANALYZE`/plan changes with no deploy; read replicas can disagree; future `OFFSET` pagination built on this silently drops/duplicates rows.
- **Severity:** P3 as registered is honest — this is determinism/hardening, no security or availability impact (verification is set-like by design; see OXA-000019 for why that remains true only while all keys stay in the table, which is that ticket's problem).

## Proposed resolution

**1. Add a total order to all three SELECTs** (matching the files' existing no-semicolon style):

```sql
-- select_all_users_query.sql
SELECT *
FROM users
ORDER BY created_at ASC, id ASC
```

```sql
-- select_all_authorities.sql
SELECT *
FROM authorities
ORDER BY created_at ASC, id ASC
```

```sql
-- select_all_public_keys.sql
SELECT id, public_key, created_at, updated_at
FROM public_keys
ORDER BY created_at ASC, id ASC
```

`created_at` is `NOT NULL` in all three tables — no `NULLS LAST` needed. Direction (`ASC`) is the recommended listing default; if an admin UX wants newest-first, flip all three to `DESC` together and note the choice in the API docs — the invariant is `(created_at, id)` totality, `id` never omitted, `updated_at` never used.

**2. Flip the three pins** — delete the `BUG(pinned)` comments at `users/select_all_users_query/mod.rs:49-50`, `authorities/select_all_authorities/mod.rs:67`, `public_keys/select_all_public_keys/mod.rs:52-53`, and strengthen each test from set-assertion to order-assertion. Because the fixture rows tie on `created_at` (single `sqlx::test` transaction, §Analysis) with server-generated random ids, the assertions cannot hardcode a sequence without changing the seed helpers; assert the invariant instead, on the returned kernel entities (all three expose `id` + `created_at`):

```rust
assert!(
    result.windows(2)
        .all(|w| (w[0].created_at, w[0].id) <= (w[1].created_at, w[1].id)),
    "rows must be returned in (created_at, id) order"
);
```

Keep the existing sorted-set content assertions (they check *which* rows, still needed), and additionally call the service twice and assert identical id sequences — pinning plan-independent determinism. (Alternative: make the seed helpers `RETURNING id` and assert the exact expected sequence; nicer, but touches `test_fixtures.rs` shared by other tests — not required.)

No test *fails* if step 1 ships without step 2 — the set assertions sort before comparing, so they pass under any order — and no test pins order-*freeness* as desired behavior (verified: the only order-adjacent pins are the three markers themselves; `the_stored_list_is_returned_in_query_order` in `oxidauth-services` asserts the service layer must *not* re-sort, which stays true and is why the fix goes in SQL). But leaving the markers in place after the fix would be a lie, so they go.

**3. Document jwks non-sensitivity; don't oversell:** per RFC 7517 §5 the JWKS `keys` array is unordered, and both consumers here iterate/brute-force regardless (no `kid`; `decode_with_public_keys`). This change buys wire determinism and pagination groundwork, **not** a verification fix. Say exactly that in the commit message so nobody closes OXA-000012/OXA-000019 as "related, handled."

**4. Do not add pagination now.** The unit-struct request types stay untouched (no compat surface). The `ORDER BY` above is the prerequisite; the eventual design is keyset pagination on `(created_at, id)` in a future ticket if the need materializes.

**5. No index, no migration.** All three tables are operator-scale (tens–hundreds of rows); the sort is trivial. If any table ever grows large enough to care, `CREATE INDEX ... (created_at, id)` is the follow-up, not a prerequisite.

**Compat:** the only observable change is that JSON array order becomes deterministic where it was previously undefined — no client may legitimately depend on the old non-order. No schema, type, route, or response-shape change; `oxidauth-rs`, `oxidauth-http`, `oxidauth-kernel` untouched.

## Verification

- `cargo test -p oxidauth-postgres select_all` (requires `DATABASE_URL`; `#[sqlx::test(migrator = "crate::MIGRATOR")]` convention): the three strengthened tests pass with the windows-sorted assertion and double-call determinism check. Prove the new assertions actually bite: run them against the unfixed SQL with tied-timestamp seeds — order comes out heap-order, not `(created_at, id)` order, whenever the plan/heap disagrees (and the double-call check documents why exact sequences aren't assertable pre-fix); they're red against any wrong orderer such as `updated_at`-first (mirror the OXA-000019 guard-test reasoning).
- Live check against Postgres: `EXPLAIN SELECT * FROM users ORDER BY created_at, id;` shows a `Sort` node; run the three fixed queries twice after `VACUUM FULL` on a seeded DB — identical id sequences.
- `cargo test -p oxidauth-services list_all` — all mock-based delegate tests green unchanged (proves zero consumer churn; they never execute SQL).
- `cargo test -p oxidauth` — client contract tests (mock server JSON) green unchanged. (Package is `oxidauth`; `oxidauth-rs` is the directory, not a package ID.)
- `hurl --test src/oxidauth/hurl/tests/users.hurl src/oxidauth/hurl/tests/authorities.hurl src/oxidauth/hurl/tests/public_keys.hurl` against a running server — the `count >= 2` / `count >= 1` list asserts stay green (they assert no order, so they must).
- Marker hygiene: `grep -rn "BUG(pinned)" src/oxidauth/oxidauth-postgres/src/users/select_all_users_query src/oxidauth/oxidauth-postgres/src/authorities/select_all_authorities src/oxidauth/oxidauth-postgres/src/public_keys/select_all_public_keys` returns nothing after the flip.

## Decision (2026-09-29) — ACCEPTED WITH SCOPE EXPANSION (five queries), scheduled; no implementation started
- Reviewed jointly (scout sweep + reviewer spot-checks + owner ruling 2026-09-29). Three registered queries confirmed (SQL files, all three `BUG(pinned)` markers, order-transparent service delegators, no pagination — unit/empty request structs). **Scope ruling: fold in the two unregistered SELECT-alls** — `permissions/select_all_permissions.sql` and `roles/select_all_roles.sql` have the identical defect (sort-before-assert tests, NO markers, no register lines); one-line cost each; one mechanism, one commit. DATA-13's register line gets REWRITTEN to name all five tables — no phantom marker deletes for permissions/roles.
- The composite is the point: `ORDER BY created_at ASC, id ASC` on all five. `created_at` alone is a trap — `NOW()`/`CURRENT_TIMESTAMP` is transaction-start, so every `#[sqlx::test]` fixture ties by construction; the `id` tie-break (the OXA-000019 convention) is what buys totality. Direction: ASC for all five; flip together only with an API-doc note. `updated_at` never used.
- Implementation erratum (fix round, 2026-09-29): "`NOW()`/`CURRENT_TIMESTAMP` … every `#[sqlx::test]` fixture ties by construction" was FALSE as a mechanism — `#[sqlx::test]` hands the body a real pool and autocommit INSERTs tick `NOW()` per statement, so ties were coincidental. Consequence, caught in review: default-default seeds did NOT redden an `updated_at ASC, id ASC` orderer. Each strengthened test now FORCES the tie after seeding (one UPDATE: pin all `created_at` to the table max, drop the max-id row's `updated_at` 1h below it), proving red against the wrong orderer in all five modules, green restored. The composite `(created_at, id)` ruling stands unchanged.
- Test flips: delete the three markers, convert the three tests to the invariant form (`windows(2)` tuple-monotonic on `(created_at, id)`) + double-call identical-id-sequences determinism check; keep the sorted-set content asserts. The two folded-in queries' tests get the same strengthening (no markers to delete there). No hardcoded sequences — fixture ids are server-generated.
- **Reviewer-verified bootstrap interaction** (not in the ticket): `oxidauth-services/src/bootstrap/mod.rs:239` does `public_keys.pop()` — `Vec::pop` takes the LAST row, so ASC ⇒ bootstrap deterministically picks the NEWEST signing key at boot, vs. arbitrary-today. This is an improvement; say so in the commit message so nobody "fixes" it back. (The scout's first pass claimed "oldest" — wrong, `pop()` takes last; ruling here reflects the verified reading.) OXA-000019's rotation story keeps this safe.
- Boundaries kept: no pagination (prerequisite only; keyset on `(created_at, id)` is the future design), no index (operator-scale tables), no service-layer sorting (`the_stored_list_is_returned_in_query_order` stays true — which is exactly why the fix is SQL), no claims against OXA-000012/OXA-000019 (verification is set-like; commit message must not oversell). N-6/can-cache chatter has zero bearing here — OXA-000056 owns that premise.
- Compat: array order becomes deterministic where it was undefined; no client may legitimately depend on non-order. Zero shape/route/type change.
- Acceptance: `cargo test -p oxidauth-postgres select_all` — five strengthened tests green, red against any wrong orderer (e.g. `updated_at`-first); `cargo test -p oxidauth-services list_all` + `cargo test -p oxidauth` green untouched (mocks never execute SQL); hurl `count >=` asserts green; the three markers gone by grep; live double-run after `VACUUM FULL` returns identical id sequences.
