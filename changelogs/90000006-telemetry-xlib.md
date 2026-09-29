- [90000006](https://www.pivotaltracker.com/story/show/90000006) - oxidauth-http boots on the xlib `telemetry` crate; `oxidauth-telemetry` deleted
    - new path dep `telemetry = { version = "0.1.0", path = "../../xlib/telemetry" }`
      in `oxidauth-http`; the boot sequence is now the template's:
      `telemetry::get_logging_envs()?` → `telemetry::get_subscriber(name, version,
      environment, env_filter, stdout)` (service label stays `oxidauth-http`
      until plan 07 renames the crate to `oxidauth-api`) →
      `info!("starting oxidauth-http")` → `telemetry::init_subscriber(...)`
    - removed direct deps `oxidauth-telemetry`, `tracing-subscriber`,
      `tracing-bunyan-formatter` and `tracing-log` from `oxidauth-http` — they
      live in xlib `telemetry` now, and no surviving code in the crate named
      them (`tracing` itself is kept; `info!` is used)
    - deleted the `println!("engaging oxidauth http server...")` boot noise;
      the bunyan `info!("starting oxidauth-http")` line replaces it
    - deleted the root `oxidauth-telemetry/` crate (`git rm -r`) and its
      explicit workspace-member line; no other crate, script, Dockerfile, or
      CI file referenced it
    - BEHAVIOR CHANGE: `ENVIRONMENT` is now REQUIRED at boot — a missing
      `ENVIRONMENT` env var is a hard fail-fast
      (`telemetry::EnvVarError::Environment`) before any provider work, where
      the old crate needed no env and hardcoded level `INFO`. `RUST_LOG` now
      drives the `EnvFilter`, so module filters such as
      `RUST_LOG=oxidauth_http=debug` are honored; when unset it logs an error
      and falls back to `INFO` (template behavior), i.e. level `INFO` is no
      longer hardcoded but the process does not die on a missing `RUST_LOG`
