- [OXA-000071](https://www.pivotaltracker.com/story/show/OXA-000071) - the kernel `Provider` is gone; the xlib `provider` crate is the only Provider
    - **Kernel DI corpse retired:** `oxidauth-kernel/src/provider/mod.rs` and
      its `pub mod provider;` declaration are deleted. Plan 04 had moved every
      consumer to the vendored xlib `provider::Provider` (api wiring + all 58
      handler `fetch_unchecked` sites, services bootstrap) and deprecated the
      kernel struct "until the coordinated 2.0"; with zero remaining
      references (`grep -rn 'oxidauth_kernel::provider' src/` → 0) the
      removal is the switch's last step, mirroring the OXA-000069 removal of
      the deprecated kernel `Service` trait. `oxidauth-api/src/provider/mod.rs`
      keeps re-exporting the xlib type as `crate::provider::Provider`, so no
      handler import changed.
    - **Deprecation noise at zero:** the 3 pre-existing `use of deprecated`
      warnings counted in OXA-000069 (the kernel struct's own deprecation) are
      gone — `cargo build --workspace | grep -c 'use of deprecated'` = 0.
    - **Unrelated repair (was blocking the all-targets gate):** 14
      `oxidauth-postgres` test mods (authorities ×2, permissions ×2, roles ×6,
      settings, users ×3) called sibling seed/re-read traits
      (`insert_authority`, `insert_role`, `select_role_by_id`,
      `select_role_by_name`, `select_permission_by_parts`, `save_setting`,
      `insert_user`, `select_authority_by_id`) without importing their
      repository trait — 16 `E0599` compile errors predating this change. One
      glob import per file added inside the test mods; no behavior touched.
    - **Docs:** `docs/CLIENT_MIGRATION.md` §4 no longer says the deprecated
      `Service`/`Provider` are "kept compiling until a coordinated 2.0" — both
      are now removed and the xlib `provider::Provider` is the only one.
    - **Semver — breaking removal for external consumers of `oxidauth-kernel`,
      rides the unpublished 0.9.0** (same ruling as the `Service` trait);
      in-tree consumers were already fully migrated.
    - **Verification:** `cargo check --workspace --all-targets` green;
      `cargo test -p oxidauth-kernel` 53/53; `cargo test -p oxidauth-postgres`
      153/153 against the live dev DB (exercises the repaired test mods);
      `cargo test -p oxidauth-services` 279/279 (bootstrap constructs the
      xlib `Provider` throughout); `cargo test -p oxidauth-api` 76/76;
      `cargo fmt --check` clean.
