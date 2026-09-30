- [OXA-000046](https://www.pivotaltracker.com/story/show/OXA-000046) - wire `EnvVarError::RustLog`; corrupt `RUST_LOG` now fails fast at boot (register SRV-10, Option A)
    - **Behavior change (boot):** a non-UTF-8 `RUST_LOG` value (corrupt env
      block) previously booted silently at `INFO` with no trace of the
      rejection; `oxidauth-api` and `seedz` now abort at
      `telemetry::get_logging_envs()?` and exit non-zero — the terminal
      prints the boxed error's `Debug` form (`Error: RustLog(NotUnicode(..))`);
      the error's `Display`/`source` text is
      `env var RUST_LOG not found: NotUnicode(...)` — parity with the
      existing `ENVIRONMENT` policy.
    - **Behavior change (stderr):** an unset `RUST_LOG` still boots at
      `INFO` (unchanged supported default — compose/helm pre-set it), but
      the fallback is now announced on stderr (`RUST_LOG not set; defaulting
      to INFO`). The old in-crate `error!`/`info!` pair fired before any
      subscriber existed and was structurally invisible; it is replaced by
      an `eprintln!` that actually escapes.
    - **Companion:** `telemetry::get_subscriber` no longer masks a corrupt
      `RUST_LOG` at subscriber build — its independent `EnvFilter` read
      swallowed `NotUnicode` into the caller's default filter; it now fails
      fast (panics with the same message) on parity with the boot abort.
      Missing/valid values keep byte-identical prior behavior; a malformed
      but valid-UTF-8 value (e.g. `RUST_LOG=debugg`) is unchanged — that
      silent level-collapse is a separate register line.
    - `EnvVarError` enum, `Display`, and `Error` impls unchanged; the
      `RustLog` variant is now reachable from both binaries via their
      existing `?` instead of only being constructed for a dead log line.
    - Tests: corrupt-`RUST_LOG` pin flipped to assert
      `Err(EnvVarError::RustLog(NotUnicode))`; missing-`RUST_LOG` pin stays
      green, reworded from `BUG(pinned)` to intentional-default docs; new
      `#[cfg(unix)]` pin asserts `get_subscriber` panics on a corrupt value.
    - Docs: `src/oxidauth/helm/README.md` env table now states `RUST_LOG` is
      optional (unset ⇒ `INFO`) and that a non-UTF-8 value aborts boot.
