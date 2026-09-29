# 02 — Move crates into src/ layout

**Status**: `in-progress` (impl done 2026-09-29; review loop pending — orchestrator flips to done at close)
**Depends on**: 01
**Risk**: medium — pure `git mv` + manifest edits; no code changes. The
workspace must build identically before/after.

## Goal

`oxidauth` becomes a PROJECT whose single stack is `src/oxidauth/`, matching
parkinglot (`src/admin/admin-*`) and the generated baseline
(`src/tmpstack/tmpstack-*`). Crate **names do not change** — the stack name is
`oxidauth` and every crate already carries that prefix, so this is paths-only.

## Changes

1. `git mv` every crate directory (current root members):
   | old | new |
   |---|---|
   | `oxidauth-kernel/` | `src/oxidauth/oxidauth-kernel/` |
   | `oxidauth-repository/` | `src/oxidauth/oxidauth-repository/` |
   | `oxidauth-postgres/` | `src/oxidauth/oxidauth-postgres/` |
   | `oxidauth-usecases/` | `src/oxidauth/oxidauth-usecases/` (renamed to `-services` in plan 09) |
   | `oxidauth-http/` | `src/oxidauth/oxidauth-http/` (split in plan 07) |
   | `oxidauth-rs/` | `src/oxidauth/oxidauth-rs/` |
   | `oxidauth-permission/` | `src/oxidauth/oxidauth-permission/` |
   | `oxidauth-cli/` | `src/oxidauth/oxidauth-cli/` (stub kept per review) |
   | `oxidauth-import-export/` | `src/oxidauth/oxidauth-import-export/` (stub kept per review) |
   | `oxidauth-seed/` | `src/seedz/` (project-level, per template; content reworked in plan 13) |
   | `oxidauth-http/hurl/` | `src/oxidauth/hurl/` |
2. **Keep the stub crates** `oxidauth-cli/`, `oxidauth-import-export/` (empty
   `lib.rs`, zero deps, zero reverse-deps) — **moved, not deleted** (review
   decision 2026-09-29); they are placeholders for future stack features and
   ride the `src/oxidauth/*` member glob unchanged. `oxidauth-telemetry/` is
   NOT deleted here — plan 06 removes it when xlib/telemetry lands.
3. **Root `Cargo.toml`** — keep `[workspace] resolver = "2"` and
   `[profile.dev.package.num-bigint-dig] opt-level = 3`; replace members:
   ```toml
   members = [
     "src/oxidauth/*",
     "src/seedz",
   ]
   exclude = [
     "src/oxidauth/helm",
   ]
   ```
   (`src/xlib/*` is added in plan 03 — an empty-glob member errors.)
   `exclude` for `helm` matters once plan 12 adds the chart; add it now.
4. **Relative path deps need no edits** — all inter-crate deps are
   `path = "../oxidauth-*"` and crates move together under the same parent.
   `src/seedz/Cargo.toml` deps pointing at `../oxidauth-kernel` must become
   `../oxidauth/oxidauth-kernel` (seedz sits at `src/`, not in the stack).
   Audit: `grep -rn 'path = "\.\./' src/seedz` and fix.
5. **Path-sensitive stragglers** (audit with `grep -rn 'oxidauth-http/\|oxidauth-postgres/\|\.\./oxidauth' \
   bin/ *.yml *.Dockerfile .github/ 2>/dev/null`):
   - `docker-compose.yml` volume `".:/home/rust/src/oxidauth:cached"` — still
     correct (mounts repo root), but `cargo run --bin oxidauth-http` still
     resolves from root workspace — OK until plan 10 rewrites compose.
   - `bin/publish.sh`, `bin/build-server.sh` — if they reference crate dirs,
     update paths now (rewrite proper in plans 11/12).
   - `oxidauth-postgres/migrations/` moves inside the crate — unchanged path
     relative to its crate (`sqlx::migrate!()` resolves crate-relative). ✓
   - `sqlx-cli`/`DATABASE_URL` scripts referencing `--migrations-path
     oxidauth-postgres/migrations` → `src/oxidauth/oxidauth-postgres/migrations`
     (`bin/reset-db.sh`).
6. Commit in two steps: (a) `git mv` only (git detects renames), (b) manifest +
   script path fixes. Reviewers can then skim `-M` diff cleanly.

## Verification

- `cargo metadata --format-version 1 | jq '.packages[].name'` lists the exact
  same crate set as before (stub crates retained per review).
- `cargo check --workspace` green.
- `cargo run --bin oxidauth-http` still boots against local postgres
  (or accept `cargo check` + `cargo test --workspace --exclude oxidauth-postgres`
  if DB unavailable).
- `bin/hurl-tests.sh` glob still matches (moved to `src/oxidauth/hurl/tests/**`
  — glob is `hurl/**/*.hurl` relative to oxidauth-http dir → update glob root
  to `src/oxidauth/hurl` in this plan; full hurl.sh convention in plan 11).
- `sqlx database drop --create` + `sqlx migrate run` from `src/oxidauth/oxidauth-postgres` work.

## PR note

changelog `changelogs/<id>-src-layout.md`; flip Status → `done`.

## Review notes

- 2026-09-29 walkthrough: **keep `oxidauth-cli` + `oxidauth-import-export`**,
  move them into `src/oxidauth/` with everything else (original draft deleted
  them). Remainder approved as written.

## Execution note

- Executed 2026-09-29 (all crates `git mv`'d per the table; stubs kept).
- **Telemetry interim**: `oxidauth-telemetry/` stays at the repo root (plan 06
  deletes it). A root directory cannot match `src/oxidauth/*`, so the root
  `Cargo.toml` lists it as an explicit member `"oxidauth-telemetry"` (marked
  `removed in plan 06`). The initially drafted member string
  `"src/oxidauth-telemetry"` was NOT used — the crate was not moved, so the
  member path is root-relative.
- **Plan item 4 straggler (missed above)**: `oxidauth-http` depends on
  `oxidauth-telemetry = { path = "../oxidauth-telemetry" }`. Telemetry stayed
  at root while its dependent moved, so the dep now reads
  `path = "../../../oxidauth-telemetry"`. All other `../oxidauth-*` sibling
  deps verified resolvable unchanged; `src/seedz/Cargo.toml` has zero
  dependencies (stub), so no repointing was needed there.
- **hurl script before/after**: `bin/hurl-tests.sh` did
  `pushd oxidauth-http`; now `pushd src/oxidauth`. The `hurl/**/*.hurl` globs,
  `hurl/variables-local`, and `hurl/public_keys_create.hurl` are untouched —
  identical semantics since `hurl/` is now a direct child of the cd target.
- **Other stragglers**: `bin/reset-db.sh` `pushd oxidauth-postgres` ->
  `pushd src/oxidauth/oxidauth-postgres` (sqlx resolves `./migrations`
  crate-relative; `DATABASE_URL` left hardcoded per plan); `bin/publish.sh`
  `FOLDER=src/oxidauth/oxidauth-$PROJECT`; `bin/build-server.sh` version probe
  and `-f` Dockerfile paths -> `src/oxidauth/oxidauth-http/`. No crate-dir
  paths exist in README.md, dev.Dockerfile, docker-compose.yml (mounts the
  repo root — still correct), `.github/`, `devops/`, or `crypt-keeper.toml`.
  `docs/SECURITY_REPORT.md` and other historical docs keep old paths as-is.
- `src/oxidauth/hurl/` has no `Cargo.toml`, and the `src/oxidauth/*` member
  glob does NOT silently skip manifest-less matches — cargo errors on them
  (`cargo metadata` failed until the fix). The shipped root `Cargo.toml`
  therefore excludes it: `exclude = ["src/oxidauth/hurl",
  "src/oxidauth/helm"]`. Relevant to plan 03, which adds `src/xlib/*`.

## Review feedback

Independent review 2026-09-29 (read-only: `git status`/`git diff HEAD`, full
reads of the changed scripts, both Dockerfiles, compose, manifests, changelog;
`cargo check` green per orchestrator after a one-line orchestrator fix).

### Checklist

- **§1 `git mv` table — MET.** 660 staged renames (659 `R` + 1 `RM`). All 9
  stack crates landed in `src/oxidauth/` (kernel, repository, postgres,
  usecases, http, rs, permission, cli, import-export); `oxidauth-http/hurl` →
  `src/oxidauth/hurl` (18 files); `oxidauth-seed` → `src/seedz` (4 files,
  content unchanged, package name still `oxidauth-seed`). Old root dirs gone
  except `oxidauth-telemetry/`.
- **§2 stubs kept — MET.** `oxidauth-cli` + `oxidauth-import-export` present
  under `src/oxidauth/` as pure renames (manifests untouched); telemetry not
  deleted, still a member.
- **§3 root Cargo.toml — MET, one necessary deviation.** `resolver = "2"` and
  the num-bigint-dig dev-profile kept; members = `src/oxidauth/*` +
  `src/seedz` + explicit `oxidauth-telemetry` (interim until plan 06 — the
  drafted members block could not build while telemetry sat at root, so the
  deviation is right and is honestly documented). Actual `exclude` is
  `["src/oxidauth/hurl", "src/oxidauth/helm"]` — one entry more than drafted,
  and it was REQUIRED (see DEFECT D1).
- **§4 seedz path-dep audit — MET (vacuous).** `src/seedz/Cargo.toml` has an
  empty `[dependencies]` (stub), so the plan's `../oxidauth-*` repointing is
  N/A; no `../` refs remain. New finding correctly handled:
  `oxidauth-http` → `oxidauth-telemetry` dep repointed to
  `../../../oxidauth-telemetry` (path arithmetic verified: crate sits two
  levels below root). `oxidauth-telemetry/Cargo.toml` itself has zero path
  deps — nothing to repoint on its side.
- **§5 script stragglers — MET (4 scripts).**
  - `bin/hurl-tests.sh`: `pushd oxidauth-http` → `pushd src/oxidauth`.
    Semantics preserved: `hurl/` was a direct child of the old cd target and
    is a direct child of the new one, so `hurl/**/*.hurl`,
    `hurl/variables-local` and `hurl/public_keys_create.hurl` resolve to the
    same moved files. Equivalent to the plan's "update glob root" with a
    smaller diff; globstar behavior (or lack of it) is unchanged by the move.
  - `bin/reset-db.sh`: `pushd src/oxidauth/oxidauth-postgres` — sqlx
    `./migrations` stays crate-relative ✓ (live `sqlx database drop/create +
    migrate run` not re-run here; DB required).
  - `bin/publish.sh`: `FOLDER=src/oxidauth/oxidauth-$PROJECT` ✓ (active loop
    is `rs` only).
  - `bin/build-server.sh`: version probe + `-f src/oxidauth/oxidauth-http/
    Dockerfile` ✓. The moved Dockerfile needs no internal edits: it only
    `COPY tmp/$TARGETPLATFORM/oxidauth-http` (build artifact, root-relative to
    the unchanged `.` build context); `cp $CARGO_TARGET_DIR/.../oxidauth-http`
    and `cross build --bin oxidauth-http` are binary names, correctly
    untouched.
  - `docker-compose.yml` mounts the repo root at `/home/rust/src/oxidauth`
    and runs `cargo run --bin oxidauth-http` from the workspace root — still
    correct until plan 10. `dev.Dockerfile` has no crate paths.
- **§6 two-step commit — IN PROGRESS (correct so far).** 660 renames staged;
  root Cargo.toml, 4 scripts, and both plan-doc files unstaged; changelog
  untracked. Don't forget the changelog + unstaged fixes in commit (b).
- **Verification (crate set) — MET (static).** 11 members before (10 moved +
  telemetry) vs 11 after (9 glob-expanded + `src/seedz` + telemetry); names
  identical. `cargo metadata`/`cargo check` green per orchestrator.
- **PR note — PARTIAL.** `changelogs/90000002-src-layout.md` exists (untracked)
  but misstates the exclude list — see DEFECT D2.

### Status line

Current worktree state confirmed: this file says `in-progress` and the
`docs/migration-plan/README.md` tracker row 02 says `in-progress` (HEAD still
carried the pre-impl walkthrough value `reviewed`; the mid-flight flip has
been corrected by the orchestrator). Flip to `done` at close only.

### DEFECTS

- **D1 (must-fix) — the Execution note's last bullet is FALSE; rewrite it.**
  It claims "`src/oxidauth/hurl/` has no `Cargo.toml`, so the `src/oxidauth/*`
  member glob silently skips it — no `exclude` entry needed." The shipped root
  `Cargo.toml` disproves it: `exclude` lists `"src/oxidauth/hurl"` under the
  comment "(cargo globs require manifests)". The entry was necessary — cargo
  errors on a glob-matched member without a manifest (it does NOT silently
  skip); `cargo metadata` failed before the fix and only passed once the
  orchestrator added the exclude (then `cargo check` green). Correct the
  bullet to: the glob errors on manifest-less matches, so `src/oxidauth/hurl`
  IS in `exclude`. Left as-is this note will mislead plan 03, which adds
  another glob member (`src/xlib/*`).
- **D2 (must-fix) — changelog repeats the wrong exclude list.**
  `changelogs/90000002-src-layout.md` says exclude = `["src/oxidauth/helm"]`;
  the actual root manifest is `["src/oxidauth/hurl", "src/oxidauth/helm"]`.
  Fix before the changelog is committed (still untracked).

### Straggler grep (classified)

`grep -rn 'oxidauth-http/\|oxidauth-postgres/\|oxidauth-kernel/' bin/
docker-compose.yml dev.Dockerfile README.md .github/` returns exactly 2 hits,
both in `bin/build-server.sh` and both are the NEW correct
`src/oxidauth/oxidauth-http/` paths (the pattern matches their tail) —
historical-doc OK by construction, **zero live-bug hits**. Wider no-slash grep
across README.md, docker-compose.yml, dev.Dockerfile, example.env,
crypt-keeper.toml, `devops/`, `.github/`, `bin/cargo-watch.sh`: remaining hits
are binary/image names (`--bin oxidauth-http`, `cross build --bin`,
`release/oxidauth-http`, image tag), not directory paths — all correct
post-move. `rfcs/` and `docs/SECURITY_REPORT.md` keep old paths — historical
docs, OK. The straggler sweep is genuinely complete.

### Cargo.lock

Tracked and byte-identical to HEAD. That is the correct outcome: cargo records
workspace path dependencies in the lockfile by name+version only — paths are
never written there — so a pure move cannot and should not produce "path
updates"; the absence of version churn is the actual check, and it passes.

### Nits (optional)

- `bin/publish.sh`'s commented-out project list still names `seed`; after this
  move `src/oxidauth/oxidauth-seed` no longer exists, so uncommenting the loop
  would break. The comment line is untouched by this patch and plan 11
  rewrites the script — leave or fix, low stakes.
- Commit (b) checklist: root `Cargo.toml`, 4 scripts, both plan docs, and the
  still-untracked changelog.

### Verdict

**fixes-needed** — the functional change set is correct and complete (renames,
manifests, dep repointing, all four scripts; no code defects found). Two
documentation statements assert something the shipped root manifest disproves
and must be corrected before close: **2 must-fix (D1, D2), 0 should-fix,
2 nits**.

### Adjudication (worker)

- **D1 — IMPLEMENT.** Execution-note bullet rewritten; it now states cargo
  member globs ERROR on manifest-less matches (`cargo metadata` failed before
  the orchestrator's fix) and quotes the shipped exclude
  `["src/oxidauth/hurl", "src/oxidauth/helm"]`
  (docs/migration-plan/02-move-crates-into-src.md:119-123; Cargo.toml:13-17).
- **D2 — IMPLEMENT.** Changelog exclude list corrected to
  `["src/oxidauth/hurl", "src/oxidauth/helm"]` with the no-manifest rationale
  (changelogs/90000002-src-layout.md:9-12), now matching the shipped manifest.
- **N1 — SKIP.** `bin/publish.sh:3`'s commented `seed` entry left untouched:
  plan §5 assigns this script's proper rewrite to plan 11, and the active loop
  (`rs`) is unaffected; rewriting dead comment text here would pre-empt plan
  11's ownership.
- **N2 — SKIP.** Commit-(b) staging is orchestrator-owned — per worker
  contract no commits are made at all, so the checklist is noted, not acted on.
