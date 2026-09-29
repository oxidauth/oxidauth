# 11 — Scripts + hurl/tests conventions

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — incl. script deletions)
**Depends on**: 02, 07
**Risk**: low.

## Goal

Adopt the `bin/` discovery-script system: root scripts find per-stack scripts;
test suites follow the unit / database / hurl triad.
References: `/tmp/migcmp/tmpdemo/bin/*.sh`, parkinglot extras
(`version.sh`, `crate_version.sh`, `e2e.sh`).

## Changes — root `bin/`

| new | source | content |
|---|---|---|
| `bin/build.sh` | template | verbatim pattern; `REGISTRY="registry.vizerapp.cloud/oxidauth"`, `RUST_BASE_IMAGE_VERSION="v1.89.0"`, `DEBIAN_BASE_IMAGE_VERSION="12.12"`; `find ./src -name build.sh` exec loop |
| `bin/unit_test.sh` | template | `cargo test --workspace --exclude oxidauth-postgres` (the `*-postgres` exclude in template = DB-test isolation) |
| `bin/database_test.sh` | template | `find ./src -name database_test.sh` exec loop → runs `src/oxidauth/database_test.sh` |
| `bin/hurl.sh` | template | `find ./src -name hurl.sh` exec loop → runs `src/oxidauth/hurl.sh` |
| `bin/deploy.sh`, `bin/uninstall.sh` | template | arrive complete in plan 12 |
| `bin/version.sh` | parkinglot | `cargo set-version --workspace` wrapper (show/dry-run/apply) — enables plan 14's version unification |
| `bin/crate_version.sh` | parkinglot | print one crate's version from `cargo metadata` — used by build scripts for image tags |

**Per-stack scripts** (new files):
- `src/oxidauth/hurl.sh` — port of today's `bin/hurl-tests.sh`:
  `hurl --test --glob "hurl/tests/*.hurl" --variables-file hurl/variables-local`
  (run twice per the existing cleanup-verification trick; keep both passes).
- `src/oxidauth/database_test.sh` — `DATABASE_URL=*** cargo test -p
  oxidauth-postgres -- --nocapture` (env from `.env`; honors
  `MIGRATIONS_ENABLED=true` + `MIGRATOR` tests from plan 05).
- `src/oxidauth/oxidauth-api/build/build.sh` + `build/Dockerfile` — created in
  plan 12 (image build); listed here so plan 11's `bin/build.sh` discovery is
  understood to pick it up once 12 lands.

## Changes — deletions/ports

- delete `bin/hurl-tests.sh` (→ hurl.sh pair), `bin/build-server.sh`
  (superseded: cross-build + `docker buildx` multi-arch logic moves into
  `oxidauth-api/build/build.sh` in plan 12 — preserve its buildx builder-name
  pattern, parkinglot's `build.sh` is the model), `bin/cargo-watch.sh`
  (fold into `database_test.sh` + `docker compose up` — or keep as personal
  convenience; default: delete, dev loop is compose watchexec now).
- `bin/publish.sh` — update crate list/paths for the new tree
  (`src/oxidauth/*`); publish order (topological): kernel → permission →
  repository → postgres → services → http → rs → api. Keep its semver/git-tag
  step; source version via `bin/crate_version.sh`.

## hurl relocation (finishes plan 02's move)

- `src/oxidauth/hurl/{variables-local, setup_user.hurl,
  public_keys_create.hurl, tests/*.hurl}` already moved in 02; update
  `variables-local` if base URL/port changed by plan 10 (default host stays
  `api.oxidauth.localhost`/localhost:port — align after 10 verification).
- Add `src/oxidauth/hurl/healthcheck.hurl` asserting the new
  `/api/v1/__meta/healthcheck` payload `{success, payload:{version,healthy}, …}`
  (plan 07 contract).

## Verification

- `bin/unit_test.sh` green with zero DB.
- `bin/database_test.sh` green against compose postgres.
- `bin/hurl.sh` green (full suite, both passes, cleanup state verified like
  today).
- `bin/build.sh` with `DRY=1`… not supported by template — instead: `bash -n`
  each script + run `bin/build.sh` up to the first `docker buildx` step in CI
  dry context, or stub `REGISTRY=localhost:5000` + one build, push disabled.
- `bin/version.sh show` prints the current (still divergent) version set —
  evidence it reads the new workspace.

## PR note

changelog `<id>-bin-scripts`; Status → `done`.
