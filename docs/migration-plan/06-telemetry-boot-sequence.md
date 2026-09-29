# 06 — Telemetry via xlib + boot sequence

**Status**: `done` (2026-09-29: impl + review approve — boot parity exact vs template/parkinglot, deletion 2/2, fail-fast ENVIRONMENT + module-filter boot verified live)
**Depends on**: 04
**Risk**: low.

## Goal

Delete `oxidauth-telemetry` (crate currently at
`src/oxidauth/oxidauth-telemetry/`) and boot the server with the template
sequence from `/tmp/migcmp/.../tmpstack-api/src/main.rs`:

```rust
let (environment, tracing_level) = telemetry::get_logging_envs()?;
let subscriber = telemetry::get_subscriber(
    "oxidauth-api",              // crate/binary name — NOT "oxidauth-http-api"
    env!("CARGO_PKG_VERSION"),
    &environment,
    &tracing_level,
    std::io::stdout,
);
telemetry::init_subscriber(subscriber);
```

Current `main.rs` calls `oxidauth_telemetry::get_subscriber(name.into(),
"INFO".into(), stdout)` — no ENVIRONMENT/RUST_LOG support, hardcoded INFO, and
`println!("engaging oxidauth http server...")` noise.

## Changes

1. `oxidauth-http/Cargo.toml`: add
   `telemetry = { version = "0.1.0", path = "../../xlib/telemetry" }`; remove
   `oxidauth-telemetry` path dep and the now-redundant direct
   `tracing-subscriber`, `tracing-bunyan-formatter`, `tracing-log` deps
   (they live in the xlib crate).
2. `oxidauth-http/src/main.rs`: adopt the sequence above; delete the
   `println!` lines (template logs `info!("starting {crate}")` instead);
   keep `provider::init()` + bootstrap + `server.start()` flow — bootstrap
   **stays in boot permanently** (first-run provisioning, `oxidauth-usecases/src/
   bootstrap/`); plan 13's seedz is a separate local-dev-only tool and does
   not touch it.
3. `git rm -r src/oxidauth/oxidauth-telemetry`; drop from workspace members
   (covered by the `src/oxidauth/*` glob — just delete the dir).
4. Env: `ENVIRONMENT=local` + `RUST_LOG=INFO` from plan 01's example.env;
   compose passes them through (plan 10).
5. Binary-name note: name stays `oxidauth-http` **until plan 07** renames the
   crate to `oxidauth-api`; when 07 lands, the `get_subscriber` first arg
   becomes `"oxidauth-api"` in the same PR that renames. If 06 lands first,
   keep `"oxidauth-http"` as the service label to avoid log-identity churn.

## Verification

- `cargo tree -p oxidauth-http | grep -i bunyan` → only via `telemetry`.
- Boot with `RUST_LOG=debug ENVIRONMENT=local`: bunyan JSON lines, module
  filters honored (`RUST_LOG=oxidauth_http=debug` scope test).
- Missing `ENVIRONMENT` → `get_logging_envs()` errors at boot (fail-fast per
  template).
- `grep -rn oxidauth_telemetry src/` → 0.

## PR note

changelog `<id>-telemetry-xlib`; Status → `done`.

## Execution note

Executed 2026-09-29. Item-3 PATH CORRECTION: the crate actually lived at the
repo root `oxidauth-telemetry/` (plan 02's interim member), not
`src/oxidauth/oxidauth-telemetry/` as drafted; deleted with `git rm -r
oxidauth-telemetry` and dropped its explicit member line + comment block from
root `Cargo.toml` (members are now `src/xlib/*`, `src/oxidauth/*`, `src/seedz`).

**Boot diff** (`oxidauth-http/src/main.rs`, `main` keeps
`Result<(), Box<dyn Error + Send + Sync + 'static>>`; `telemetry::EnvVarError`
is `Error + Send + Sync` so `?` boxes directly, same as the template):

```diff
-    println!("engaging oxidauth http server...");
-
-    let subscriber = oxidauth_telemetry::get_subscriber(
-        "oxidauth-http-api".into(),
-        "INFO".into(),
-        std::io::stdout,
-    );
-
-    oxidauth_telemetry::init_subscriber(subscriber);
+    let (environment, tracing_level) = telemetry::get_logging_envs()?;
+
+    let subscriber = telemetry::get_subscriber(
+        "oxidauth-http",
+        env!("CARGO_PKG_VERSION"),
+        &environment,
+        &tracing_level,
+        std::io::stdout,
+    );
+
+    info!("starting oxidauth-http");
+
+    telemetry::init_subscriber(subscriber);
```

Service label is `"oxidauth-http"`, not the plan snippet's `"oxidauth-api"` —
item 5's keep-the-label rule since 06 lands before 07. `provider::init()`,
bootstrap, `PORT` and `server.start()` untouched; the pre-existing
`info!("starting server...")` / `info!("http booting...")` flow logs kept
(only the `println!` was noise per item 2).

**Removed deps** (`oxidauth-http/Cargo.toml`): `oxidauth-telemetry`,
`tracing-subscriber`, `tracing-bunyan-formatter`, `tracing-log` (grep proved
no `tracing_subscriber|tracing_bunyan_formatter|tracing_log` use in surviving
crate code; `tracing` kept). Added `telemetry = { version = "0.1.0",
path = "../../xlib/telemetry" }`.

**All `oxidauth[-_]telemetry` references found + disposition:**

- `oxidauth-telemetry/` (crate) — deleted (`git rm -r`, staged)
- root `Cargo.toml` member line + comment — removed
- `oxidauth-http/Cargo.toml` path dep — replaced by xlib `telemetry`
- `oxidauth-http/src/main.rs` (2 call sites) — rewritten to the template
  sequence; `grep -rn oxidauth_telemetry src/` → 0
- `Cargo.lock` (2 entries) — generated; the orchestrator's first cargo run
  prunes them (no cargo per assignment)
- `changelogs/90000002-src-layout.md`, `docs/migration-plan/01/02/06/15` —
  historical/plan records; left intact (01/02 prose describes the interim
  state accurately at its time)
- `README.md:51` "Dropped crates: `oxidauth-telemetry` only (→
  xlib/telemetry, plan 06)" — now true; living-doc refresh is plan 15's scope
- **zero hits** in `bin/*.sh`, `.github/`, `devops/`, Dockerfiles, compose,
  `*.yml` — nothing deferred to plan 11

**Behavior note vs plan Verification**: `ENVIRONMENT` missing is fail-fast
(`get_logging_envs` errors); `RUST_LOG` missing is NOT — xlib
`get_logging_envs` logs an error and falls back to `INFO` (template
behavior), so the Verification bullet "Missing `ENVIRONMENT` → errors at
boot" holds exactly. Changelog: `90000006-telemetry-xlib`.

## Review feedback

Verdict: **approve** — 0 must-fix · 1 should-fix · 2 nits (independent
verification 2026-09-29; the worker's report held up on every check).

**(a) Boot parity — MET, exact template sequence.** `main.rs` matches
`/tmp/migcmp/tmpdemo/src/tmpstack/tmpstack-api/src/main.rs` and
`parkinglot/src/admin/admin-api/src/main.rs` step-for-step:
`telemetry::get_logging_envs()?` → `telemetry::get_subscriber(name,
env!("CARGO_PKG_VERSION"), &environment, &tracing_level, std::io::stdout)`
(identical arg order/types and version const) → `info!("starting {name}")` →
`telemetry::init_subscriber(...)` → `provider::init().await?`. The subscriber
is installed before `provider::init`, so no provider-path logging lands on a
default subscriber, and nothing runs before `get_logging_envs` that can log.
Service label `"oxidauth-http"` per item 5 (`"oxidauth-api"` deferred to plan
07 as drafted).

- **NIT** — `info!("starting oxidauth-http")` executes *before*
  `set_global_default`, so with no global subscriber yet the event is a no-op
  and the line never appears in bunyan output: the deleted `println!` is
  replaced by silence, not by a log line. Both reference mains carry the same
  quirk (inherited template behavior), so boot code should stay as-is without
  a template decision; but the changelog bullet "the bunyan
  `info!(...)` line replaces it" overstates observable behavior — reword (the
  first emitted line is `starting server...`). Same applies to the
  `error!`/`info!` inside `get_logging_envs`' RUST_LOG fallback: emitted
  pre-init, silent in practice.
- **NIT** — both reference mains return the kernel `BoxedError` alias, which
  `oxidauth_kernel` does export (`dev_prelude`). Keeping the explicit
  `Box<dyn Error + Send + Sync + 'static>` is documented and equivalent —
  optional parity polish, e.g. folded into plan 07's rename PR.

**(b) Deletion completeness — MET.** `git ls-tree HEAD -- oxidauth-telemetry/`
= exactly 2 files (`Cargo.toml`, `src/lib.rs`); `git status` shows staged `D`
for both (2/2). Root members are now exactly `["src/xlib/*",
"src/oxidauth/*", "src/seedz"]` + the two `exclude` entries — no stale member
line or comment residue, and the deleted root-level dir cannot be re-globbed.
Independent re-grep `oxidauth[-_]telemetry` (repo-wide, gitignore disabled,
covering `.github/`, `bin/`, `devops/`, `*.Dockerfile`, `docker-compose.yml`,
all `*.yml`/`*.toml`/`*.md`): **zero** script/CI/Docker/compose/toml hits —
the worker's claim is verified. Remaining hits are history-only:
`changelogs/90000002`, this PR's own `90000006`, `docs/migration-plan/01/02/
06/15`, and `README.md:51` (statement is now true; refresh is plan 15's
scope). `Cargo.lock` now has 0 `oxidauth-telemetry` entries (pruned).

**(c) Dependency honesty — MET.** grep
`tracing_subscriber|tracing_log|bunyan|tracing_bunyan` across the whole
`src/oxidauth/oxidauth-http/` crate (manifest + sources): no matches, so the
three removals strand no code; `tracing = "0.1.40"` remains a direct dep and
is still used (`use tracing::info` in `main.rs`). `log` was never in this
manifest (HEAD listed only `tracing-log`). The `telemetry` pin
`version = "0.1.0"` matches xlib `telemetry`'s manifest. Fail-fast env
sanity: `docker-compose.yml` gives the api service `env_file: .env`
(`ENVIRONMENT=local`, `RUST_LOG=INFO`), helm staging/prod values already set
`ENVIRONMENT`, and no `bin/` script boots the server directly — the README
`docker compose up -d` quickstart still boots under the new requirement.

**(d) RUST_LOG adjudication — the worker's correction is right.**
`src/xlib/telemetry/src/lib.rs`: `var(ENVIRONMENT).map_err(...)?` is the hard
fail; `var(RUST_LOG).unwrap_or_else(...)` logs `error!` + `info!` and returns
`"INFO"` — `EnvVarError::RustLog` is only built for that (silent) log message
and never returned. Missing `RUST_LOG` = INFO fallback, not fail-fast; the
plan Verification bullet is correctly scoped to `ENVIRONMENT` only, and both
the changelog and the Execution note state the actual behavior (caveat: the
fallback's error/info are no-ops pre-subscriber, per the nit above).

- **SHOULD-FIX (commit hygiene, not code)** — the workset's staging is split:
  only the 2 deleted files are staged, while root `Cargo.toml`,
  `oxidauth-http`'s manifest + `main.rs`, and `Cargo.lock` are unstaged and
  `changelogs/90000006-telemetry-xlib.md` is untracked. A plain `git commit`
  of the staged set alone would land a commit that deletes the crate while
  the manifest still depends on it (unbuildable). Stage the full set
  (including the new changelog) before committing.

**Counts: 0 must-fix · 1 should-fix · 2 nits.**
