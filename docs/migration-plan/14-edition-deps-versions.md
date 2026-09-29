# 14 — Edition 2024 + dependency/version alignment

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — uniform 0.9.0, toolchain pin, no workspace.dependencies)
**Depends on**: 02 (do last — smaller diffs while the tree still moves)
**Risk**: medium — compiler churn; publish-sensitive.

## Goal

Every crate: `edition = "2024"`, current dependency versions at template
parity, and a single workspace version story.

## Changes

1. **Edition sweep** — all `Cargo.toml`: `edition = "2021"` → `2024`
   (currently every crate is 2021; templates are 2024; repo's `rustfmt.toml`
   already pins `edition = "2024"` + `style_edition = "2024"`, so formatting
   is ahead of the crates). Fixups: `cargo fix --edition` per crate; known
   2024 gotchas here: `gen` keyword (none expected), RPIT capture rules in
   `oxidauth-api` handlers returning `impl IntoResponse` (fine — captures
   lifetime elision changes only under `use<>` syntax, none written), unsafe
   attr ordering (none).
2. **Dependency bumps** to template/generated parity:
   | dep | now | target (template/baseline) |
   |---|---|---|
   | axum | 0.8.8 | 0.8 latest (drop the explicit `http1/tower-log/…` feature pile to match baseline list `http1,json,macros,matched-path,original-uri,tower-log,query,tokio` + `default-features = false`) |
   | tower-http | 0.5.2 | 0.6.2 (cors only where needed; `fs` feature: drop — unused per grep) |
   | sqlx | 0.8.2 | 0.8.6 (`runtime-tokio`, `tls-rustls`, `macros`, `migrate`, `postgres`, `uuid`) |
   | tokio | 1.40 | 1.43 |
   | reqwest | 0.12.5 | 0.12 latest (`json`, `rustls-tls`, no default features) |
   | serde / serde_json | 1.0.203-209 | 1.0 latest |
   | uuid | 1.8-1.10 mixed | 1.x latest (one version everywhere — Cargo.lock currently double-locks uuid) |
   | chrono | 0.4.26-38 mixed | 0.4 latest |
   | async-trait | 0.1.69-0.1.89 mixed | 0.1 latest |
   | mockall | 0.14 | 0.14 (keep) |
   | gloo-storage / gloo-timers | 0.4/0.3 | latest 0.x (wasm feature) |
   | jsonwebtoken, rsa, rand, argon2, boringauth, rust_decimal, url | as-is | bump within current major; majors are a 2.0 decision, not this plan |
   Version discipline per template: pin exact minor versions **per crate
   manifest** (templates do NOT use `[workspace.dependencies]` — do not
   introduce it; consistency tool = `bin/version.sh` + review).
3. **Workspace version unification** via `bin/version.sh` (plan 11): current
   spread kernel/postgres/usecases/permission/telemetry 0.4.0, repository
   0.2.0, import-export 0.2.0 (deleted), http 0.8.0, rs 0.4.0-rc2 —
   `cargo set-version --workspace 0.9.0` (aligns `-rs`/`-http` onto the 0.9
   line the DTO split started; kernel et al. jump 0.4→0.9 — semver-wise these
   crates are 0.x anyway, breaking is allowed).
4. `rust-toolchain.toml`: **add** pinning the MSRV you build with (templates
   don't ship one; dev image is v1.98.0 — pin `1.98.0`… verify registry
   image rustc first: `docker run --rm
   registry.vizerapp.cloud/lib/rust-dev:v1.98.0 rustc -V`; pin that).
5. `[workspace.lints]` — templates have none; **do not add** (deviation
   accepted; clippy stays a check, not a manifest policy).
6. Keep: `[profile.dev.package.num-bigint-dig] opt-level = 3` (rsa perf),
   GPL-3.0 licenses, publish metadata.
7. **Published-API gate**: `cargo public-api` (or `cargo doc` diff) on
   `oxidauth` + `oxidauth-kernel` before/after — only import-path churn and
   version bumps allowed; anything else is a bug (or belongs in the 2.0 plan
   in plan 04/09 notes).

## Verification

- `cargo +<pinned> check --workspace --all-features` green;
  `cargo test --workspace --exclude oxidauth-postgres` green (via
  `bin/unit_test.sh`).
- `cargo tree --workspace -d` → **no duplicate versions** of uuid/serde/
  chrono/async-trait.
- wasm smoke: `cargo check -p oxidauth --features wasm --target
  wasm32-unknown-unknown`.
- `bin/crate_version.sh oxidauth` == `bin/crate_version.sh oxidauth-api` ==
  0.9.0.
- `cargo publish --dry-run` for every published crate.

## PR note

changelog `<id>-edition-2024-deps`; Status → `done`. **Follow-up (not this
plan):** coordinated 2.0 breaking release removing deprecated
`oxidauth_kernel::{provider::Provider, service::*, XService}` aliases +
`oxidauth_http::server::*` shim — schedule with parkinglot (they pin rev
`f358259` today).
