# OXA-000046 — `EnvVarError::RustLog` is constructed but never returned; any unreadable `RUST_LOG` is silently rewritten to "INFO"

**Original ID:** SRV-10 · **Severity:** P3 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved; Option A live — match-split w/ stderr notice on missing + Err(RustLog) abort on NotUnicode, companion applied at get_subscriber (header's xlog coordinate proven phantom), corrupt pin flipped + panic tripwire test, boot smoke both cases executed; review minors fixed same-day: helm README exact abort text + 90000006 changelog supersession note)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED — **Option A** (owner approved; corrupt-`RUST_LOG` fail-fast, parity w/ ENVIRONMENT policy). Scope: `get_logging_envs` match-split (absent → INFO + `eprintln!` notice — tracing pair is pre-subscriber no-op; `NotUnicode` → `Err(EnvVarError::RustLog)` via existing `?`); flip corrupt-value pin (missing-value pin stays green by design = intentional default); **mandatory companion:** `xlog/logger.rs:147` same match semantics (else corrupt value masks at subscriber build — fix would be cosmetic-only); helm README amendment; Option B (delete variant) explicitly rejected. Both OSes verified


## Locations

Register cites `src/xlib/telemetry/src/lib.rs:145,193` — **marker drift**: both cited lines are `BUG(pinned)` comment headers inside the test module, not the defect. The register's *claim* is accurate; the cited coordinates point at the pins that lock the behavior in. Real sites:

- `src/xlib/telemetry/src/lib.rs:53-68` — `get_logging_envs()`: the swallow. `var(RUST_LOG).unwrap_or_else(|err| { error!("no valid tracing level provided: {:?}", EnvVarError::RustLog(err)); info!("falling back to INFO"); "INFO".into() })` (`:56-65`), then unconditional `Ok((environment, tracing_level))` (`:67`).
- `:73` — `EnvVarError::RustLog(VarError)` variant declaration; `:80` Display arm; `:91` `Error::source` arm. The `:59` construction is the *only* production use of the variant, and it exists solely as a `Debug` format argument for a log line that never escapes (see Problem).
- `:27` — `get_subscriber`'s independent `RUST_LOG` read: `EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter))` (adjacent behavior, see Analysis).
- Sole callers, both `?`-propagating: `src/oxidauth/oxidauth-api/src/main.rs:14` and `src/seedz/src/main.rs:25`; each installs the subscriber afterwards (`main.rs:24` / `main.rs:35`).
- Pinned tests: `missing_rust_log_falls_back_to_info_instead_of_erroring` `:143-156` (marker `:145-147`, the `:145` register cite), `non_unicode_rust_log_also_falls_back_to_info_silently` `:190-209` (companion marker `:193-195`, the `:193` register cite, `#[cfg(unix)]`). Supporting pins: `environment_is_resolved_before_rust_log` `:158-169`, `non_unicode_environment_surfaces_as_not_unicode_kind` `:171-188`, `display_names_the_var_and_inner_kind_for_both_variants` `:211-221`, `source_returns_the_wrapped_var_error` `:223-237` (both construct `RustLog` directly, keeping the variant compiler-live).
- Deployment context: `oxidauth/docker-compose.yml:55` (`RUST_LOG: ${RUST_LOG:-INFO}`), `oxidauth/helm/values.yaml:54` (`RUST_LOG: "info"`), `oxidauth/helm/README.md:62` (documents `ENVIRONMENT`/`RUST_LOG` as "standard"). Crate header `:1`: vendored from the freshbrewlabs project-template @ c38ec0a, "re-diff when templates change".

## Problem

`EnvVarError::RustLog` is a public error variant that is unreachable from every production call path. `get_logging_envs()` returns `Err` for exactly one condition — `ENVIRONMENT` unreadable (`:54`) — and maps **any** `VarError` from `RUST_LOG` (`NotPresent` *or* `NotUnicode`) onto the literal `"INFO"` (`:56-65`), returning `Ok`. Neither binary (`oxidauth-api`, `seedz`) can ever observe `Err(EnvVarError::RustLog(..))` through its `?`; the variant's Display/source machinery (`:80`, `:91`) exists only to format the discarded `error!` argument and satisfy the two direct-construction tests.

The silence is doubled. The `error!`/`info!` pair inside the closure (`:57-62`) fires **before** any subscriber exists — both callers run `get_logging_envs()` at boot and only then call `init_subscriber` (`oxidauth-api main.rs:14` vs `:24`; `seedz` `:25` vs `:35`), and `LogTracer::init()` happens inside `init_subscriber` too (`lib.rs:49`) — so with tracing's default (no global subscriber ⇒ events discarded) dispatcher, even the "falling back to INFO" notice itself is dropped. Nothing, anywhere, records that `RUST_LOG` was read-and-rejected. Both test markers state this in comments (`:145-147`, `:193-195`); the two pin tests then assert the fallback as expected behavior (`:152-155`, `:203-206`).

## Analysis

**The two conflated states deserve different handling.** `VarError::NotPresent` is the ecosystem-normal case: the whole `tracing-subscriber` stack treats `RUST_LOG` as optional (`EnvFilter::try_from_default_env` returns `Result`, `DEFAULT_ENV` docs at `filter/env/mod.rs:244-249` of tracing-subscriber 0.3.23), and the shipped deploy configs effectively pre-default it anyway (`docker-compose.yml:55` `${RUST_LOG:-INFO}`, `helm/values.yaml:54`). Erroring at boot on an unset `RUST_LOG` would be hostile and would break every bare `cargo run`. **`NotUnicode` is not normal** — it means a corrupt env block (producible via `execve` from non-shell parents; the test itself injects `[0xff, 0xfe]`, `:201`) — and today it gets silently rewritten to a *different configuration value than the operator set*, with zero output.

**The function is internally inconsistent about `NotUnicode`.** For `ENVIRONMENT`, non-UTF-8 is fatal — `map_err(EnvVarError::Environment)?` (`:54`), pinned by `non_unicode_environment_surfaces_as_not_unicode_kind` (`:171-188`). For `RUST_LOG`, the identical error kind from the identical `var()` call on the adjacent line is swallowed. Same function, same corruption class, opposite policy — arbitrary, not designed.

**Adjacent, deliberately not re-ticketed here:** a *malformed but valid-UTF-8* `RUST_LOG` (e.g. `RUST_LOG=debugg`) passes through `get_logging_envs` untouched (it only reads, never parses), and in `get_subscriber:27` `try_from_default_env()` rejects it; the fallback `EnvFilter::new(env_filter)` (env_filter = the same malformed string) is tracing-subscriber 0.3.23 `with_default_directive(ERROR).parse_lossy(...)` (`filter/env/mod.rs:350-354`, `builder.rs:146-158`): bad directives are dropped with an `eprintln!("ignoring ...")` and the level collapses to **ERROR**, not INFO. So the realistic "typo in RUST_LOG" outcome is ERROR-level logging with only a bare stderr line as a hint — a third silent-degradation mode, distinct from this register line (which covers missing/non-UTF-8), and it also shows the `env_filter` parameter of `get_subscriber` is vestigial in both current binaries (it always equals `RUST_LOG` verbatim or `"INFO"`, and `unwrap_or` evaluates it eagerly on every boot regardless).

**Why the variant stays compile-clean:** constructed at `:59` (dead end), Display-matched at `:80`, source-matched at `:91`, directly built in tests at `:218` and `:227`. No `dead_code` lint can fire; only the pins document that the `Err` path is unreachable. Vendoring (`:1`) means upstream almost certainly carries the same defect — any local fix becomes a permanent re-diff delta; keep the hunk small.

**Likelihood/blast radius of `NotUnicode`:** very low (malformed env blocks are exotic), which is exactly why P3 is proportionate: no security exposure, no data corruption, no realistic boot break — the observable effect is "log level is INFO and no artifact says why".

## Impact

Who is affected: developers booting `oxidauth-api`/`seedz` outside compose/helm (the fallback is their real UX — and it is *acceptable* UX); operators of hosts with a corrupt env block, whose `RUST_LOG` is silently replaced by `INFO` with no log, no stderr, no error — precisely in the debugging scenario where log level matters. The misleading part is contractual: `get_logging_envs() -> Result<_, EnvVarError>` and the exported `RustLog` variant advertise an error capability the function never exercises, and the in-code `error!` message implies a report that is structurally impossible to see.

## Proposed resolution

**Option A (recommended): wire the variant; keep missing → INFO as documented behavior.** In `get_logging_envs`, replace the `unwrap_or_else` closure (`:56-65`) with a match so the two `VarError` kinds split, and make the fallback notice actually visible (`eprintln!`, since no subscriber exists yet — the current `tracing` pair is a runtime no-op):

```rust
let tracing_level = match var(RUST_LOG) {
    Ok(level) => level,
    // Missing RUST_LOG is the supported default path (compose/helm pre-set it):
    // boot at INFO, but say so on stderr.
    Err(VarError::NotPresent) => {
        eprintln!("RUST_LOG not set; defaulting to INFO");
        "INFO".into()
    },
    // Corrupt value (NotUnicode): same policy as ENVIRONMENT — fail fast.
    Err(err) => return Err(EnvVarError::RustLog(err)),
};
```

Enum, `Display`, and `Error` impls stay untouched; `Err(RustLog(..))` propagates via the existing `?` in both `main`s and aborts boot with the `:83` message. Pin flips:

1. `non_unicode_rust_log_also_falls_back_to_info_silently` (`:190-209`): **flip** — rename to e.g. `non_unicode_rust_log_is_returned_as_rust_log_variant`, replace the `assert_eq!(..unwrap(), ("staging", "INFO"))` (`:203-206`) with `match .. { Err(EnvVarError::RustLog(VarError::NotUnicode(_))) => {}, other => panic!(..) }`; delete marker `:193-195`; keep `ENVIRONMENT="staging"` + `remove_env(RUST_LOG)` cleanup.
2. `missing_rust_log_falls_back_to_info_instead_of_erroring` (`:143-156`): **stays green by design** (that is the point of Option A) — but rewrite marker `:145-147` (its "only ever built for a log line … NEVER returned" claim becomes false); the test now pins the *intentional* default, not a bug.
3. `environment_is_resolved_before_rust_log` (`:158-169`): stays green — with both vars absent, `ENVIRONMENT` still short-circuits first (`:54`), and `RustLog(NotPresent)` remains non-fatal; the comment at `:164` ("no RustLog variant surfaces") remains true for this input but should be reworded (it's now policy, not deadness).
4. `display_names_the_var_and_inner_kind_for_both_variants` (`:211-221`) and `source_returns_the_wrapped_var_error` (`:223-237`): stay green; they now pin a reachable variant.
5. Docs: amend `oxidauth/helm/README.md:62` — `RUST_LOG` is optional (absent ⇒ INFO); a non-UTF-8 value aborts boot.

**Option B (alternative, minimal churn): delete the dead variant.** Narrow `EnvVarError` to `Environment(VarError)`; simplify `:56-65` to an untyped `var(RUST_LOG).unwrap_or_else(|_| { eprintln!("RUST_LOG not set or unreadable; defaulting to INFO"); "INFO".into() })`; delete Display arm `:80`, source arm `:91`, the `RustLog` cases in the `:211-221`/`:223-237` tests, and rewrite both `BUG(pinned)` markers as doc comments recording the fallback as intended design. `EnvVarError` is `pub` but the crate is workspace-internal (path-deps only: `oxidauth-api/Cargo.toml:18`, `seedz/Cargo.toml:20`), so removal is compile-verified with no external-consumer breakage. Choose B if the team decides corrupt-value tolerance is the product; A is recommended because it makes corruption visible and restores symmetry with `ENVIRONMENT`. Under either option: replace the `error!`/`info!` pair with `eprintln!` (or move the notice after `init_subscriber`) — otherwise the "silent" half of this bug survives the fix. Do not re-scope the malformed-`RUST_LOG` → ERROR-level `parse_lossy` degradation documented above into this ticket.

## Verification

- `cargo test -p telemetry` — 8 tests on this darwin host (both `#[cfg(unix)]` tests compile and run); after Option A: the flipped `non_unicode_*` test passes on `Err(RustLog(NotUnicode(_)))`, and `missing_rust_log_*`, `environment_is_resolved_before_rust_log`, `returns_the_environment_and_rust_log_pair`, `non_unicode_environment_*`, `display_*`, `source_*` remain green. Option B: same command after deleting the `RustLog` pins; add a permanent assertion that `remove_env(RUST_LOG)` still yields `Ok(("production", "INFO"))` (already carried by the retained missing-case test).
- `cargo build -p oxidauth-api -p seedz` — compile check (Option B must fix nothing in the mains — `?` coercion to `Box<dyn Error>` holds for the single-variant enum; Option A changes no signature).
- Boot smoke, missing case: `env -u RUST_LOG ENVIRONMENT=local cargo run -p oxidauth-api` — stderr shows `RUST_LOG not set; defaulting to INFO` **before** the later boot output/failure, proving the notice now escapes (today it produces nothing).
- Boot smoke, corrupt case (Option A flip): shells cannot pass non-UTF-8 env; use a launcher, e.g. Python `subprocess.run([...], env={..., "RUST_LOG": os.fsdecode(b"\xff\xfe")})` (surrogateescape round-trips the raw bytes through `execve`) — `oxidauth-api` must exit non-zero with `env var RUST_LOG not found: NotUnicode(...)` (the `:83` Display text); `[recipe not executed here]`.
