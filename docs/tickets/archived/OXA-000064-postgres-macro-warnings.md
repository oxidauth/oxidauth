# OXA-000064 — `database!` macro noise in the `postgres` test build (T-4's path and mechanism are both wrong)

**Original ID:** T-4 · **Severity:** n/a · **Type:** tooling · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #18 of 27 · reviewed 2026-09-29 · SCHEDULED (lands standalone)


## Locations

- `BUGS_AND_NOTES.md:99` — the T-4 line under audit ("sqlx `#[test]` expansion in `test-migrations/db_tests/mod.rs`", "upstream-only fix")
- `src/xlib/postgres/src/lib.rs:302-512` — `#[cfg(test)] mod db_tests` (doc comment `:303-306`, `mod db_tests {` at `:307`); the crate's **only** `database!` invocation, at `:325`, inside this module
- `src/xlib/postgres/src/lib.rs:97-103` — the five `use` statements `database!` injects at its invocation site (`std::env` `:98`, `async_trait::async_trait` `:100`, `$crate::ping` `:101`, `pub use $crate::{PgError, Ping, PingTrait, mock}` `:102`, `sqlx::{PgPool, migrate::Migrator}` `:103`)
- `src/xlib/postgres/src/lib.rs:312-314` — the existing NOTE: the author already knew the expansion injects `use sqlx::{PgPool, migrate::Migrator};` and skipped an explicit `PgPool` import to avoid E0252
- `src/xlib/postgres/src/lib.rs:219-225` / `:227-235` — `migrator_const!` / `migrator!`; both arms spell the type as the qualified path `sqlx::migrate::Migrator`, never the imported short name
- `src/xlib/postgres/src/lib.rs:181-215` — `DatabaseBuilder`; the two setters at `:192-198` have **zero** call sites in `src/`
- `src/xlib/postgres/src/ping/mod.rs:14-24` — the crate's only `#[sqlx::test]` (`:18`): the attribute T-4 blames
- `src/oxidauth/oxidauth-postgres/src/lib.rs:45-52` — the only other `database!` call site (external crate, crate root; its doc comment `:3-6` explains why the re-exports there are the point)
- `src/oxidauth/database_test.sh:18` (`cargo test -p oxidauth-postgres -p postgres`, driven by `bin/database_test.sh`) — the lane that shows the noise; `bin/unit_test.sh:6` excludes `postgres` and `*-postgres`
- `~/.cargo/registry/src/index.crates.io-…/sqlx-macros-core-0.8.6/src/test_attr.rs:63-186` — the `#[sqlx::test]` expansion (version matches `Cargo.lock:2652-2654`)
- `~/.cargo/registry/src/index.crates.io-…/async-trait-0.1.92/src/lib.rs` — the `#[async_trait]` expansion (`tracing::instrument` is applied at `lib.rs:114`, `:130`, `:144`)
- `docs/tickets/archived/OXA-000061-stage-test-migrations-gitkeep.md:38`, `:64` — already flagged T-4's stale path and assigned its correction to T-1's retirement
- `src/oxidauth/oxidauth-api/src/middleware/can.rs:83-84` — precedent: `#[allow(deprecated)]` on a `mod tests` to neutralize macro-inherent lint noise (the same posture as register T-7, `BUGS_AND_NOTES.md:102`)

## Problem

T-4's *observation* (a fixed number of macro-generated warnings in the `postgres` test build) is real; three of its four claims about it are not.

1. **The path never existed.** No `test-migrations/db_tests/mod.rs`, nor any `db_tests` file, exists (`find src -path '*db_tests*'` → empty; `src/xlib/postgres/test-migrations/` holds only the 0-byte `.gitkeep` tracked by OXA-000061). The code is `src/xlib/postgres/src/lib.rs:302-512`.
2. **The named mechanism cannot produce the symptom.** `#[sqlx::test]`'s expansion emits *no* `use` items at all — everything is fully qualified (`::sqlx::test_block_on`, `::sqlx::testing::TestArgs::new`, `::core::prelude::v1::test`; `sqlx-macros-core-0.8.6/src/test_attr.rs:69-76` and `:166-185`). `#[async_trait]` likewise emits zero `use` items (grep over `async-trait-0.1.92/src/lib.rs` → no `quote!` block containing a `use`). So no import warning in this crate can originate from either attribute macro. The only macro in the build that injects imports is `database!` (`lib.rs:97-103`).
3. **"upstream-only fix" is false.** A module-level `#[allow(unused_imports)]` on `mod db_tests` silences the expansion's imports — verified with a standalone `rustc` repro (§Analysis 3, probe 1). `mod db_tests` is *our* code: `git show HEAD:src/xlib/postgres/src/lib.rs` is 300 lines with no `db_tests`/`cfg(test)` hit (`grep -c` → 0); the working-tree file is 512 lines. The fix therefore needs no vendored-template divergence at all (`lib.rs:1` "re-diff when templates change").
4. **The count is right, the label is wrong — [INFERENCE].** By inspection, exactly two of the injected imports can be unused (`Ping`, `migrate::Migrator`), and the *same* expansion also emits one `dead_code` group (the two never-called `DatabaseBuilder` setters). 2 unused-import + 1 dead-code = 3 macro-origin warnings, which is what a std-only shape repro of the module prints (`rustc --test --edition 2024`, probe 5 → "3 warnings emitted"). I did not run the crate's build (out of ticket scope), so the identity of all three in the real build is inference; §Verification pins them in one command.

## Analysis

**1. Only `database!` injects imports, and only in-crate invocations are linted.** `database!` is a `#[macro_export] macro_rules!` (`lib.rs:89-90`) whose body begins with five `use` items (`:98-103`), so every call site gets those bindings in its own module. Standalone probes under the pinned toolchain (`rust-toolchain.toml` → `rustc 1.98.0`), all in `/tmp`, repo untouched:

- *probe 1* (`macro_rules!` in the same crate, invoked inside `mod a` / `mod b` / `mod c`): macro-generated `use` items **do** warn, with the span pointing at the macro *definition* plus an `… in this macro invocation` note; `#[allow(unused_imports)]` on the **enclosing module** (mod b) silences them completely; `#[allow(unused_imports)]` on the macro *invocation* (mod c) is **ignored** and itself warns (`unused_attributes`).
- *probe 3* (`#[macro_export]` macro compiled as an rlib, invoked from a second crate): the same unused imports produce **no warnings at all** in the consumer crate, while a plain unused `use` in that same crate still warns (control). So `oxidauth-postgres`'s call site (`oxidauth-postgres/src/lib.rs:47-52`) is silent — its `pub use` re-exports are exactly the public surface its doc comment at `:3-6` advertises — and the noise is unique to `postgres`'s self-invocation. "Upstream" would fix nobody in this workspace.
- *probe 2*: `use super::*` does re-import the parent module's **private** `use` declarations, so `db_tests`' `VarError` (`:385`) and `Error` (`.source()` calls, `:380`+) keep the glob alive → no glob-side warning, and the glob-vs-explicit shadowing at `:102` is legal (the expansion's NOTE at `:312-314` shows the author already relied on this).

**2. Which injected imports are dead, by construction.** Three of the five always have a named user inside any expansion: `std::env` (`env::var` at `:116`, `:146`), `async_trait::async_trait` (attribute at `:171`), `$crate::ping` (`ping::ping` at `:174-175`); `PgPool`/`PgError`/`PingTrait`/`mock` are used by the expansion and the test bodies (`mock::PingMock` at `:429`, `impl PingTrait` at `:172`). The two that are never named: `Ping` (nothing in `lib.rs:302-512` writes a bare `Ping` — grep over the module → 0 hits; it exists purely as a downstream re-export) and `migrate::Migrator` — dead in **both** macro arms, because `migrator_const!(…)`/`migrator!(…)` (`:219-235`) spell `sqlx::migrate::Migrator` fully qualified or use the caller's ident. That second one is a genuine defect in the vendored macro (harmless, but the upstream template should drop it or `#[allow(unused_imports)]` the arms).

**3. rustc groups, so "3 warnings" ≠ "3 unused imports."** Multiple unused names inside one `use` group are reported as a single message (`warning: unused imports: \`A\` and \`B\``; probe 1), and multiple never-used methods of one impl are one `dead_code` message (probe 5). Hence the register's "3" is only consistent with the source if it counted *messages across both lints*, e.g. `unused import: migrate::Migrator` + `unused import: Ping` + ``methods `database_url` and `read_database_url` are never used``. Alternative reconstruction, if the build log disagrees: `mock` may have been unused before the `ping_mock_replays_ok_but_downgrades_any_error` test (`:425-437`) was written, which would make it 3 names across 2 messages. Either way the mechanism is `database!`, not sqlx.

**4. Fix shape: `#[allow]` for the imports, a real test for the dead code.** `#[allow(unused_imports)]` on `mod db_tests` kills the two import warnings at their only site of relevance (probe 1, mod b). For the `dead_code` group, a blanket module-level `#[allow(dead_code)]` is the lazy option and would also mask genuinely dead helpers (`poll_ready` `:340`, `lazy_pool` `:353`, `env_guard` `:333`); better is to *call* the two setters — `let mut b = DatabaseBuilder::new(); b.database_url(NEVER_CONNECTS_URL.to_string()); b.read_database_url(…);` without ever `await`ing `build()` stays I/O-free (the module's no-live-database rule, `:303-306`; `build()` would call `PgPool::connect` at `:205`), so the lint keeps biting on real dead code.

**5. Convention home.** There is no `CONTRIBUTING.md` (repo root and `docs/`); the postgres test conventions already live in the register (T-6, `BUGS_AND_NOTES.md:101`) and in plan 11 (`docs/migration-plan/11-scripts-and-tests.md`). This item is a one-line build-noise fact, not a convention: fix it and keep one corrected line, rather than promoting a permanent exception. Register T-7 (`:102`) remains the instructive contrast, though not as first written — its `Service` deprecation noise is plan-09 migration telemetry, not inherent to the mock convention (OXA-000067 corrected the attribution: the mock sites are already silenced by allows; the remaining warnings must stay audible until the trait retires), so its register line stays KEEP as a corrected do-not-`#[allow]` record, while T-4's is a two-token annotation on our own module.

## Impact

- Cosmetic only, but permanent and mis-attributed: every `src/oxidauth/database_test.sh` run (`:18`) prints the warnings, and the register sends a future reader to sqlx and to a file path that does not exist. It is the same file T-1 needs staged (OXA-000061) — the two items share the cutover commit.
- Warning desensitization: `bin/unit_test.sh:6` never compiles this module and `cargo check` never compiles `#[cfg(test)]` (register T-3, `:98`), so the DB lane is the *only* place this crate's test code is linted. Three standing warnings there is precisely where a new, real one gets lost.
- Zero product/runtime/data/security impact: nothing here is in a shipped artifact (`bin/publish.sh` publishes only `src/oxidauth/oxidauth-*`, per OXA-000061, never `src/xlib/postgres`), and the fix is test-profile-only.

## Proposed resolution

**Disposition: FIX NOW** — two lines in code we own, in the same cutover commit as OXA-000061's `git add`; then rewrite the T-4 register line instead of deleting the noise into perpetuity.

1. Annotate the module in `src/xlib/postgres/src/lib.rs` — one attribute inside the `:302-307` block, directly above `mod db_tests {` (attributes and the existing `///` block at `:303-306` may interleave; this ordering keeps the doc comment adjacent to the module):

   ```rust
   #[cfg(test)]
   /// A11.2: structural coverage for the `database!` macro + `PgError` mappings
   /// … existing doc block, `:303-306`, unchanged …
   #[allow(unused_imports)] // `database!` injects its `use` items into this module;
   mod db_tests {           // `Ping`/`migrate::Migrator` are only live for callers
       …                    // of the macro outside it
   ```

   Hand-format it — `cargo fmt` is still blocked repo-wide by T-2/OXA-000062.

2. Kill the `dead_code` group with coverage instead of an allow: extend `builder_without_url_fails_before_any_connection_attempt` (`:439-448`), or add one sibling test, to call `DatabaseBuilder::database_url` and `DatabaseBuilder::read_database_url` and drop the builder without `build()` — no I/O, no new `#[allow]`.

3. Rewrite `BUGS_AND_NOTES.md:99` (T-4 owns this edit; drop the duplicate from OXA-000061 step 3, `:64`, which currently also claims it — land it in whichever ticket commits first):

   > **T-4** `postgres` compiles `database!` *inside* its own `#[cfg(test)] mod db_tests` (`src/xlib/postgres/src/lib.rs:302-512`), so the macro's injected `use` items and its never-called `DatabaseBuilder` setters get linted there (rustc lints macro-injected imports only for same-crate invocations — `oxidauth-postgres`'s call site is silent). Silenced with `#[allow(unused_imports)]` on `mod db_tests` + one test that exercises the builder setters. (The old line's "sqlx `#[test]` expansion in `test-migrations/db_tests/mod.rs`" was wrong twice: sqlx's test attribute injects no imports, and that path never existed — OXA-000061.)

4. Optional, non-blocking upstream/template nit (freshbrewlabs project-template, `lib.rs:1`): drop the unused `migrate::Migrator` from the injected import list at `:103` (dead in both arms) or add `#[allow(unused_imports)]` inside the expansion. Do not wait on it — it changes nothing for this workspace's other consumer.

Rejected: (a) KEEP as "known noise" — rests on the wrong mechanism and on "upstream-only", which probe 3 disproves; (b) `#[allow(unused_imports)]` on the macro *invocation* (`:325`) — verified no-op that adds an `unused_attributes` warning (probe 1, mod c); (c) deleting the injected `pub use` re-exports or the setters from the macro — they are the macro's public contract for `oxidauth-postgres` (`oxidauth-postgres/src/lib.rs:3-6`); (d) module-wide `#[allow(dead_code)]` — masks the module's own dead helpers to save one 4-line test.

## Verification

Already gathered, read-only plus standalone `rustc` probes in `/tmp` (no crate build, lint, or fmt run in the repo; no repo file touched):

- Path/mechanism: `find src -path '*db_tests*'` → empty; `grep -rn "sqlx::test" src/xlib/postgres/src` → only `ping/mod.rs:18` and the doc example at `lib.rs:83`; `sqlx-macros-core-0.8.6/src/test_attr.rs:63-186` and `async-trait-0.1.92/src/lib.rs` → no `use` in either expansion; `Cargo.lock:2652-2654` confirms the read sqlx version is the locked one.
- `git show HEAD:src/xlib/postgres/src/lib.rs | grep -cE "db_tests|cfg\(test\)"` → `0` (HEAD file is 300 lines vs 512 now) → the module, and any allow on it, is ours.
- Lint mechanics (probe 1/2/3/5, `rustc 1.98.0`, `--test --edition 2024`): macro-injected imports warn with definition-site spans; module-level `#[allow(unused_imports)]` silences them; invocation-level `#[allow]` is ignored; `#[macro_export]` invocations from another crate emit no import warnings (control confirms the lint is on); `use super::*` does carry parent private imports; the faithful std-only repro of `mod db_tests`' call graph prints exactly **3** macro-origin warnings (2 unused-import groups + 1 `dead_code` method group).
- Dead generated API: `grep -rn "from_database_url\|\.database_url(\|\.read_database_url(" --include=*.rs src` → `from_database_url` *is* live via `from_env` (`lib.rs:120`, called at `:459`/`:475`); the two builder setters have no call site anywhere in `src/`.

After the fix, in this order (`cargo check -p postgres` is **not** proof — register T-3):

1. Pin the trio **before** applying, to close the [INFERENCE] tag: `cargo test --no-run -p postgres 2>&1 | grep -B6 "in this macro invocation"` — expected: `unused import: \`Ping\``, `unused import: \`migrate::Migrator\`` (both spanning the same `mod db_tests` invocation), and ``methods `database_url` and `read_database_url` are never used``. If the build disagrees, correct this ticket's §Problem 4 and the T-4 line to the observed set. (Requires `src/xlib/postgres/test-migrations/` on disk — OXA-000061.)
2. Apply, re-run step 1: expected zero `postgres`-source warnings, and specifically **no** `unused_attributes` warning (that would mean the allow was pinned to the invocation at `:325` instead of the module).
3. `bin/database_test.sh` (→ `src/oxidauth/database_test.sh:18`) must still pass — `ping/mod.rs:18`'s `#[sqlx::test]` needs the compose Postgres and `DATABASE_URL`, which the working-tree-only `mod db_tests` tests deliberately do not (`lib.rs:303-306`), so this is the one command that exercises both.
4. `bin/unit_test.sh` must stay green (it excludes both postgres crates, so it is a regression guard on the *other* crates only).

## Decision (2026-09-29) — ACCEPTED as proposed (one coordination step struck), scheduled; no implementation started
- Reviewed jointly (scout coordinate sweep + owner ruling 2026-09-29). Every coordinate CONFIRMED: `mod db_tests` is `src/xlib/postgres/src/lib.rs:306-512` (absent at HEAD — our code), `database!` invocation `:325`, injected imports `:97-103`, dead setters `:192-198` (zero callers), sole `#[sqlx::test]` at `ping/mod.rs:18`, `oxidauth-postgres` call site `:47-52`, register line `:99` verbatim, zero existing `allow(unused_imports)` patterns in `src/`.
- T-4's three register errors ratified for the record: the path never existed; neither `#[sqlx::test]` nor `#[async_trait]` injects imports (only `database!` does, and only same-crate invocations are linted); "upstream-only" is false — the fix is ours, two tokens + one test.
- Step 1 approved: `#[allow(unused_imports)]` on the MODULE, hand-formatted (cargo fmt still blocked repo-wide by OXA-000062). Step 2 approved: kill the `dead_code` group with a real test calling `DatabaseBuilder::database_url` + `read_database_url` without `build()` (I/O-free) — not a blanket allow. Step 3 approved: rewrite `BUGS_AND_NOTES.md:99` with the ticket's drafted text (register owns that edit).
- **STRUCK: the "same cutover commit as OXA-000061's git add" step.** OXA-000061 was reviewed and closed NO-ACTION (residual risk accepted; `test-migrations/.gitkeep` stays working-tree-only). This lands standalone. The fresh-clone test-build risk is 000061's accepted residue — noted, not re-litigated here.
- [INFERENCE] tag preserved honestly: implementer pins the warning trio FIRST (`cargo test --no-run -p postgres 2>&1 | grep -B6 "in this macro invocation"` — expect `Ping`, `migrate::Migrator`, dead-setter group); if the build disagrees, correct §Problem 4 and the rewritten T-4 line to observed, per the ticket's own instruction.
- Ratified rejections: KEEP-as-noise (wrong mechanism), invocation-level allow (verified no-op that adds `unused_attributes`), deleting injected `pub use` re-exports/setters (public contract of `oxidauth-postgres`), module-wide `#[allow(dead_code)]` (masks live helpers to save a 4-line test).
- Acceptance: pin run shows the trio before and ZERO postgres-source warnings after (specifically no `unused_attributes`); `bin/database_test.sh` green (the one command compiling `mod db_tests` + hitting `#[sqlx::test]`); `bin/unit_test.sh` unaffected.
