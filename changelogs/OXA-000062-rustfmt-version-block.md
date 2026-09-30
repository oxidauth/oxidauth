- [OXA-000062](https://www.pivotaltracker.com/story/show/OXA-000062) - unblock the fmt lane: bump `required_version` 1.8.0 → `>=1.11.0` on a canonical dated nightly and reformat the workspace (register T-2)
    - `rustfmt.toml` pinned `required_version = "1.8.0"`, an exact-match guard,
      so every current toolchain — the pinned stable 1.98.0 channel (rustfmt
      1.9.0-stable) and current nightly (1.11.0) — hard-errored on `cargo fmt`;
      the only working lane was the frozen `nightly-2025-07-01` magic string.
      The canonical fmt toolchain is now **`nightly-2026-09-27`** (rustfmt
      1.11.0-nightly): `required_version` bumped to `">=1.11.0"`, and the
      whole workspace reformatted in one pass with
      `cargo +nightly-2026-09-27 fmt --all` — formatting only, zero semantic
      edits (drift was import regrouping, chain/line re-wrapping, empty-item
      braces, and hex-literal uppercasing; 228 `.rs` files).
    - Measured, not guessed: pre-bump `--check` on the canonical nightly showed
      **no deprecated/removed config keys** — the expected casualties
      `make_backup` and `brace_style` are still accepted and stay; only
      `required_version` changed. `unstable_features = true` and
      `style_edition = "2024"` stay.
    - Why the pin is the comparator `">=1.11.0"` (measured on both channels,
      so nobody re-derives this the hard way): the version guard compares
      **major.minor.patch only** — the `-nightly`/`-stable` channel suffix is
      stripped before matching, so a `"-nightly"` suffixed pin is unsatisfiable
      by *any* toolchain including the canonical nightly; and the current
      stable-channel rustfmt (1.9.0-stable) is **lenient** — it does not
      enforce a file-sourced `required_version` at all and formats with only
      `can't set …` warnings for every nightly-only option. So no pin string
      can make plain `cargo fmt` hard-error on today's stable toolchain; the
      comparator pin instead hard-rejects rustfmt **older** than 1.11.0
      (measured: `nightly-2026-05-16`'s 1.9.0 aborts with
      `doesn't match the required version (>=1.11.0)`), which is what keeps
      stale toolchains from reformatting back to old style. The stable-lane
      hazard is therefore a documented-convention residual, not a guard one:
      plain `cargo fmt` is NEVER the fmt lane (it ignores the nightly-only
      options and reformats to a different style — 334 hunks across 192 files
      of churn). When
      the stable channel's rustfmt reaches 1.11+, this pin should become an
      exact `=1.11.x`-family spec so the guard starts enforcing it.
    - Convention docs updated in the same pass: `rust-toolchain.toml` header
      and `README.md` now cite `cargo +nightly-2026-09-27 fmt --all`.
      Replaces the retired register's T-2 line (BUGS_AND_NOTES is retired).
    - Verification: `rustup run nightly-2026-09-27 rustfmt --version` →
      `1.11.0-nightly`, satisfying the pin; `cargo +nightly-2026-09-27 fmt
      --all --check` exits 0 across all workspace members after the sweep
      (the 1.9.0-formatted tree passed the 1.11.0 check with zero additional
      diff — the two nightlies agree on this config).
