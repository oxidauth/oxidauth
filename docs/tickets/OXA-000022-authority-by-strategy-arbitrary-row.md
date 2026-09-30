# OXA-000022 — select_authority_by_strategy: `LIMIT 1` without ORDER BY returns an arbitrary authority; bootstrap can attach the admin to the wrong one

**Original ID:** DATA-9 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; claims verified intact at HEAD — `LIMIT 1` w/o ORDER BY since 20240523190325 dropped strategy UNIQUE; bootstrap admin binding is plan-luck, ties are the normal seeded/restore case. Oldest-wins + smallest-id recipe (restore-stable, no Vec churn, additive plural query + bootstrap warn) sound — revisit before multi-authority installs; name-based bootstrap lookup (TODO at bootstrap/mod.rs:380) is the separate long-term identity query)


## Locations

All paths relative to `src/oxidauth/` unless noted.

- **Register line** (`BUGS_AND_NOTES.md:41`): `oxidauth-postgres/src/authorities/select_authority_by_strategy/:70` — "`LIMIT 1` without ORDER BY — with multiple matches one arbitrary row is returned and the caller cannot tell more exist."
- **Register drift (confirmed, matches the pattern from OXA-000016/OXA-000019):** the quoted `:70` is the first line of the `BUG(pinned)` comment inside the test module (`select_authority_by_strategy/mod.rs:70-72`), not the product defect; the crate tree also moved under `src/oxidauth/`. The actual defect is `oxidauth-postgres/src/authorities/select_authority_by_strategy/select_authority_by_strategy.sql:3-4` — `WHERE strategy = $1 LIMIT 1` with no `ORDER BY`. Verified: that marker is the **only** `BUG(pinned)` hit in the module (`grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-postgres/src/authorities/select_authority_by_strategy/` → 1 hit).
- **Defect SQL:** `select_authority_by_strategy.sql:1-4` (`SELECT * FROM authorities WHERE strategy = $1 LIMIT 1`).
- **Repository call:** `select_authority_by_strategy/mod.rs:15-19` — `query_as::<_, PgAuthority>` + `fetch_optional(&self.db.read_pool())`, `Response = Option<Authority>` (`:8`). Read-pool note: replicas plan independently (see OXA-000019 §Locations), so *which* replica answers can change the answer.
- **Pinned test:** `it_should_return_exactly_one_of_multiple_matches` (`mod.rs:56-80`): seeds two `oauth2` authorities (`:59-60`), then asserts only the disjunction `found.name == "oauth_one" || found.name == "oauth_two"` (`:73-77`) — the only sound assertion against an unordered `LIMIT 1`. Companion test `it_should_map_strategy_enum_and_return_none_when_absent` (`:83-112`) pins enum mapping and `None`-when-absent (invariants, not pins).
- **Multiplicity became legal at:** `migrations/20221020180349_create_authorities.sql:6` had `strategy VARCHAR(64) NOT NULL UNIQUE`; `migrations/20240523190325_remove_unique_strategy_from_authorities.sql:1-2` dropped `authorities_strategy_key` with no replacement constraint or index (noted in OXA-000016's migration context). `name` remains UNIQUE (`create_authorities.sql:3`), so a duplicate-strategy row just needs a different name — and **no application-level guard exists**: `oxidauth-services/src/authorities/create_authority.rs` has no strategy-uniqueness check (verified: no existence/strategy check anywhere in the use case), so `POST /api/v1/authorities` freely creates N authorities per strategy.
- **Caller 1 — bootstrap first-or-create (the sharpest edge):** `oxidauth-services/src/bootstrap/mod.rs:376-435` (`first_or_create_authority`), invoked at `:162-171`. Runs whenever the `bootstrap` setting row is absent (`:101-105`, key at `:207`). `Ok(authority)` ⇒ attach; `Err(Box<AuthorityNotFoundError>)` ⇒ create `DEFAULT_USERNAMEPASSWORD_NAME` (`:373`, `:415-427`). The chosen authority's `client_key` is what the admin registers under (`:473-476` in `first_or_register_user`), and once the admin user exists, every later bootstrap returns it early and never re-picks (`:452-453`).
- **Caller 2 — service/HTTP:** `oxidauth-services/src/authorities/find_authority_by_strategy.rs:33-44` maps `Ok(None)` → `AuthorityNotFoundError::strategy`; handler `oxidauth-api/src/server/api/v1/authorities/find_authority_by_strategy.rs:39-53` on `GET /api/v1/authorities/by_strategy/{strategy}` (router `mod.rs:21-24`) returns a single `authority` object (`oxidauth-http/src/authorities/find_authority_by_strategy.rs:10-12`); hurl pins 200-with-pinned-key and 400-when-absent (`hurl/tests/authorities.hurl:25-39`). Client SDK mirror: `oxidauth-rs/src/client/authorities/find_authority_by_strategy.rs:33-38`. Wiring: `oxidauth-api/src/provider/services.rs:610-618`. No other product callers (`oxidauth-import-export`/`oxidauth-cli` never reference it — grep-verified across `src/oxidauth`).
- **Downstream settings consumers:** every login reads `authority.settings.totp` / `jwt_ttl` / `entitlements_encoding` / `refresh_token_ttl` (`oxidauth-services/src/auth/authenticate.rs:152-247`) for the authority the user is linked to.
- **Provenance:** `missing-tests.md:61, 216, 326`; `:337` records the disjunction as "mod.rs:90" — drifted; it is now `:73-77`.

## Problem

`select_authority_by_strategy.sql` promises "the authority for this strategy" but only enforces "some authority for this strategy, if any." Since migration 20240523190325 made strategy reuse legal, `strategy` is a non-unique column queried like a key, and Postgres is free to return **any** matching row: which one it returns depends on the physical row order and plan (seq-scan short-circuit vs any future index on `strategy`, VACUUM/page reclamation, ANALYZE stats — and which replica answered), none of which the application controls. Nothing in the response tells the caller that other matches exist.

The pinned test proves the ambiguity is real and intended-green today: with two `oauth2` rows seeded, it can only assert "one of the two" (`mod.rs:73-77`). Under `#[sqlx::test]` (whole test in one transaction) both seeds even share an identical `created_at` by `NOW()`-per-transaction semantics, so there is zero ordering information at all — the exact tie mechanism OXA-000019 analyzed for `public_keys`.

## Analysis

**Why this is the same class as OXA-000019 but with a different blast radius.** OXA-000019's arbitrary pick chooses a signing key among tied keys; this one chooses a *policy object* — and it is baked into persistent state at bootstrap, not re-evaluated per request.

**Bootstrap nondeterminism (the register's "may attach to the wrong authority").** `first_or_create_authority` (`bootstrap/mod.rs:382-389`) treats "found" as "the default username_password authority exists — use it". If the table holds ≥2 `username_password` rows when bootstrap runs, which one the admin gets registered under (`:473-476` pass `authority.client_key` to `register`) is plan luck. Consequences of the wrong pick, all silent:

- The admin's `user_authorities` link is written to that authority, and **stays there forever** — `first_or_register_user` returns the existing user on every later run (`:446-453`), so re-running bootstrap never re-picks, and "fixing" the data later requires raw SQL to move the link row.
- All admin tokens thereafter inherit that authority's `settings`: `jwt_ttl`, `refresh_token_ttl`, `entitlements_encoding`, and crucially `totp` (`authenticate.rs:152-247`). Attaching to an operator's TOTP-enforcing authority silently 2FA-gates the bootstrap admin; attaching to one with a 2-minute `jwt_ttl` instead of the default 48h refresh makes the admin integration break on expiry. The intended default (created via `:407-413`: `DEFAULT_JWT_TTL`, `TotpSettings::Disabled`, `OXIDAUTH_DEFAULT_CLIENT_KEY`) is bypassed entirely — the env-var client key is consulted only on the *create* branch (`:397-402`), so an arbitrary pre-existing row wins over the operator's configured default key.
- `register` does **not** gate on authority `status` (verified: `oxidauth-services/src/auth/register.rs` checks only `jwt_nbf_offset`, `:125-127`), so a pick of a `Disabled` username_password authority succeeds silently — the admin is now permanently parked on a disabled authority, which every status-respecting flow will later reject. [INFERENCE: no in-tree flow re-checks authority status on login was located; the claim is limited to "register does not reject it".]

**When can ≥2 `username_password` rows exist at bootstrap time?** All realistic paths are operator data, not app bugs (first-boot multiplicity needs pre-seeded rows):
1. Seed/restore: SQL seed or single-transaction restore inserts several username_password authorities (e.g. a default plus a tenant one) while the `settings` bootstrap row is absent — re-bootstrap then re-picks, with all seeds sharing one `NOW()` stamp (maximum ambiguity).
2. Crash-recovery: bootstrap failed after `first_or_create_authority` (`:170`) but before `save_bootstrap_setting` (`:200`); the next boot repeats the pick, and any second row added in between (operator SQL) enters the lottery.
3. Post-bootstrap HTTP creates are irrelevant to bootstrap (setting already saved) but are exactly how *multiple* rows accumulate for the `GET /by_strategy` consumer.

**The HTTP caller can't tell either.** `GET /authorities/by_strategy/oauth2` returning 200 with one authority is indistinguishable whether one or five exist; the only multiplicity audit an operator has is `GET /authorities` (unfiltered, and itself unordered — register DATA-13, cross-ref only). The error contract also hides partial truth: the absent case is a 400, not a 404 (`hurl/tests/authorities.hurl:36-39`), so the endpoint has no vocabulary for "more than one."

**Interaction with OXA-000016 (poisoned rows):** if one matching row has `client_key IS NULL` (the OXA-000016 corruption), decoding into non-`Option` `PgAuthority.client_key` makes `fetch_optional` itself fail with `Decode` — so *whether bootstrap hard-fails at `:428-431` or proceeds* depends on which row the unordered `LIMIT 1` happened to pick. Same nondeterminism, different failure mode. [INFERENCE from OXA-000016's verified decode analysis; not exercised in a test here.]

**The code itself admits the misuse:** `bootstrap/mod.rs:380-381` — `TODO(dewey4iv): we should swap this for something that can pull the authority by the name`. By-strategy was always a stand-in for identity lookup (`name` is UNIQUE and is the real identity); strategy multiplicity broke the premise, and the query grew a `LIMIT 1` that pretends the premise still holds.

**Rejected shortcuts:**
- `updated_at` as an ordering source: mutable by unrelated UPDATEs, says nothing about "which is the default" — same rejection and reasoning as OXA-000019 (`select_most_recent_private_key/mod.rs:84-95` pins it).
- `ctid`/`xmin`-style physical order: unstable across VACUUM/page reuse — rejected for the same reasons as OXA-000019 §Edge cases.
- DB-side "one authority per strategy" re-add: reverses the deliberate 20240523190325 change (multi-strategy deployments are the point).

## Impact

- **Who is affected:** every deployment whose `authorities` table ever holds more than one row per strategy. Bootstrap: fresh installs from operator seeds, restores, and crash-recovery — the admin's authority binding, token policy, and client key become plan luck, and the wrong binding is sticky (only raw SQL repairs it). Operations: `GET /by_strategy` silently hides multiplicity, so an operator "confirming the authority" in a support session may be looking at one of several. SDK users (`oxidauth-rs`) inherit the same one-of-N blindness.
- **Severity (P2 as registered, agreed):** wrong behavior, recoverable only with DBA work; not a live break for single-row-per-strategy installs (the overwhelmingly common case so far — hence P2, not P1). Worst realistic outcome: admin parked on a TOTP-enforcing or short-TTL authority after a restore, discovered as "we can't log in the way we used to" with nothing in the error pointing at `ORDER BY`.
- **Data integrity:** no corruption by this bug alone (pure read), but it *selects which* corrupted/valid row surfaces, and it writes a persistent mis-binding to `user_authorities` at bootstrap [mechanism verified, outcome via the register flow].
- **Not affected:** single-match deployments (pick is trivially right, `None`-when-absent unchanged); all writes; the `oxidauth-import-export` tooling (no callers).

## Proposed resolution

**1. Deterministic total order in the SQL (minimum fix, zero migration).** `select_authority_by_strategy.sql`:

```sql
SELECT *
FROM authorities
WHERE strategy = $1
ORDER BY created_at ASC, id ASC
LIMIT 1
```

- `created_at ASC` = "the **first** authority of this strategy": matches the `first_or_create_authority` name and, crucially, is *restore-stable* — if an operator later adds another username_password authority and the bootstrap setting is lost, re-bootstrap re-attaches to the **original** default, not to the newest row a `DESC` pick would prefer. With `ORDER BY … DESC` the second choice would silently change admin bindings after every added authority.
- `id ASC` is the tie-breaker for the same-`NOW()` seed/restore case (the dominant tie in practice — see Problem). Postgres compares `uuid` byte-wise and `uuid::Uuid: Ord` uses the same byte order (established in OXA-000019 §Resolution 1), so the expected winner is expressible identically in Rust tests. `created_at` is `NOT NULL` (`create_authorities.sql:9`), so no `NULLS` clause is needed.
- Add a comment above `ORDER BY` stating the contract: "oldest row wins; ties break to the smallest id; multiplicity is NOT an error — callers needing all matches must use the plural query."
- No index needed: `authorities` holds a handful of rows; if it ever grows, `CREATE INDEX ON authorities (strategy, created_at, id)` then.

**2. Make multiplicity observable: plural query + bootstrap warning (the "caller cannot tell" half).** The register asks for either a count/exists companion or a `Vec` return. Converting the *existing* query to `Vec` is the wrong cut: it churns `Service::Response` (`mod.rs:8`), the `SelectAuthorityByStrategyQuery` trait (`oxidauth-repository/src/authorities/select_authority_by_strategy.rs:8-18`), the use case's `ok_or_else` mapping (`find_authority_by_strategy.rs:41`), the kernel service trait's `Result<Authority, _>`, the HTTP handler, and the `oxidauth-rs` response shape — to fix a bug for callers that want *one*. Instead, additive:

- New module `oxidauth-postgres/src/authorities/select_authorities_by_strategy/` with `select_authorities_by_strategy.sql` (`SELECT * FROM authorities WHERE strategy = $1 ORDER BY created_at ASC, id ASC`, `fetch_all`, `Response = Vec<Authority>`) plus the kernel DTO (`ListAuthoritiesByStrategy`) and repository trait, mirroring the existing `select_all_authorities` module shape. Order matches step 1 so the first element **is** the singular query's answer — an invariant worth a test.
- Bootstrap: after `first_or_create_authority` returns `Ok`, call the plural query and `warn!` (`bootstrap/mod.rs:388`, `Ok(authority) =>` arm) when `matches.len() > 1`, logging every match id/name. Pure observability — the pick is already deterministic; behavior and the `first_or_create_authority` signature stay unchanged, so the bootstrap mock tests (`bootstrap/mod.rs:1019-1033, 1231-1235`) need only a mock addition, no assertion flips.
- **No new HTTP surface.** Multiplicity exposure over the API is a contract decision (payload shape or header) with no consumer identified; `GET /authorities` + client-side filter remains the operator audit path. If a consumer materializes, add `GET /authorities/by_strategy/{strategy}/all` (returning `{authorities: [...]}`) then — deliberately out of this ticket's scope.

**3. Rejected alternatives, recorded for the record:**
- *Return `Vec` from the existing query:* blast radius above; rejected.
- *`count`/`EXISTS` companion instead of plural:* strictly weaker (can't warn with names/ids, can't list), and the plural query subsumes both (`len()` is the count).
- *Name-based lookup as proposed by the `TODO(dewey4iv)` at `bootstrap/mod.rs:380-381`:* the right long-term identity query for bootstrap (`name` is UNIQUE — `create_authorities.sql:3`), but it is a new feature (new kernel DTO + service + bootstrap rewiring + create-side naming policy) and does not excuse leaving today's query nondeterministic for its other caller (`GET /by_strategy`). If someone tickets it, this ticket's step 1 stays as defense for that endpoint; bootstrap's `first_or_create_authority` would then stop calling by-strategy altogether. Cross-reference rather than bundle.
- *Partial unique index "one enabled username_password authority":* encodes an arbitrary policy 20240523190325 deliberately removed; rejected.

**4. Pin flips (the only marker is `select_authority_by_strategy/mod.rs:70-72`):**
- `it_should_return_exactly_one_of_multiple_matches` (`:56-80`): rename to `it_should_deterministically_return_the_oldest_match`; delete the `BUG(pinned)` block (`:70-72`); replace the disjunction (`:73-77`) with an exact assertion. Because `#[sqlx::test]` shares one transaction (all fixture INSERTs get the same `NOW()`), keep the existing `seed_authority` seeds but additionally raw-seed one row with an *explicitly older* `created_at` (pattern: `private_keys/select_most_recent_private_key/mod.rs:50-63` and `test_fixtures.rs:133-144`) named e.g. `oauth_zero`, and assert `found.name == "oauth_zero"` — proving `created_at ASC` drives the pick. Keep `:78-79` (`strategy`/`status` mapping) unchanged.
- Add `it_should_break_same_created_at_ties_by_smallest_id`: raw-seed two rows with an identical explicit `created_at` (the OXA-000019 `:109-111` pattern) and assert `found.id == std::cmp::min(id_a, id_b)` plus `assert_ne!` against the max — pins the tie-breaker itself.
- Add `it_should_match_the_first_row_of_the_plural_query` (step 2): `select_authorities_by_strategy` returns ≥2 rows, strictly ordered per the contract, and `singular.id == plural.first().unwrap().id`.
- Leave green, untouched: `it_should_map_strategy_enum_and_return_none_when_absent` (`:83-112`) — mapping and `None`-when-absent invariants survive the fix unchanged; bootstrap's mock tests (mocks never run SQL); `hurl/tests/authorities.hurl:25-39` (single-match environment — both asserts unchanged).
- Register/docs hygiene once green: drop `BUGS_AND_NOTES.md:41`; update `missing-tests.md:61` ("done" → fixed), and note `:337`'s stale line cite (`mod.rs:90` → the flipped test) alongside `:326`.

**5. Compat concerns.** No schema change, no migration, no DTO/wire change for any existing caller: `Option<Authority>` stays, so the HTTP 200 shape and the `oxidauth-rs` contract are byte-identical. The only observable delta is *which* row wins on previously-ambiguous data: from plan luck to oldest-then-smallest-id. For every single-match deployment (all in-tree tests, hurl, and the common install) the result is unchanged — `ORDER BY` over one row is a no-op. Existing multi-row production data becomes deterministic but is not *repaired* — step 2's bootstrap warning is the operator's detector; whether the oldest row is the *intended* authority remains an operator decision, like OXA-000019 §3.

## Verification

- **Targeted proof:** `cargo test -p oxidauth-postgres authorities::select_authority_by_strategy` (DB env per `./src/oxidauth/database_test.sh`): flipped oldest-wins test, new smallest-id tie test, and plural-consistency test green; the none/mapping test still green.
- **Regression soundness:** the oldest-wins and smallest-id assertions pin a rule today's SQL does not implement; on the pre-fix SQL they pass only by the accident of a small-table seq scan (insertion order) — note this honestly as OXA-000019 §Verification does. The genuine red-before proof is the tie test run with `SET enable_seqscan = off` / a scratch `CREATE INDEX ON authorities (strategy)` on a test DB: the old SQL flips its answer (test can't exist yet, so reproduce once manually via psql against the fixed SQL and confirm the index no longer changes the returned id).
- **Live determinism/replica check:** against a seeded test stack with two same-`NOW()` username_password rows: `psql -c "INSERT ...; INSERT ..."` in one `BEGIN/COMMIT`, then run the fixed SQL twice and after `VACUUM authorities` — identical id (`min(id)` of the tie); `EXPLAIN` shows the `ORDER BY` in the plan.
- **Bootstrap observability:** `cargo test -p oxidauth-services bootstrap` green (mock addition only); a throwaway integration check seeding two up-authorities and running bootstrap asserts the `warn!` fires with both ids (e.g. capture via `tracing` test subscriber, or manual run — behavior itself must be unchanged).
- **Consumer non-churn:** `cargo test -p oxidauth-services authorities::find_authority_by_strategy` (mock-based `None`→`AuthorityNotFoundError` mapping untouched) and `cargo test -p oxidauth-rs authorities` (route contract `find_authority_by_strategy_route_contract` unchanged).
- **hurl:** `hurl --test src/oxidauth/hurl/tests/authorities.hurl` against a running server — byte-identical results (single-match seed).
- **Marker removal:** `grep -rn "BUG(pinned)" src/oxidauth/oxidauth-postgres/src/authorities/select_authority_by_strategy/` returns nothing; `BUGS_AND_NOTES.md:41` deleted.
