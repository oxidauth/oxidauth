# OXA-000061 — Stage the empty `test-migrations` directory before the cutover commit

**Original ID:** T-1 · **Severity:** n/a · **Type:** tooling · **Status:** done
**Review tier:** Tier 1 (docs-only), ranked item 4/9 · CLOSED as NO-ACTION (residual risk accepted by owner; register note landed)


## Locations

- `src/xlib/postgres/test-migrations/.gitkeep` — on disk (0 bytes, `2026-09-29 19:10`), **untracked and unstaged**
- `src/xlib/postgres/src/lib.rs:302-325` — `#[cfg(test)] mod db_tests` (A11.2); `sqlx::migrate!("./test-migrations")` at `:323`, `crate::database!(…)` at `:325`; empty-dir assertion at `:361-364`
- `src/xlib/postgres/src/lib.rs:219-235` — `migrator_const!` / `migrator!`, the macro arms the 4-arg `database!` form selects
- `.gitignore:28` — `!**/.gitkeep` (the register line cites `:31`; `:31` is `**/values-staging.yaml`)
- `src/oxidauth/oxidauth-postgres/src/lib.rs:45-52` — sibling 4-arg `database!` call whose `sqlx::migrate!("./migrations")` dir *is* tracked
- `bin/unit_test.sh:6`, `src/oxidauth/database_test.sh:18` — the two test entrypoints (both themselves still untracked)
- `secrets/.gitkeep` — precedent: same pattern, already tracked in HEAD

## Problem

T-1 is still live — there is nothing to retire. Verified state:

- `git ls-files -s -- src/xlib/postgres/` lists only `Cargo.toml`, `src/lib.rs`, `src/ping/mod.rs`, `src/ping/ping.sql` — no `.gitkeep` entry.
- `git status --porcelain -uall -- src/xlib/postgres/` → `?? src/xlib/postgres/test-migrations/.gitkeep` (untracked, *not* ignored), and `git diff --cached --name-only -- src/xlib/postgres/` is empty (not staged).
- `git archive HEAD src/xlib/postgres | tar -t` → no `test-migrations/` at all. Git does not track empty directories, so the directory does not exist in any clone of HEAD.
- The register's parenthetical is off by three lines: `!**/.gitkeep` is `.gitignore:28`, identical in the worktree and in HEAD (`git status --porcelain -- .gitignore` → clean; `git show HEAD:.gitignore | grep -n gitkeep` → `28:`). The pattern's *effect* is exactly as the note claims, so the action item stands; only the line reference is wrong.

## Analysis

**Why the directory must exist at compile time.** `sqlx::migrate!` is a compile-time macro. It resolves the literal relative to `CARGO_MANIFEST_DIR` (`sqlx-macros-core-0.8.6/src/common.rs:5-31`), then `expand_migrator` calls `path.canonicalize()` and, on failure, emits `compile_error!("error canonicalizing migration directory …")` (`sqlx-macros-core-0.8.6/src/migrate.rs`, `expand_migrator`; error surfaced as `::std::compile_error!` at `sqlx-macros-0.8.6/src/lib.rs:71-80`). `./test-migrations` resolves to `src/xlib/postgres/test-migrations`, so a checkout without that directory cannot build the crate's tests. `.gitkeep` is the only thing that makes a directory with no `.sql` files existable in git.

**The register's wording is slightly wrong, the conclusion is right.** The macro invocation at `lib.rs:325` is the 4-arg form, which routes through `migrator_const!(TEST_MIGRATOR)` → empty expansion (`lib.rs:220-225`) and therefore *skips* the macro's own `pub const MIGRATOR = sqlx::migrate!()`. Nothing inside the `database!` expansion touches `./test-migrations`. The hard dependency is the `TEST_MIGRATOR` const at `lib.rs:323` in the same `mod db_tests` that calls `database!`. Net effect for the cutover is identical: the test build of the crate that owns `database!` needs the directory.

**The keep-file is inert for sqlx.** `resolve_blocking` skips any entry that is not `<VERSION>_<DESCRIPTION>.sql` (`sqlx-core-0.8.6/src/migrate/source.rs:57-61, 98-101`); `.gitkeep` splits into one `_`-part and is skipped, so `TEST_MIGRATOR.migrations` stays empty exactly as `test_migrator_compiles_empty_migration_dir_to_zero_migrations` (`lib.rs:361-364`) and the comment at `lib.rs:320-322` ("EMPTY `./test-migrations` dir, so nothing here can ever carry a real migration") require. An empty `.gitkeep` is the only content that satisfies both.

**Ignore rules.** `!**/.gitkeep` is present and effective; there is no `.git/info/exclude` customization and no `core.excludesFile`. No pattern in `.gitignore` would match a `.gitkeep` name anyway, so line 28 is defensive — the same posture under which `secrets/.gitkeep` is already committed. Practical caveat: `git check-ignore -v <path>` prints `.gitignore:28:!**/.gitkeep` and exits 0 even though the file is *not* ignored (`-v` reports the last matching pattern, negations included). The authoritative signals are the `??` (not `!!`) status in `git status --porcelain -uall --ignored` and the dry-run below.

**HEAD does not fail; the cutover commit is what introduces the requirement.** `git show HEAD:src/xlib/postgres/src/lib.rs` has no `#[cfg(test)]` block and no `./test-migrations` reference — the only `migrate!` hits are doc comments and the macro's default `sqlx::migrate!()`. The `mod db_tests` module is part of the +212-line uncommitted A11.2 diff. So HEAD compiles, and the failure appears the moment the test module lands in a commit that omits the directory. Both must be in the same commit.

**Who would notice.** There is no CI: `.github/` contains only `PULL_REQUEST_TEMPLATE.md`, no `workflows/`. Locally, `cargo test -p postgres` / `cargo test --no-run -p postgres` (the tail of `src/oxidauth/database_test.sh:18`) hit the compile error; `bin/unit_test.sh:6` excludes `postgres` and `*-postgres`, and plain `cargo check` never compiles `#[cfg(test)]` at all (register T-3), so both stay green and the break is invisible until someone runs the database suite on a fresh clone. `bin/publish.sh:34-44` publishes only `src/oxidauth/oxidauth-*` folders (never `src/xlib/postgres`) and refuses a dirty tree, so there is no packaging exposure today.

**Adjacent stale text (not this item's fix).** T-4 attributes the 3 macro-generated unused-import warnings to "`test-migrations/db_tests/mod.rs`"; no such path exists (`find src -path '*db_tests*'` → empty). The code is `src/xlib/postgres/src/lib.rs:302+`. Correct T-4's reference when this item is retired.

**`BUG(pinned)` audit:** grep for `test-migrations` / `gitkeep` across `src`, `bin`, `docs` returns only `src/xlib/postgres/src/lib.rs:321-323`. No `BUG(pinned)` markers guard this area, and none of OXA-000001…000058 references the postgres test harness.

## Impact

- Every clean clone of the cutover commit fails to compile the `postgres` crate's tests with `error canonicalizing migration directory …/src/xlib/postgres/test-migrations: No such file or directory (os error 2)` — the A11.2 `database!` coverage (`lib.rs:302-512`) is unreachable there, and `bin/database_test.sh` (which runs `cargo test -p oxidauth-postgres -p postgres`) fails at compile time, before any database is contacted.
- Silent until noticed: `cargo check` and `bin/unit_test.sh` both pass on the broken clone, so CI-shape checks would report green.
- Zero product/runtime/data/security impact — the failure is entirely compile-time on the test profile. Pure bookkeeping risk: the working tree currently has 386 untracked paths, so a single 0-byte keep-file is exactly the kind of thing the cutover commit drops.

## Proposed resolution

**Disposition: FIX NOW** — one `git add` in the cutover commit; no code change.

1. Stage the keep-file (plain add is sufficient, no `-f`; verified below):

   ```sh
   git add src/xlib/postgres/test-migrations/.gitkeep
   git ls-files --stage -- src/xlib/postgres/test-migrations/
   # expect: 100644 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 0 src/xlib/postgres/test-migrations/.gitkeep
   ```

   `100644` mode and the empty-blob hash `e69de29b…` are the expected values; keep the file 0 bytes (any `.sql`-shaped name, or any `.sql` content, would break the empty-migrator assertion at `lib.rs:361-364`).

2. Land it in the same commit as the `src/xlib/postgres/src/lib.rs` test module and the three untracked test scripts, then note the addition in the plan-11 changelog under `changelogs/`.

3. Retire T-1, and while in `BUGS_AND_NOTES.md` fix the two stale references this ticket found: T-1's `.gitignore:31` → `.gitignore:28`, and T-4's `test-migrations/db_tests/mod.rs` → `src/xlib/postgres/src/lib.rs` (`mod db_tests`).

Rejected alternatives: pointing `TEST_MIGRATOR` at `./migrations` (contradicts `lib.rs:320-322` — a test migrator must never carry real migrations, and `#[sqlx::test]` would replay production migrations); committing a placeholder `.sql` (breaks the zero-migration assertion); hand-writing `Migrator::DEFAULT` to dodge the directory entirely (`#[doc(hidden)]`, fields documented as semver-exempt — brittle for a fixture whose whole job is proving the macro compiles).

## Verification

Read-only git evidence already gathered (no builds/tests were run, per ticket scope):

- `git ls-files -s -- src/xlib/postgres/` → no `.gitkeep`; `git diff --cached --name-only -- src/xlib/postgres/` → empty; status porcelain → `??`. Item is genuinely open.
- `git archive HEAD src/xlib/postgres | tar -t` → `Cargo.toml`, `src/lib.rs`, `src/ping/{mod.rs,ping.sql}` only — HEAD carries no `test-migrations/`.
- Ignore-rule dry run against a throwaway index (the real `.git/index` was verified unchanged afterwards, `git ls-files --stage -- src/xlib/postgres/test-migrations/` → empty):
  `cp .git/index /tmp/oxa61-idx && GIT_INDEX_FILE=/tmp/oxa61-idx git add src/xlib/postgres/test-migrations/.gitkeep && GIT_INDEX_FILE=/tmp/oxa61-idx git ls-files --stage -- src/xlib/postgres/test-migrations/` → `100644 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 0 …/.gitkeep`, i.e. the add is not ignore-blocked and the blob is empty.

After the fix, whoever lands it should confirm:

1. `git ls-files -- src/xlib/postgres/test-migrations/.gitkeep` prints the path, and `git cat-file -s $(git rev-parse :src/xlib/postgres/test-migrations/.gitkeep)` prints `0`.
2. Clean-clone check in a scratch dir — `git clone . /tmp/oxa-61 && ls -a /tmp/oxa-61/src/xlib/postgres/test-migrations` shows `.gitkeep` (this is the exact condition a checkout needs; remove the scratch clone afterwards).
3. Compile proof on that clone: `cargo test --no-run -p postgres` succeeds (`cargo check` is NOT proof — register T-3), and `cargo test -p postgres test_migrator_compiles_empty_migration_dir_to_zero_migrations` passes with the clone's database stack absent, confirming the empty dir resolved to zero migrations rather than being silently replaced.

## Decision (2026-09-29) — RESOLUTION CHANGED by owner; scheduled as no-action
- **Review tier: Tier 1 (docs-only), item 4/9** — resolution changed to no-action; residual follow-on is a one-line register note.
- Owner decision: do NOT stage/track `test-migrations/.gitkeep` as a planned action — the vendored `postgres` crate is a lightweight macro and this bookkeeping is not worth guarding. The original FIX-NOW resolution (`git add` + same-commit invariant + changelog note) is WITHDRAWN.
- Residual risk ACCEPTED knowingly: if the uncommitted A11.2 `mod db_tests` (`src/xlib/postgres/src/lib.rs:323`) lands without the directory, a clean clone fails `cargo test --no-run -p postgres` with `compile_error!` at compile time (test profile only; no product/publish exposure; `cargo check`/`unit_test.sh` stay green). Normal `git add .` habit usually captures the keep-file anyway.
- Reopen triggers: any clean-clone failure of the postgres test build, or any CI workflow landing that gates on it. Escape hatches if it bites: (a) plain `git add src/xlib/postgres/test-migrations/.gitkeep` after all, or (b) drop the `TEST_MIGRATOR`/`./test-migrations` instantiation (trade-away: `test_migrator_compiles_empty_migration_dir_to_zero_migrations` coverage).
- Follow-on docs work folded in, no code action:
  1. At next `BUGS_AND_NOTES.md` maintenance pass: replace the T-1 action line with a one-line accepted-risk note (or strike it); the `.gitignore:31`→`:28` correction only matters if T-1 text survives.
  2. T-4's stale path fix is OWNED BY OXA-000064 (its full T-4 rewrite subsumes it); this ticket must not edit the T-4 line.
- Ticket closes when the register note lands; no git/index changes are expected from this ticket.
