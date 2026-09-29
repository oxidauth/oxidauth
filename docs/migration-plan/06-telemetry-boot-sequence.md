# 06 — Telemetry via xlib + boot sequence

**Status**: `in-progress` (worker dispatched 2026-09-29)
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
