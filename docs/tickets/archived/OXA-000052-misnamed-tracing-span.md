# OXA-000052 — `select_user_by_username_query` instruments under the span name `"select_user_by_id_query"`: every username lookup impersonates a different, live query in traces

**Original ID:** N-2 · **Severity:** n/a · **Type:** note · **Status:** done
**Review tier:** Tier 2 (mechanical code fix) — reviewed early, pulled forward · SCHEDULED 2026-09-29


## Locations

Register (`BUGS_AND_NOTES.md:84`) cites `oxidauth-postgres/src/users/select_user_by_username_query/mod.rs:11` — **exact hit**, no marker drift. Crate lives at `src/oxidauth/oxidauth-postgres/`.

- **Defect:** `src/oxidauth/oxidauth-postgres/src/users/select_user_by_username_query/mod.rs:11` — `#[tracing::instrument(name = "select_user_by_id_query", skip(self))]` on `Service<&Username>::call` (`:12`), which runs `select_user_by_username_query.sql` (`WHERE username = $1`).
- **The name it steals:** `src/oxidauth/oxidauth-postgres/src/users/select_user_by_username_query/../select_user_by_id_query/mod.rs:12` carries the same literal `name = "select_user_by_id_query"` — the *correct* owner of that span name still exists as a sibling module (`users/mod.rs:11-12` declares both). So this is not a stale leftover of a deleted module; two live, semantically different queries emit one span name.
- **Not test-pinned:** no `BUG(pinned)` marker exists anywhere in `select_user_by_username_query/`. Its tests (`mod.rs:60-83`) pin behavior only — found-user fields (`:70-72`) and missing-row → `Ok(None)` (`:76-84`, the refutation recorded at `missing-tests.md:336`). Repo-wide grep for the string literals `"select_user_by_id_query"`, `"select_public_key_by_id_query"`, `"delete_refresh_token_query_by_id"`, `"permission_tree_query"` in `src/` returns only the four `#[tracing::instrument(name = …)]` attributes themselves — no test, log parser, fixture, `.hurl` file, or config matches a span-name string.
- **Prior acknowledgment:** `missing-tests.md:338` already recorded this as "P3, pre-existing (HEAD, NOT introduced by this patch) … worth a one-word fix in a future slice."
- **Log pipeline:** `src/xlib/telemetry/src/lib.rs:27` (`EnvFilter::try_from_default_env()`), `:39-40` (`BunyanFormattingLayer`); the shipped default filter is level-based only (`.env:3`: `RUST_LOG=INFO`).

## Problem

The tracing span around the by-username user lookup is named `"select_user_by_id_query"` — the name of a different, still-present query one directory over. A copy-paste of the instrument attribute (or of the whole module) that never had the `name` edited, predating the test-coverage pass. Every execution of the username lookup logs a span that claims to be the by-id lookup.

## Analysis

**The register line is accurate and the fix is exactly one word.** The attribute is a verbatim clone of the sibling's (`select_user_by_id_query/mod.rs:12`): same name string, same `skip(self)`. The only difference between the two modules' attributes that should exist is the name token.

**Sweep results — same copy-paste class across `oxidauth-postgres`.** All 60 `#[tracing::instrument(name = …)]` sites in the crate pair a module directory with a span name; 56 follow the module's `<name>_query` convention exactly. The 4 deviations split into three different dispositions:

1. `users/select_user_by_username_query/mod.rs:11` → `"select_user_by_id_query"` — **this ticket.** The only deviation that collides with another module's live span name. Fix here.
2. `refresh_tokens/delete_refresh_token_by_id/mod.rs:10` → `"delete_refresh_token_query_by_id"` — word-order typo (`query` infix misplaced; compare correctly-named sibling `delete_refresh_token_by_user_id/mod.rs:10` → `delete_refresh_token_by_user_id_query`). No collision (no module emits the correct spelling), so it is cosmetic convention drift, but it is the same paste-then-fumble class and belongs in the same one-word sweep: `"delete_refresh_token_by_id_query"`.
3. `public_keys/select_public_key_by_user_id/mod.rs:11` → `"select_public_key_by_id_query"` — **not the same defect; do not "fix" the span.** Here the span name is *true to the SQL*: the module's `select_public_key_by_id.sql` is `WHERE id = $1` (`public_keys` has no `user_id` column at all), pinned by `BUG(pinned)` at `select_public_key_by_user_id/mod.rs:41-43`. The liar is the *module directory name*, and that is already ticketed as **OXA-000024** (register DATA-11, `BUGS_AND_NOTES.md:43`). Renaming the span to match the lying module would make it wrong; the span becomes consistent for free when OXA-000024 renames the module.
   *(Implementation-time note 2026-09-29: OXA-000024 has since landed — directory is `select_public_key_by_id/`, marker deleted there; the old path above is kept as review-time history. The span line is untouched, per this ruling.)*
4. `auth/tree/mod.rs:22` → `"permission_tree_query"` — **intentional, keep.** The module directory is `tree`; the mechanical name `"tree_query"` would be strictly worse. This is the crate's one deliberate descriptively-named span (it serves the permission-tree endpoint behind OXA-000039/OXA-000040). Worth a one-line comment so the next sweep doesn't "fix" it.

(Marginal fifth: `totp_secrets/select_where_no_totp_secret_by_authority_id` uses the bare module name with no `_query` suffix — name matches module, so no copy-paste class; leave.)

**Why the collision actually costs something, and what still saves you.** `tracing::instrument` overrides only the span *name*; two mitigations survive: the span `target` remains the defining module path (`oxidauth_postgres::users::select_user_by_username_query` vs `…::select_user_by_id_query`), and the recorded field names differ (`username: &Username` at `:12` vs `user_id: &FindUserById` at `select_user_by_id_query/mod.rs:13`). So a reader who inspects per-record fields can disambiguate. What is destroyed is anything that keys on span name alone: grouping/aggregating by span in log tooling, dashboards, and — most commonly — a human grepping Bunyan JSON for which lookup ran. `[INFERENCE — I verified the subscriber wiring (EnvFilter + BunyanFormattingLayer) but not which fields BunyanFormattingLayer serializes per span record.]` EnvFilter itself is unaffected: directives match target+level, never span name, and `.env:3` sets level only.

**Both colliding spans are hot, and they overlap on the login path.** `SelectUserByIdQuery` is consumed by `oxidauth-services/src/auth/authenticate.rs:34`, `authenticate_or_register.rs:26`, and four grant use-cases (`user_permission_grants/create_user_permission_grant.rs:11` et al.); the username lookup backs the `FindUserByUsername` use-case wired at `oxidauth-services/src/bootstrap/mod.rs:63,174,442-447` (bootstrap's first-or-register and OAuth auto-registration flows). A trace through an auto-registration login therefore contains *both* lookups under the name `select_user_by_id_query` — exactly the scenario where the mislabel burns diagnosis time.

**No contract depends on the wrong name** (see Locations: zero string matches outside the attributes). Same posture as OXA-000048 and OXA-000032 — the copy-paste label family — where the resolution is likewise "rename + repin, nothing else moves."

## Impact

Affects operators/SREs reading Bunyan JSON or trace output during login, bootstrap, or OAuth auto-registration incidents: username lookups are indistinguishable from by-id lookups by span name, and by-id lookup counts (the more common operation) absorb them silently. No security exposure, no behavior or data change, no wire/API surface, no client impact — the name is consumed by no code in-repo. Severity truly n/a as a bug; it is a one-word diagnostic-quality fix.

## Proposed resolution

**Recommendation: FIX NOW.** This is the cheapest item in the register — a one-word rename with zero callers — and `missing-tests.md:338` already pre-approved it as "a one-word fix in a future slice." Deferring only guarantees the name fossilizes into people's log queries.

1. **The fix, `oxidauth-postgres/src/users/select_user_by_username_query/mod.rs:11`:** `name = "select_user_by_id_query"` → `name = "select_user_by_username_query"`. The new name equals the module directory (the crate's dominant convention, 56/60) and the `.sql` file's name; no collision exists (`select_user_by_username_query` is emitted nowhere else — verified by the sweep and the repo-wide literal grep).
2. **Ride-along sweep, same commit:** `oxidauth-postgres/src/refresh_tokens/delete_refresh_token_by_id/mod.rs:10` → `name = "delete_refresh_token_by_id_query"` (typo fix; no collision — the by-user_id sibling already owns `delete_refresh_token_by_user_id_query`).
3. **Retire the note + precedent trail:** delete register line `BUGS_AND_NOTES.md:84`; strike or annotate the open item at `missing-tests.md:338`.
4. **Guard the deliberate exception, `oxidauth-postgres/src/auth/tree/mod.rs:22` (comment only):** `// span name intentionally descriptive: module dir is `tree`; mechanical `tree_query` would be worse` — prevents the next sweep from "correcting" it.
5. **Explicitly out of scope:** the public_keys rename (owned by OXA-000024 — renaming its span now would make the span *lie*, see Analysis item 3); introducing span-name-pinning tests (no existing test in the crate asserts span names; doing it for this one site invents a new convention — the permanent regression guard is the grep in Verification, run as part of the sweep).

**Compat:** nothing in-repo matches either old name; no signature, trait, SQL, or client change. If an external dashboard/alert groups by the span name `select_user_by_id_query` for *username*-lookup failures, it must be repointed — nothing in-repo does.

## Verification

Commands for the implementer (this ticket was written from static reading — per assignment, no builds or tests were executed here).

- **Grep proof (the permanent check):** after the change, `grep -rn '"select_user_by_id_query"' src` returns exactly one hit (`select_user_by_id_query/mod.rs:12`), and `grep -rn '"select_user_by_username_query"' src/` returns exactly one (the renamed attribute). Likewise `"delete_refresh_token_by_id_query"` lands exactly once and `"delete_refresh_token_query_by_id"` disappears.
- **Compile check:** `cargo check -p oxidauth-postgres` — attribute-name-only edits; no behavior surface to compile beyond that. (Full-suite run belongs to the batched main-agent pass, not this change.)
- **Existing tests stay green untouched:** `cargo test -p oxidauth-postgres select_user_by_username_query` — the two tests (`mod.rs:60-83`) pin query behavior, not the span name; neither references it.
- **Visual confirmation, needs Postgres + running API `[recipe not executed here]`:** `RUST_LOG=oxidauth_postgres=debug cargo run -p oxidauth-api` (Bunyan JSON per `src/xlib/telemetry/src/lib.rs:39-40`), perform a username login, and confirm the emitted spans now read `select_user_by_username_query` (field `username`) and, for a by-id path, `select_user_by_id_query` (field `user_id`) — one name per query, fields match names.

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- **Review tier: Tier 2 (mechanical code fix), reviewed early (pulled forward ahead of remaining Tier 2 items)** — scout labeled MODERATE on file count; edits themselves are one-word attribute renames.
- All five steps approved, including the per-site verdict split:
  1. `select_user_by_username_query/mod.rs:11` span name → `"select_user_by_username_query"` (1 word; zero in-repo consumers).
  2. Ride-along: `delete_refresh_token_by_id/mod.rs:10` → `"delete_refresh_token_by_id_query"` (word-order typo, same commit).
  3. Strike `missing-tests.md:338` + delete register line `BUGS_AND_NOTES.md:84`.
  4. Guard comment at `auth/tree/mod.rs:22` — `permission_tree_query` stays, intentional descriptive name.
  5. `select_public_key_by_user_id` span NOT touched (span is true to SQL; module rename owned by OXA-000024 — this ticket must not "fix" it into lying).
- No span-name-pinning tests (rejected — would invent a convention); permanent guard = the Verification grep: each correct name exactly once, typo string gone, `"select_user_by_id_query"` exactly one hit.
- Compat noted: any external dashboard grouping by the old misnamed span must be repointed; none known in-repo.
- Acceptance: `cargo check -p oxidauth-postgres` + existing two module tests untouched-green; visual Bunyan check is an optional recipe.
