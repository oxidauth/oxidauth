- [90000003](https://www.pivotaltracker.com/story/show/90000003) - Vendor xlib crates from project-template
    - `cp -R`'d `src/xlib/{http,postgres,provider,telemetry}` from
      `~/dev/freshbrewlabs/project-template/src/xlib` (11 files, contents
      byte-identical except the two deviations below)
    - each crate's `src/lib.rs` gained one header doc line:
      `//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.`
    - `src/xlib/provider` decoupled from oxidauth (inside this repo the
      template's `oxidauth = { git = ..., rev = "f358259" }` dep is a
      self-dependency loop pinned to a stale rev): removed the dep from
      Cargo.toml, removed `use oxidauth::{OxidAuthClient, axum::extract::FromRef};`
      and `impl FromRef<Provider> for OxidAuthClient` from lib.rs, leaving a
      `///` doc-comment (consumers re-add the impl in their own tree, see
      parkinglot); `Provider` + `ProviderError`
      and all 3 provider unit tests kept verbatim (tests have zero oxidauth
      references)
    - root Cargo.toml: added `"src/xlib/*"` to members (first entry);
      exclude unchanged — all four crates ship manifests
    - nothing consumes xlib yet; `take()` (present in this xlib, absent from
      the kernel Provider) gets adopted server-side in plan 04
