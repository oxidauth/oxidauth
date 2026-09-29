# 03 — Vendor xlib crates

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — verbatim vendoring, FromRef/oxidauth-dep stripped from provider copy)
**Depends on**: 02
**Risk**: low — additive; nothing consumes xlib yet.

## Goal

Add the four shared crates every new stack assumes:
`src/xlib/{http,postgres,provider,telemetry}`, copied from
`project-template/src/xlib/*` (verified in generated baseline
`/tmp/migcmp/tmpdemo/src/xlib/`).

## Changes

1. `cp -R ~/dev/freshbrewlabs/project-template/src/xlib src/` (then re-add to
   git). Copy `src/seedz` template files **contents later** (plan 13); keep
   only the moved stub for now.
2. Manifests stay `version = "0.1.0"`, `edition = "2024"`, path deps —
   exactly as generated (e.g. `tmpstack-api` refs
   `provider = { version = "0.1.0", path = "../../xlib/provider" }`).
3. **`src/xlib/provider/src/lib.rs` — strip the oxidauth coupling.** The
   template crates `oxidauth = { git = …, rev = "f358259" }` and add
   `impl FromRef<Provider> for OxidAuthClient`. Inside THIS repo that is a
   self-dependency loop (the client will consume these xlibs transitively)
   and pins a stale rev. In this vendored copy:
   - remove the `oxidauth` dependency + the `FromRef` impl;
   - keep `Provider { store/fetch/fetch_unchecked/take }` + `ProviderError`
     verbatim so behavior is byte-identical for consumers;
   - add a doc-comment: *"consumer projects re-add `impl FromRef<Provider> for
     oxidauth::OxidAuthClient` in their own tree (see parkinglot)"* — the impl
     is 5 lines and belongs to whoever owns both types (orphan-rule friendly
     since both are foreign here).
   - Note `take()` exists (template) — the kernel Provider lacks it; adopt the
     xlib version everywhere server-side (plan 04).
4. **`src/xlib/postgres`** keep verbatim: `database!` macro
   (write+read `PgPool`, `from_env`, `from_database_url` with read falling
   back to write URL, `migrate()` gated on `$MIGRATIONS_ENABLED == "true"`,
   `read_pool()/write_pool()`, `PingTrait` pinging both pools,
   `DatabaseBuilder`, `PgError`, `mock` module), `ping/mod.rs` + `ping.sql`,
   `migrator_const!`/`migrator!` helper macros. sqlx 0.8.6.
5. **`src/xlib/http`** keep verbatim: `Response<P>` (fields `success`,
   `payload`, `errors`, `warnings`, `notices`, `status_code`;
   `IntoResponse` — same envelope oxidauth's `response.rs` already speaks),
   `data.rs` state enums (`LoadingState`, `SaveState`, `DeleteState`,
   `CreateState`) — unused server-side today, needed when a consumer builds UI
   against these DTOs; keep anyway.
6. **`src/xlib/telemetry`** keep verbatim: `get_logging_envs()` (reads
   `ENVIRONMENT`, `RUST_LOG`), `get_subscriber(name, version, environment,
   env_filter, sink)` Bunyan-formatted, `init_subscriber`.
7. **Root `Cargo.toml`** — add `"src/xlib/*"` to members.
8. Vendoring policy: add `docs/migration-plan/VENDORED.md`? No — one line in
   each vendored crate's `lib.rs` header doc: `//! Vendored from freshbrewlabs
   project-template @ <git rev>; re-diff when templates change.` (comment only).

## Verification

- `cargo check --workspace` green (xlibs compile standalone; xlib/provider
  now has zero oxidauth coupling: `cargo tree -p provider` shows no
  `oxidauth` entries).
- `cargo test -p provider -p postgres -p http -p telemetry` green
  (xlib/provider ships its own unit tests).
- Nothing else depends on xlib yet — no consumer churn in this PR.

## PR note

changelog `<id>-vendor-xlib`; Status → `done`.
