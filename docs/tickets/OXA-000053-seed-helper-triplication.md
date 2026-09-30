# OXA-000053 — Grant-pair seed copies remain across the three grant test mods (`test_fixtures` has no `seed_user_role_grant`)

**Original ID:** N-3 · **Severity:** n/a · **Type:** note · **Status:** scheduled
**Review tier:** Tier 3 (small precise recipe, multi-file) — ranked #27 of 27 · reviewed 2026-09-30 · SCHEDULED (accepted as proposed, split per advisory). **(a) This ticket's scope:** add `seed_user_role_grant` export to `test_fixtures.rs` beside its three sibling edge seeders (grep `fn seed_.*_grant` — exact line cites drift under the in-flight refactor; locate by name, T-6 conventions verbatim) + swap the two non-colliding copies — in `user_role_grants/delete_user_role_grant/mod.rs`'s `seed_grant` and `users/delete_user_by_id_query/mod.rs`'s grant seed. Test-only; `#[allow(dead_code)]` makes merge order safe. **(b) Retirement claim NARROWED:** N-3 retires when "every grant `delete_*` mod composes a shared seeder" holds (zero raw pair-INSERTs in grant `delete_*`/`users` test mods + module-doc line "edge composition wrappers stay local; edge *inserts* never do" + register line retired). NOT gated on copies #2/#3 (`select_user_role_grants_by_user_id/mod.rs:94`/`:100` — owned by OXA-000018, DEFERRED; ownership pointer in its Decision section) nor #5/#6 (`auth/tree` raw inserts — owned by OXA-000001's rewrite; tree shapes stay inline). Composition wrappers (`seed_grant`×3 + `seed_edge`) and `auth/tree` locality stay — S14-blessed; cosmetic `seed_edge` rename skipped. **HOLD (owner 2026-09-30):** no edits applied to the tree — parked until `cargo check -p oxidauth-postgres --lib` is green (464 lib errors at hold time from the in-flight named-trait/prelude migration; DB suite is this ticket's only real gate). Verification then: `cargo test -p oxidauth-postgres` green, S14 150-test census unchanged, `BUG(pinned)` marker set identical before/after; requires Postgres (:5434, `.env` sourced, `database_test.sh` convention).


## Locations

All paths relative to `src/oxidauth/oxidauth-postgres/src/`. Verified against the working tree 2026-09-29.

The shared module (audit A11.3 / S14 fold):

- `test_fixtures.rs` — `#[cfg(test)] pub(crate) mod` declared at `lib.rs:39`; module doc `:1-11` records the isolation decision (`sqlx::test` gives each test its own freshly-migrated database — no transaction (docs/TESTING.md §2; the former transaction-rollback wording here was folklore, corrected in place by OXA-000066); seeding stays per-test, only the single-INSERT bodies were folded). Exports (`#![allow(dead_code)]` at `:13`): `default_authority_settings`, `create_authority`, `create_user`, `assert_sql_state` (`:91`), `seed_user`, `seed_user_named`, `seed_authority`, `seed_authority_with_settings`, `seed_role` (`:165`), `seed_permission`, `seed_public_key`, `seed_refresh_token`, `seed_refresh_token_now`, and **three** of the four grant-edge seeders: `seed_role_permission_grant` (`:246`), `seed_role_role_grant` (`:255`), `seed_user_permission_grant` (`:264`). **There is no `seed_user_role_grant`.**

The remaining duplication in the three register-named directories (all under `*/delete_*/mod.rs` unless noted):

| Copy | Site | Builds on shared export? |
|---|---|---|
| `seed_grant` (user, role) | `user_role_grants/delete_user_role_grant/mod.rs:39-49` | **No** — inlines `INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)` at `:42` (the export doesn't exist) |
| `seed_edge` (parent, child) | `role_role_grants/delete_role_role_grant/mod.rs:36-41` | Yes — `seed_role_role_grant` |
| `seed_grant` (role, permission) | `role_permission_grants/delete_role_permission_grant/mod.rs:39-44` | Yes — `seed_role_permission_grant` |
| `seed_grant` (user, permission) — the unsanctioned fourth sibling, outside the register line | `user_permission_grants/delete_user_permission_grant/mod.rs:46-50` | Yes — `seed_user_permission_grant` |

Raw copies of the missing seeder's SQL, `INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)` — **6 copies across 4 modules**:

1. `user_role_grants/delete_user_role_grant/mod.rs:42` (inside `seed_grant`)
2. `user_role_grants/select_user_role_grants_by_user_id/mod.rs:94` (inside the OXA-000018 pin test)
3. `user_role_grants/select_user_role_grants_by_user_id/mod.rs:100` (same test, foreign pair)
4. `users/delete_user_by_id_query/mod.rs:61` (inside the OXA-000020 cascade pin test)
5. `auth/tree/mod.rs:166` (`seed_full_fixture`, via local `exec`)
6. `auth/tree/mod.rs:353` (`it_should_recurse_forever_on_a_role_role_grant_cycle`, via `exec`)

For contrast, the role-side tables' pair SQL has **no** raw copies left outside `auth/tree` — every non-tree consumer already composes the shared seeders (e.g. `role_role_grants/select_*/mod.rs:75-77`, `role_permission_grants/select_*/mod.rs:76-78`, `roles/delete_role/mod.rs:75,99`, `permissions/delete_permission/mod.rs:91`, `user_permission_grants/select_*/mod.rs:77-79`). `auth/tree/mod.rs` keeps raw copies of all four edge tables (`:166-218`, `:353-377`) plus a cycle test that raw-inserts `role_role_grants` twice (`:360-370`).

Pinned tests nearby (`grep BUG(pinned)` in the three directories): `user_role_grants/select_user_role_grants_by_user_id/mod.rs:131` (OXA-000018 timestamp mapping), `role_role_grants/insert_role_role_grant/mod.rs:75` and `:96` (OXA-000001 self-grant / two-role cycle). The N-3 lines themselves are not test-pinned.

## Problem

This is a record, not a defect. The register's wording — "`seed_role`, user/role grant seeds, pair rows **are** triplicated" — describes the pre-S14 state and is already stale: the S14 fold (missing-tests.md Review Log "S14 pg fixtures (A11.3) — round 1 — verdict: PASS") moved the entity seeders into `test_fixtures`, and all three directories now import `seed_role`/`seed_user`/`seed_permission` from it (e.g. `user_role_grants/insert_user_role_grant/mod.rs:36`, `role_role_grants/insert_role_role_grant/mod.rs:34`, `role_permission_grants/insert_role_permission_grant/mod.rs:38`). No local `seed_role`/`seed_permission` copies remain anywhere (the four local `seed_user` fns under `users/` are the deliberately-distinct write-path/kind×matrix variants S14 audited, not copies).

What S14 **deliberately did not fold** — the residual N-3 actually refers to — is the pair-row composition layer: one local `(Uuid, Uuid)` wrapper per `delete_*` test mod, three of the same shape in the register-named directories (plus a fourth in `user_permission_grants`, not in the register line), and a hole in the shared API: three grant-edge seeders exist but `seed_user_role_grant` does not, so the `user_role_grants` copies are the only ones that cannot compose and instead hand-copy the INSERT text — six times across four modules.

## Analysis

**Verified current-state answers to "fix or keep":** the note is half-actionable. Two distinct decisions are tangled in it:

1. *The composition wrappers* (`seed_grant` ×3 + `seed_edge`) — **intentionally keep.** S14's PASS verdict explicitly blessed them ("composition wrappers built on shared exports (`seed_grant`×3, `seed_edge`) … deliberately distinct from the 18 shared exports", missing-tests.md:610). They are 5-10 lines, sit next to their only two call sites, and adding four shared two-row `seed_user_role_pair`-style helpers would move logic *away* from the tests that read them.
2. *The missing `seed_user_role_grant` export and the six raw SQL copies it forces* — **actionable, small, and drift is already visible:**
   - **Construction drift:** of the four pair-row wrappers, three compose a shared seeder and one (`user_role_grants/delete`) inlines SQL — not by design, but because the export is absent. The shared module's own API (`:246-271`) is the argument against the gap: three identical-shape seeders exist; the fourth is the one table the auth tree and every user delete touch.
   - **Naming drift:** the identical composition shape is `seed_grant` in two directories and `seed_edge` in `role_role_grants/delete` — no semantic difference (`seed_role_role_grant`'s own panic message even says `"seed edge"` while its siblings say `"seed grant"`).
   - **Panic-message drift over byte-identical SQL:** the same INSERT carries `"seed grant"` (`delete_user_role_grant/mod.rs:47`, `select …/mod.rs:99`), `"seed foreign grant"` (`:105`), `"seed user_role_grant"` (`users/delete_user_by_id_query/mod.rs:65`). Any future column rename (e.g. `role_id` on `user_role_grants`) is a 6-site edit with no compiler help — exactly the failure mode the S14 fold existed to remove.
   - Precedent that follow-through happens: S14's one nit (a verbatim-duplicated `assert_unique_violation` in `permissions/insert_permission`) is already gone from the tree (grep: zero hits), so the shared-helper convention is actively enforced after the fact.

**T-6 conventions the extraction must follow** (codified in docs/TESTING.md §3 — the old `BUGS_AND_NOTES.md:101` register pointer retired with the register, 2026-09-30): tests run under `#[sqlx::test(migrator = "crate::MIGRATOR")]` (one isolated database per test, docs/TESTING.md §2; `MIGRATOR` at `lib.rs:45`), repositories are constructed with `Database::from_pool(pool.clone())` — the pool is taken by value, and every write commits for real in the test's own database — and `test_fixtures` helpers take `pool: &PgPool`, bind explicit columns, and `.expect(...)` on execute — all visible at `test_fixtures.rs:246-271` and used at every leaf test site (e.g. `role_role_grants/delete_role_role_grant/mod.rs:47`).

**Sequencing with pending tickets that own the same test bodies:**

- **OXA-000018** (grant-timestamp `From<PgUserRole>` bug): its fix flips the pin body `select_user_role_grants_by_user_id/mod.rs:69-137`, which *contains two of the six raw copies* (`:94`, `:100`). Collision is direct (same line ranges).
- **OXA-000001** (cycle guard): its fix rewrites `auth/tree`'s `it_should_recurse_forever_on_a_role_role_grant_cycle` (`:344-424`, raw copies `:353/:360/:367/:374`) and flips `role_role_grants/insert` pins `:75`/`:96`. Any effort to route `auth/tree` raw INSERTs through shared seeders collides there.
- **OXA-000020** (CASCADE/non-idempotent deletes): owns the test holding copy #4 (`users/delete_user_by_id_query/mod.rs:61`) but its fix is schema/executor-side; a drop-in `seed_user_role_grant(pool, user_id, role_id)` swap at that site is compatible in either order.

Therefore: land the non-colliding half of the extraction **now-ish** (new export + copies #1 and #4), and fold copies #2/#3 into OXA-000018's rewrite and copies #5/#6 into OXA-000001's tree rewrite rather than racing them. `auth/tree` locality itself stays (S14: "auth-tree/cycle shapes … intentionally stay inline", `test_fixtures.rs:9-11`).

## Impact

Test-only. `test_fixtures` is `#[cfg(test)] pub(crate)` (`lib.rs:39`) with `#![allow(dead_code)]` (`:13`), so a new export can merge before its call sites swap, and unused exports can't fail CI. Zero product surface: the S14 audit verified the fold produced zero non-test diffs, zero `migrations/` and zero `.sql` changes, and the same holds for this follow-up — the repository insert paths (`*/insert_*.sql`, which are the product's pair-row writers) are untouched. No user-visible behavior; the risk is entirely "a green test suite goes red via a botched mechanical swap," caught by the suite itself.

## Proposed resolution

**Recommendation: FIX LATER (small, sequenced).** Keep the composition wrappers and `auth/tree` locality (retire that half of N-3 as intended behavior); close the `seed_user_role_grant` gap and retire the register line once copies are gone.

1. **Add the missing export** to `test_fixtures.rs` beside its siblings (after `:271`, keeping the four edge seeders grouped), mirroring the T-6 conventions verbatim:

   ```rust
   /// Insert a `user_role_grants` edge row.
   pub(crate) async fn seed_user_role_grant(pool: &PgPool, user: Uuid, role: Uuid) {
       sqlx::query("INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)")
           .bind(user)
           .bind(role)
           .execute(pool)
           .await
           .expect("seed grant");
   }
   ```

2. **Swap the two non-colliding copies immediately:** `user_role_grants/delete_user_role_grant/mod.rs:42-47` (inside its `seed_grant`) and `users/delete_user_by_id_query/mod.rs:61-66`. The swap is drop-in: same columns, same pool-by-ref convention, only the panic string changes.

3. **Copies #2/#3 ride OXA-000018's commit:** when that ticket rewrites the `:69-137` pin fixture, express the two grant pairs as `seed_user_role_grant(...)` calls and keep the backdated-role `INSERT INTO roles (… '2000-01-01' …)` (`:74-84`) inline — it is test-specific semantics of exactly the kind `test_fixtures.rs:9-11` says stays local.

4. **Copies #5/#6 ride OXA-000001's commit** (or a post-fix follow-up): the tree fixture's *shapes* stay inline, but the individual edge inserts can call the four shared seeders (`seed_user_role_grant`, `seed_role_role_grant`, `seed_role_permission_grant`, `seed_user_permission_grant`) instead of raw `exec(...)` text — mechanical, semantics-preserving. Do not pre-empt the rewrite.

5. **Skip the cosmetic rename** (`seed_edge` → `seed_grant`, panic-string unification) unless it lands inside one of the above commits; it has no independent value and churns pinned-test vicinity.

6. **Retire the note:** update the `test_fixtures.rs:9-11` module doc to name the four edge seeders as the shared layer ("edge composition wrappers stay local; edge *inserts* never do"), and mark N-3 done in BUGS_AND_NOTES.md §5 once grep shows zero `INSERT INTO *_grants (` inside any `mod tests` outside `auth/tree`/composition wrappers.

## Verification

No product code changes, so proof is the suite staying green plus grep invariants (postgres required: docker at host port 5434 with `.env` sourced — the S14 run convention).

- `cargo test -p oxidauth-postgres` → all green. S14 recorded **150 passed; 0 failed** with per-entity census `role_role_grants` 8, `user_role_grants` 7, `role_permission_grants` 7, `user_permission_grants` 7 — the pass count must not change (the extraction adds and removes no test fns). *(Counts are from the S14 log; not re-run for this ticket.)*
- Targeted: `cargo test -p oxidauth-postgres --lib -- user_role_grants:: role_role_grants:: role_permission_grants:: user_permission_grants::` plus `-- users::delete_user_by_id_query::` and `-- auth::tree::` for steps 2 and 4.
- Pins that must stay green until their owning ticket flips them: `user_role_grants/select_user_role_grants_by_user_id` (OXA-000018), `role_role_grants::insert_role_role_grant::it_should_allow_a_self_referencing_grant_parent_eq_child` / `it_should_allow_a_two_role_cycle` and `auth::tree::it_should_recurse_forever_on_a_role_role_grant_cycle` (OXA-000001), `users::delete_user_by_id_query` cascade/non-idempotence pins (OXA-000020). `grep -rn 'BUG(pinned)' src --include='*.rs'` (from repo root) must show the same marker set before and after the mechanical steps.
- End-state grep invariants: zero `INSERT INTO user_role_grants` text in `user_role_grants/`, `users/` test mods; zero raw pair-INSERT text in any grant `delete_*` mod (wrappers must compose seeders).
