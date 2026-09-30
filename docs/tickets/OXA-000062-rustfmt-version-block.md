# OXA-000062 — rustfmt required_version pin freezes the fmt lane

**Original ID:** T-2 · **Severity:** n/a · **Type:** tooling · **Status:** implemented 2026-09-30 (coder+reviewer approved; canonical fmt lane nightly-2026-09-27 / rustfmt 1.11.0, pin `>=1.11.0`; stable-leniency residual documented — see changelog; workspace `cargo test --no-run` gate owned by Main)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved; scope = pick canonical dated nightly, MEASURE `fmt --check` diff first (expected casualties: `make_backup`, `brace_style` deprecated keys), bump `required_version`, prune, ONE reformat-only commit, toolchain header + README in same commit; register strike moot (BUGS_AND_NOTES retired) → changelog; **sequencing mandate: lands FIRST, before any scheduled-fix branch diverges** — reformat-first else per-crate sweeps at merge breaks, never interleave fmt hunks with fix commits; stable `cargo fmt` erroring stays by-design)


## Locations

- `rustfmt.toml` (repo root) — `required_version = "1.8.0"` (line 3), `unstable_features = true` (line 4), `style_edition = "2024"` (line 2), and ~20 nightly-only options incl. `imports_granularity = "Crate"` (:48), `group_imports = "StdExternalCrate"` (:56), `blank_lines_upper_bound` (:11), `wrap_comments = true` (:76), `make_backup = false` (:77)
- `rust-toolchain.toml:1-5` — channel `1.98.0`; header comment states formatting is `cargo +nightly-2025-07-01 fmt --all` and claims "every CI lane agree[s]" on the toolchain number
- `README.md:130-132` — the fmt convention: "the fmt lane is a pinned nightly, NOT plain `cargo fmt`"
- `BUGS_AND_NOTES.md:97` — register line T-2
- `.github/` — contains **only** `PULL_REQUEST_TEMPLATE.md`; no workflows directory, no CI config anywhere (also `.gitlab-ci.yml`/`Makefile`/`justfile`/pre-commit hooks: absent)
- `docs/migration-plan/15-docs-refresh.md:154-161` — already records that "the repo also carries no CI config at all … 'CI lanes' attests nothing from this tree"
- Scope of a sweep: 625 `.rs` files / ~53,560 lines across workspace members `src/xlib/*`, `src/oxidauth/*`, `src/seedz` (`Cargo.toml:5-9`)

## Problem

Register T-2 (verbatim): `cargo fmt` is unrunnable repo-wide; `rustfmt.toml` pins `required_version = "1.8.0"`, installed toolchain is 1.11.0.

Verified against the actual toolchain state, the claim is true in effect but loose about the mechanism:

- `rustfmt.toml:3` pins `required_version = "1.8.0"`. rustfmt treats this as an exact-match guard and **aborts with a version-mismatch error** whenever its own version differs. (Not executed per assignment; behavior read from the config key's semantics — `error_on_unformatted`/`error_on_line_overflow` are false but `required_version` is a hard config check that ignores those escape hatches. [INFERENCE — message not observed by running fmt.])
- Plain `cargo fmt --all` resolves through `rust-toolchain.toml`'s channel `1.98.0`, which ships **rustfmt 1.9.0-stable** (verified: `rustfmt --version` → `1.9.0-stable`). 1.9.0 ≠ 1.8.0 → abort. Independently, stable rustfmt cannot honor the config's nightly-only options at all.
- The register's "installed toolchain is 1.11.0" matches the **latest installed nightly** (`nightly`, 2026-09-28 → `rustfmt 1.11.0-nightly`), not the repo-pinned toolchain. `cargo +nightly fmt --all` aborts on the same version-mismatch guard.
- A workaround already exists and is documented in two places (README:130-132, rust-toolchain.toml header): `cargo +nightly-2025-07-01 fmt --all`. That dated nightly ships `rustfmt 1.8.0-nightly` (verified), exactly matching the pin — which is why that date was chosen.

Accurate statement: the **only working fmt lane is a frozen ~14-month-old nightly** invoked by a magic string that lives in a README comment; every current toolchain (pinned stable channel, current nightly) hard-errors on `cargo fmt`.

## Analysis

- **Why the pin is structurally fragile:** `required_version` makes formatting reproducibility expire with every rustfmt release. The repo's response so far was to freeze `nightly-2025-07-01`, which converts a loud error into silent staleness: nobody can run current rustfmt on this tree, and no automation notices when hand-formatting drifts from even the old rustfmt's output.
- **Why the fmt lane must be nightly regardless of the bump:** `imports_granularity`, `group_imports`, `blank_lines_lower_bound`/`blank_lines_upper_bound`, `wrap_comments`, `format_code_in_doc_comments`, `force_multiline_blocks`, `trailing_comma` are nightly-only options (well-established set; per-option stability at 1.11.0 not re-verifiable offline). Stable rustfmt silently ignores them, so formatting the repo with stable would *fight* the hand-formatting. T-2's "then run workspace-wide" therefore necessarily means "workspace-wide on the canonical nightly", never plain `cargo fmt`. [INFERENCE for any option not in the well-known nightly-only set.]
- **What changed between rustfmt 1.8.0 and 1.9.0/1.11.0:** cannot be enumerated from the config alone, and 1.9→1.11 behavior changes postdate this analysis's reliable knowledge — the diff must be measured, not guessed (`fmt --all --check` in step 2 below). What the config does show is where drift concentrates: the repo leans hard on **unstable options**, and unstable options carry no formatting-stability promise across rustfmt releases (stable options may only change under a new `style_edition`, which this config pins to `"2024"`). Two specific keys are likely to error as deprecated/removed on a newer rustfmt — `make_backup` (:77, long-deprecated) and `brace_style` (:12, superseded by style-edition granular options) — [INFERENCE; the bump step surfaces unknown/deprecated-option errors authoritatively]. Register's "expected diff is large" is consistent with this: behavior fixes under style-edition 2024 plus unstable-option changes accumulate across three rustfmt minors.
- **Hand-formatting fidelity (the "tests were hand-formatted" claim):** faithful to **rustfmt 1.8.0's** interpretation of the config. Verified markers: imports merged per crate and grouped Std/external/crate per `imports_granularity=Crate` + `group_imports=StdExternalCrate` (e.g. `src/oxidauth/oxidauth-postgres/src/auth/tree/mod.rs:1-16`); zero single-line empty items (`empty_item_single_line=false` honored — 0 matches for `) {}`); zero single-line `if`/loop bodies (`force_multiline_blocks=true` honored); zero `rustfmt::skip` attributes repo-wide; sampled `lib.rs` files have zero lines >100 chars, and the 47 repo files that do exceed 100 chars sample as single unbreakable `use` paths and string literals (`format_strings=false`) — constructs rustfmt itself leaves over-long. The residual risk is not sloppiness but **version skew**: 1.11.0 would reformat parts of a tree that is internally consistent with 1.8.0.
- **CI gate:** none exists. There is no fmt gate to be pinned to any toolchain; the rust-toolchain.toml comment's "CI lanes" claim is already flagged wrong by migration-plan 15. So the bump carries no CI-migration burden — and equally, no CI currently catches formatting regressions.
- **`BUG(pinned)` proximity:** 51 source files carry `BUG(pinned)` test pins documenting known-broken behavior pending the ticket-fix wave (SEC-000001..13, DATA-000014..26, SRV-000037..50, notes OXA-000051..58; e.g. `oxidauth-kernel/src/error.rs:110`, `oxidauth-postgres/src/auth/tree/mod.rs:261`). None of these pins is formatting-sensitive (they assert values, not source layout), but those files are exactly what the open fix wave rewrites — pure rebase-conflict surface for a reformat sweep, which drives the sequencing constraint below.

## Impact

- Every contributor who types `cargo fmt --all` gets an opaque config-version error; those who don't read README:130-132 either skip formatting (drift the repo cannot detect — no CI) or format with a non-canonical rustfmt and generate churn.
- Reformat debt compounds monotonically: each rustfmt release widens the gap between the frozen-1.8.0 tree and current rustfmt output, making the eventual deliberate diff larger and riskier to land.
- The toolchain story is self-undermining: `rust-toolchain.toml` pins 1.98.0 "for everyone" while the one command every dev runs (`cargo fmt`) refuses to use it.

## Proposed resolution

**Disposition: FIX NOW — deliberate bump on a canonical dated nightly.** The frozen nightly is a countdown bomb with zero CI to detect drift, and the hand-formatting drift risk only grows.

1. **Pick one canonical dated nightly** and install it everywhere (dev image + `rust-toolchain.toml` header + README). Recommendation: a *dated* nightly, not moving `nightly`, so `required_version` stays satisfiable for months — `nightly-2026-05-16` (installed here, rustfmt 1.9.0-nightly) is the conservative pick; the register implicitly picks current `nightly` (rustfmt 1.11.0). Decide once; the recipe is identical.
2. **Measure before touching anything:** `cargo +<canonical> fmt --all --check`; the error list tells you which config keys are deprecated/removed (expected casualties: `make_backup`, possibly `brace_style` — prune them in `rustfmt.toml` at this step), and the check-diff gives the true blast radius instead of the register's "expected large" guess.
3. **Bump `required_version`** in `rustfmt.toml:3` to the canonical nightly's rustfmt version. Keep `unstable_features = true` and `style_edition = "2024"` — do not attempt to make plain `cargo fmt` work; that is impossible while the config uses nightly-only options, and pretending otherwise would reformat the repo to a *different style*.
4. **Single workspace-wide sweep as a reformat-only commit** (`cargo +<canonical> fmt --all`, message: `style: reformat with rustfmt <ver> [OXA-000062]`, zero semantic edits). Prefer one sweep over per-crate rollout: one shared config, no crate-specific style, and per-crate would leave the tree half-failing `--check` and double the rebase window.
5. **Sequencing: land BEFORE the fix wave opens.** The 51 `BUG(pinned)` files are about to be rewritten by the SEC/DATA/CLI/SRV/notes tickets (OXA-000001..58). Reformat-first means fix branches rebase over small fix diffs; reformat-after means every in-flight branch rebases over a whole-file whitespace diff. If the wave has already started by execution time, fall back to per-crate sweeps at merge breaks — but never interleave reformat hunks into fix commits.
6. **Update the convention docs in the same commit** as the config bump: `rust-toolchain.toml:1-4` header, `README.md:130-132`, and register T-2 at `BUGS_AND_NOTES.md:97`.
7. **Rejected alternative — bump `required_version` to the installed version WITHOUT running fmt:** it only buys a non-erroring `--check` for nobody (there is no CI fmt gate to satisfy, and any CI added later must adopt the canonical nightly anyway because of the nightly-only options). Meanwhile it abandons the repo's own formatting contract: the tree would be *defined* unformatted relative to its config, the first person to run fmt gets the same large diff, and that diff then lands contaminated inside unrelated fix commits — the exact outcome step 5 prevents.
8. **Adjacent hardening (optional, out of T-2 scope):** when CI finally lands, add a `cargo +<canonical> fmt --all --check` lane; a `bin/fmt.sh` encoding the incantation would keep plain `cargo fmt` misuse from producing half-formatted files in the meantime.

## Verification

- `rustup run <canonical> rustfmt --version` numerically equals the new `required_version`.
- `cargo +<canonical> fmt --all --check` exits 0 across all members (`src/xlib/*`, `src/oxidauth/*`, `src/seedz`) after the sweep.
- Reformat-only commit audit: `git show --stat` touches `*.rs` (+ `rustfmt.toml` if bundled per step 6) only; hunks are whitespace/line-wrapping only; behavior gate per repo rule T-3 (`cargo check` skips `#[cfg(test)]` code) is a full `cargo test --workspace --no-run` compile — run once after the sweep, not in this ticket.
- Convention docs agree: `grep -rn "nightly-" README.md rust-toolchain.toml` cites the canonical nightly; `BUGS_AND_NOTES.md:97` struck.
- Expected-by-design residual: plain `cargo fmt --all` on the pinned stable channel still errors (stable rustfmt ≠ canonical nightly version, and stable ignores the nightly-only options). That is not a regression — it is the config's design; the verification target is that the *documented lane* is current rustfmt, not a frozen 2025 nightly. [No fmt/check runs were performed for this ticket; pre-bump error behavior is config-derived and marked as such above.]
