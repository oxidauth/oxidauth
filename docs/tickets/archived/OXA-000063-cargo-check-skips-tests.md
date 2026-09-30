# OXA-000063 — `cargo check` is blind to `#[cfg(test)]` code; codify the compile gate

**Original ID:** T-3 · **Severity:** n/a · **Type:** tooling · **Status:** done
**Review tier:** Tier 1 (docs-only), ranked item 2/9 · IMPLEMENTED 2026-09-29 (CI-compile-lane follow-up clause lives in this ticket only)


## Locations

- `BUGS_AND_NOTES.md:98` — register line T-3: "`cargo check` does NOT compile `#[cfg(test)]` code — always gate with `cargo test --no-run` / a real test run."
- `bin/unit_test.sh:6` — `cargo test --workspace --exclude *-postgres --exclude postgres` (compliant; the exclusion is per its own header comment, `:4-5`, and per OXA-000061)
- `src/oxidauth/database_test.sh:18` — `cargo test -p oxidauth-postgres -p postgres -- --nocapture`, dispatched by `bin/database_test.sh:18-29` (compliant)
- `README.md:137` — lint gate `cargo clippy --workspace --all-targets` (compliant — see Analysis); `README.md:121` — the `## Development` section, the natural home for this convention; `README.md:146-159` — the scripts table; `README.md:75-81` — quickstart already sends people to `bin/unit_test.sh`
- `src/oxidauth/` + `src/xlib/` + `src/seedz` — 166 `.rs` files contain `#[cfg(test)]` (all inline `mod tests`), i.e. the surface plain `cargo check` never compiles
- `src/xlib/postgres/src/lib.rs:302` — `#[cfg(test)] mod db_tests` (the T-1 break lives entirely inside this blind spot; `sqlx::migrate!("./test-migrations")` at `:323`)
- `docs/tickets/archived/OXA-000061-stage-test-migrations-gitkeep.md` — depends on this exact fact three times (Analysis "Who would notice", Impact "Silent until noticed", Verification item 3: "`cargo check` is NOT proof — register T-3")
- `.github/` — contains only `PULL_REQUEST_TEMPLATE.md`; no `workflows/` (independently re-verified; same finding as OXA-000061/OXA-000062 and `docs/migration-plan/15-docs-refresh.md:154-161`)
- Historical prose, NOT forward guidance: `docs/migration-plan/02:73`, `03:58`, `05:76`, `07:94`, `09:69`, `13:376`, `14:181` and `changelogs/90000013-seedz-dev-seeding.md:59` record "`cargo check …` green" as per-plan gates

## Problem

T-3 states a toolchain fact and a working rule. Verified state: the fact is true, every executable gate in the repo already obeys the rule, and no script or README line gives wrong guidance — so this item is not a defect to fix but a convention to codify before it drifts.

The failure mode it guards against is already visible in the repo's own history: seven migration-plan completion logs and one changelog record plain `cargo check` invocations as their "green" signal. Plan 05's gate (`docs/migration-plan/05-postgres-database-macro.md:76`, "`cargo check -p oxidauth-postgres` green") is precisely the command that would sail through the T-1 `.gitkeep` break — the `compile_error!` fires only when the test profile builds `mod db_tests`.

## Analysis

**The fact — standard toolchain knowledge, marked as such** (documented cargo behavior; nothing repo-specific or vendored to cite): plain `cargo check` checks lib/bin targets without `--test`, so `#[cfg(test)] mod tests` is never type-checked; doctests are never checked either. `cargo check --tests` / `--all-targets` includes the test targets (cargo passes `--test` per target, which enables `cfg(test)`); `cargo test --no-run` compiles *and links* test binaries without executing them; a real test run does all of it.

- Doctest angle is moot here: every ```` ```rust ```` fence in the workspace is ```` ```rust,ignore ```` — 4 occurrences, all in `src/xlib/postgres/src/lib.rs:19,30,43,52`, zero non-`ignore` fences repo-wide. No doctest ever compiles, under any command.
- The blind spot is wider than type errors: compile-time macros inside test modules run only in the test profile. T-1's `sqlx::migrate!("./test-migrations")` (`src/xlib/postgres/src/lib.rs:323`, inside the `#[cfg(test)]` mod at `:302`) emits `compile_error!` on a checkout missing the keep-file — invisible to plain `cargo check`.

**Executable guidance is already correct — verified, no wrong guidance found:**

- `grep` for `cargo check` across all `.md`/`.sh`/`.yml`/`.yaml` in the repo: **zero hits in `bin/`, `src/`, `README.md`, `missing-tests.md`**. The only hits are historical plan/changelog prose and the register/tickets themselves. No automation invokes it.
- `bin/unit_test.sh:6` and `src/oxidauth/database_test.sh:18` are real `cargo test` runs. Caveat inherited from their design, not from T-3: `unit_test.sh` excludes `postgres`/`*-postgres` entirely, so even it is not a full compile gate for the DB crates — the T-1 ticket documents exactly this gap closing only under `cargo test … -p postgres` (or a fresh clone of the broken tree).
- `README.md:137` `cargo clippy --workspace --all-targets` is honest for test code too: `--all-targets` makes clippy check the test targets, which compiles them with `cfg(test)` active (standard cargo/clippy semantics, marked as toolchain knowledge). The repo's own lint guidance therefore already catches broken `#[cfg(test)]` code — but only for whoever runs clippy, and nothing explains *why* the flag matters.

**The wording of T-3 is right but conservative.** "Always gate with `cargo test --no-run` / a real test run" is the safe rule; the cheaper correct middle for a *compile-only* signal is `cargo check --all-targets` (type-checks `#[cfg(test)]`, no link, no run). It would have caught the T-1 break. The distinction worth codifying is: behavior claims require a real run; "the test code compiles" is satisfied by `cargo test --no-run` or `cargo check --all-targets`; plain `cargo check` satisfies neither claim about test code and must never be quoted as proof.

**No CI exists** (re-verified; three prior tickets agree), so nothing automated is wrong today — the entire exposure is human habit. That makes this cheap to fix now and expensive to retrofit: whoever writes the first CI workflow will copy the pattern recorded in seven plan logs ("cargo check green") unless the convention is visible in the README, the file plan 15 established as the dev entry point.

**`BUG(pinned)` audit:** no `BUG(pinned)` markers and no existing OXA ticket guard this area; OXA-000061 is a *consumer* of the fact, not a duplicate — it cites T-3 as evidence, so retiring T-3 without codifying the rule elsewhere would leave 000061 pointing at a deleted register line.

## Impact

- If retired silently: the knowledge lives only in `BUGS_AND_NOTES.md:98` and one ticket; the plan-log habit ("cargo check green") resurfaces in the first CI workflow and rubber-stamps broken test builds exactly like T-1's — `bin/unit_test.sh` and `cargo check` both report green on a clone where `cargo test -p postgres` cannot compile.
- If kept as-is (uncodified): works until it doesn't — same exposure, just delayed; nothing onboards a new contributor to the rule (166 test-bearing files, per OXA-000062's fmt analysis of where repo conventions actually live, README is what people read).
- Zero product/runtime/data/security impact either way — this is purely a development-process latency trap: test-code compile breaks surface late (first DB-suite run on a fresh clone) and misattribute ("CI says green / README says run `bin/unit_test.sh`").

## Proposed resolution

**Disposition: KEEP as a convention** — the register line is true and every executable gate obeys it, so the resolution *is* the doc change that codifies (and thereby retires) the item:

1. Add the rule to `README.md` `## Development` (a one-liner under the lint block, ~`:137`, or a table footnote at `:146`): plain `cargo check` does not compile `#[cfg(test)]` code — never quote it as a green signal for test-bearing crates; compile-only proof is `cargo test --no-run` (workspace-wide, one command) or `cargo check --all-targets`; the real gates are `bin/unit_test.sh` + `bin/database_test.sh` (which together cover what `unit_test.sh` alone excludes). Keep it to a sentence; this is a footnote rule, not a section.
2. Mirror the one line into `missing-tests.md:7` "Conventions (what 'a test' means here)", where the run commands already live.
3. Update `BUGS_AND_NOTES.md:98` to point at the README (or collapse to a pointer), preserving the `--all-targets` refinement so the cheaper honest gate stays documented. Do not rewrite the historical "cargo check green" entries in `docs/migration-plan/` or `changelogs/` — they are dated records of what those plans actually ran.
4. CI implication for the future workflow (none exists): the compile lane must be `cargo test --no-run --workspace` or a `cargo check --all-targets` lane — never bare `cargo check --workspace`; and it must not inherit `unit_test.sh`'s crate exclusions, since `postgres` is exactly the crate whose test build is macro-fragile (T-1).

Rejected: new `docs/testing.md` — none exists, plan-15 (`docs/migration-plan/15-docs-refresh.md:65`) deliberately settled the README as the dev entry point, and a one-line rule doesn't justify a new doc surface; `bin/check.sh` wrapper — the two existing scripts already run honest gates, and the README sentence covers the manual case an alias would.

## Verification

Read-only checks performed while writing this ticket (no builds/tests/fmt run, per scope):

- `BUGS_AND_NOTES.md:98` read in context of the §6 tooling block — register text matches the assignment quote verbatim.
- `grep "cargo check"` over all `*.md`/`*.sh`/`*.yml`/`*.yaml` in the repo → hits only in `docs/tickets/`, `docs/migration-plan/` (historical logs), `changelogs/90000013`, `BUGS_AND_NOTES.md`; **none** in `bin/`, `src/`, `README.md`, `missing-tests.md`.
- `grep -rn "cargo " bin/` enumerated: only `cargo test` (`unit_test.sh:6`), `cargo publish`/`metadata`/`set-version`, and `cargo build` nowhere — no script gates on `cargo check`.
- `grep -rl "#\[cfg(test)\]" src --include="*.rs" | wc -l` → 166.
- `grep '```rust'` over `src` → 4 fences, all `rust,ignore`, all in `src/xlib/postgres/src/lib.rs:19,30,43,52`; zero executable doctests.
- `ls .github/` → `PULL_REQUEST_TEMPLATE.md` only; no `workflows/` — consistent with OXA-000061 and OXA-000062.
- The cargo semantics (default target selection, `--tests`/`--all-targets` enabling `cfg(test)`, doctests excluded from check) are marked **[standard toolchain knowledge]** — no repo artifact could prove them; a maintainer may confirm against the cargo book's `cargo check` page.

After the doc lands: a contributor following only `README.md` learns the gate without opening the register, and `grep -n "cfg(test)" README.md` shows the codified line; the register line then carries the README pointer, and OXA-000061's three T-3 references keep resolving (register line survives as a pointer, not a deletion).

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- **Review tier: Tier 1 (docs-only), item 2/9.**
- Proposal approved verbatim, all four steps:
  1. `README.md` ## Development: one-liner under the lint block — plain `cargo check` never a green signal for test-bearing crates; compile-only proof = `cargo test --no-run` or `cargo check --all-targets`; real gates = `bin/unit_test.sh` + `bin/database_test.sh` (together they cover the crates `unit_test.sh` alone excludes).
  2. Mirror the line into `missing-tests.md` Conventions (~:7).
  3. `BUGS_AND_NOTES.md:98` → pointer to README, PRESERVING the `--all-targets` refinement (so OXA-000061's three T-3 references keep resolving). Historical "cargo check green" entries in `docs/migration-plan/` and `changelogs/` NOT rewritten (dated records).
  4. CI implication recorded for the future workflow: compile lane = `cargo test --no-run --workspace` or `check --all-targets`; never bare `cargo check --workspace`; must not inherit `unit_test.sh`'s postgres exclusions.
- `--all-targets` refinement: APPROVED — codified as the cheaper honest compile-only gate alongside `cargo test --no-run`.
- Rejected alternatives stand: no new `docs/testing.md` for this rule (belongs in README; distinct from the OXA-000066/68 TESTING.md), no `bin/check.sh` wrapper.
- Acceptance = Verification "after the doc lands": `grep -n "cfg(test)" README.md` hits; register line carries pointer; no automation changed.
