# 02 — Move crates into src/ layout

**Status**: `reviewed` (walkthrough 2026-09-29: approved with tweak — cli/import-export KEPT, moved under src/oxidauth/)
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
