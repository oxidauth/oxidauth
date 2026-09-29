# 03 — Vendor xlib crates

**Status**: `done` (2026-09-29: impl + review approve (S1/N2/N3 fixed, N1 recorded for plan 11); workspace tests green, committed with fix 90000000)
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

## Execution note

Executed 2026-09-29. `cp -R` from `project-template/src/xlib` → `src/xlib`:
**11 files**, verified byte-identical to template except the deviations below.

Copied files:

- `http/Cargo.toml`, `http/src/data.rs`, `http/src/lib.rs`
- `postgres/Cargo.toml`, `postgres/src/lib.rs`, `postgres/src/ping/mod.rs`,
  `postgres/src/ping/ping.sql`
- `provider/Cargo.toml`, `provider/src/lib.rs`
- `telemetry/Cargo.toml`, `telemetry/src/lib.rs`

Header line added to all four `src/lib.rs` (line 1, per §8; rev spot-checked
with `git -C project-template rev-parse --short HEAD` → `c38ec0a`):
`//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.`

Provider deviations (the only content edits):

`provider/Cargo.toml` — removed dep line:

    -oxidauth = { git = "https://github.com/oxidauth/oxidauth.git", version = "0.2.0", rev = "f358259" }

`provider/src/lib.rs` — removed import + impl (`serde` dep kept as-is):

    -use oxidauth::{OxidAuthClient, axum::extract::FromRef};
    -
    -impl FromRef<Provider> for OxidAuthClient {
    -    fn from_ref(provider: &Provider) -> Self {
    -        provider
    -            .fetch_unchecked::<OxidAuthClient>()
    -            .clone()
    -    }
    -}

In place of the impl (plan §3 doc-comment form, per review N3):

    /// consumer projects re-add `impl FromRef<Provider> for oxidauth::OxidAuthClient`
    /// in their own tree (see parkinglot) — the impl is 5 lines and belongs to
    /// whoever owns both types (orphan-rule friendly since both are foreign here).

Provider tests module: all three tests (`it_should_return_fetch`,
`test_provider_take`, `it_should_return_an_error`) reference only
`Provider`/`ProviderError` — **zero oxidauth references, nothing removed**.
`grep -rn oxidauth src/xlib/` matches exactly **one** line (count = 1), the
head of the consumer doc-comment:

    provider/src/lib.rs:50:/// consumer projects re-add `impl FromRef<Provider> for oxidauth::OxidAuthClient`

This is so because the header lines do not contain the string "oxidauth" and
the old `use` line was deleted, not commented; workspace gained
`members = ["src/xlib/*", ...]`, exclude untouched. Cargo.lock **is** modified
(lock refreshed by the orchestrator's cargo gate after the first draft of this
note): `git diff --stat` → +49/−13, i.e. four new path-crate stanzas
(`http`/`postgres`/`provider`/`telemetry` @ 0.1.0) plus 13 mechanical
`"http"` → `"http 1.4.0"` dependency-reference disambiguations; zero other
version churn, zero new/removed registry packages.

N1 disposition (http package-name CLI ambiguity): **no rename** — template
parity wins. Plan-11 scripts MUST use exact specs (`http@0.1.0` /
`http@1.4.0`) for full-graph commands (`cargo tree`/`cargo update -p http`)
or adopt the template's `--exclude`-based `unit_test.sh` form;
member-scoped `cargo test -p http` stays unambiguous; lockfile tooling keys
by (name, version).

## Review feedback

Reviewed 2026-09-29 against the template at
`/Users/georgewheeler/dev/freshbrewlabs/project-template` (checkout rev `c38ec0a`).
Static review only; cargo not run (`cargo check`/test gates deferred to orchestrator).

### Checklist (per numbered `## Changes` bullet)

1. **`cp -R` src/xlib, 11 files** — MET. `diff -r <template>/src/xlib src/xlib` → no `Only in`
   entries; both trees hold exactly 11 files. Diff output is 33 lines total, 19 changed-content
   marker lines, across 5 file pairs — exactly: 4× `0a1` header-line additions (all four
   `src/lib.rs`), `provider/Cargo.toml 7,8c7` (oxidauth dep line removed), `provider/src/lib.rs
   9,10d9` (import removed) + `51,57c50` (FromRef impl → consumer comment). No other deltas.
2. **Manifests stay 0.1.0 / edition 2024** — MET. All four `Cargo.toml` carry
   `version = "0.1.0"`, `edition = "2024"`; byte-identical to template except the §3 provider dep
   removal. No inter-xlib path deps exist in the template set (§2's path-dep example is
   consumer-side and is preserved by the verbatim copy).
3. **provider oxidauth strip** — MET. The three provider hunks above are exactly the sanctioned
   removals + replacement comment; `Provider` (`store`/`fetch`/`fetch_unchecked`/`take`),
   `ProviderError`, and all 3 tests verbatim. FromRef-strip rationale confirmed satisfied:
   `grep -rn oxidauth src/xlib` → single hit, the consumer comment at
   `provider/src/lib.rs:50`; lock stanza shows `provider 0.1.0` deps = `["serde"]` only, so
   `cargo tree -p provider` cannot show oxidauth edges. `take()` present. (Nits N3; serde dep
   kept unused-in-crate per "kept verbatim" intent.)
4. **postgres verbatim** — MET. Diff shows only the header line; `sqlx = { version = "0.8.6",
   features = ["postgres"] }` unchanged.
5. **http verbatim** — MET. Diff shows only the header line (`Response<P>`, `data.rs` intact).
6. **telemetry verbatim** — MET. Diff shows only the header line.
7. **Root Cargo.toml** — MET. Diff is the single line `+    "src/xlib/*"` (first member; glob
   ordering is semantics-neutral); `exclude` untouched (no other diff lines); all four crates
   ship manifests so the glob resolves.
8. **Vendored header lines** — PARTIAL. Header present at line 1 of all four `lib.rs`, but text
   uses `(2026-09)` where the plan specified `@ <git rev>`; the template checkout is a git repo
   (`git -C … rev-parse --short HEAD` → `c38ec0a`), so the rev was obtainable at copy time.
   Comment-only impact (N2).

### Defects / observations

- **Cargo.lock churn — CLEAN.** `git diff Cargo.lock`: +49/−13, consisting solely of the four
  new path-crate stanzas (`http`/`postgres`/`provider`/`telemetry` @ 0.1.0) and 13 mechanical
  `"http"` → `"http 1.4.0"` dependency-reference disambiguations caused by the collision in N1.
  Zero other `+/-name`/`+/-version` lines; no registry packages added or removed; no version
  bumps anywhere.
- **N1 (nit) — `http` package-name collision on the cargo CLI.** The lock now holds two `http`
  packages: `http@0.1.0` (path, vendored) and `http@1.4.0` (crates.io, deep axum/hyper/
  tower-http dep). Full-graph package selection with a bare `-p http` — e.g. `cargo tree -p
  http`, `cargo update -p http` — fails with "multiple `http` packages … ambiguous";
  workspace-member-scoped commands (`cargo test -p http`) stay unambiguous. Recommend scripts/CI
  use exact specs (`http@0.1.0` / `http@1.4.0`) and any lockfile tooling key by (name, version).
  `postgres`/`telemetry`/`provider` have no registry twins in the graph today; same latent risk
  if e.g. crates.io `postgres` is ever added. Names are fixed by the verbatim-vendoring intent,
  so this is a nit, not a defect.
- **N2 (nit) — header lacks the planned git rev** (Checklist 8): `@ c38ec0a` was available;
  consider folding it into the four header lines on the next touch of these files.
- **N3 (nit) — consumer comment deviates from §3 wording/format:** plain `//` (invisible to
  rustdoc) instead of a doc comment, and it drops "in their own tree" plus the "impl is 5 lines
  … orphan-rule" rationale. Functionally complete.
- **S1 (should-fix, docs) — two stale claims in the Execution note (part of this change set):**
  (i) "no Cargo.lock changes (cargo deliberately not run)" — Cargo.lock **is** modified in the
  working tree (refreshed after the note was written; content itself is clean per above);
  (ii) "`grep -rni oxidauth src/xlib/` now matches only the two comment lines" — it matches
  exactly **one** line (`provider/src/lib.rs:50`), since the `use oxidauth…` line was removed
  rather than commented. Update the note before opening the PR so it matches the diff.

### Rating

**Must-fix: 0 · Should-fix: 1 (S1, plan-file wording only) · Nit: 3 (N1–N3).**
Verdict: **approve** — vendoring is byte-faithful to the template except the sanctioned provider
decoupling; lock additions are minimal and pull **no new dependency majors** (axum `0.8.7`→0.8.8,
sqlx `0.8.6`→0.8.6, tracing-log `0.2.0`, async-trait `0.1.89`, uuid `1.10`→1.19.0, plus
serde/serde_json/tracing/tracing-subscriber/tracing-bunyan-formatter — all already present; zero
new registry `[[package]]` stanzas).

### Adjudication (worker)

2026-09-29, post-review:

- **S1 — implemented.** Execution note rewritten in place: lock claim now
  quotes the real churn (+49/−13 = 4 path stanzas + 13 `"http"` → `"http 1.4.0"`
  disambiguations, zero other version churn) and the grep claim quotes the
  live single-line output (`provider/src/lib.rs:50`, count = 1).
- **N2 — implemented.** Rev self-verified: `git -C ~/dev/freshbrewlabs/project-template
  rev-parse --short HEAD` → `c38ec0a` (matches review). All four `src/lib.rs`
  line 1 now read `//! Vendored from freshbrewlabs project-template @ c38ec0a
  (2026-09); re-diff when templates change.` (verified via `head -1`); the
  changelog quote was updated to match.
- **N3 — implemented.** Consumer comment converted to the plan §3 `///`
  doc-comment verbatim ("in their own tree", 5-line impl + orphan-rule
  rationale included). Caveat recorded: a mid-file `///` attaches to the
  following item, so rustdoc will surface it on `ProviderError`; that is the
  only position a `///` can occupy outside a file head, and it is inert for
  compilation (single-backtick span → no doctest).
- **N1 — disposition: no code change.** Template parity wins; recorded in the
  Execution note that plan-11 scripts use exact specs (`http@0.1.0`) or the
  template `--exclude`-based `unit_test.sh` form.
