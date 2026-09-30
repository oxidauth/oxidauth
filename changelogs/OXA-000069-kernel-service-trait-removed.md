- [OXA-000069](https://www.pivotaltracker.com/story/show/OXA-000069) - plan-09: every named site moved off the kernel `Service` trait, then the trait was removed (supersedes OXA-000067's T-7 rewrite)
    - **Kernel dispatch machinery deleted:** `oxidauth-kernel/src/service.rs` is
      gone — `Service`, `Layer`, `CanLayer`, `CanService`, `CanError`,
      `ExtractPermissions` and the module's `pub mod service;` declaration.
      The 37 `pub use crate::service::Service;` re-exports across the kernel
      request modules were removed, `dev_prelude` narrowed from
      `pub use crate::{error::BoxedError, service::*};` to
      `pub use crate::error::BoxedError;`, and the 47 `oxidauth-repository`
      trait files + `oxidauth-repository/src/prelude.rs` dropped their
      `service::Service` re-exports. The compiler was the completeness proof:
      removal compiled only because every consumer had first been migrated.
    - **Named `*Query` traits now own real methods:** the 57 Service-bound
      repository traits (of 60; 3 already had methods) carry their real async
      method (e.g. `InsertRoleQuery::insert_role(&self, params: &CreateRole)`);
      the `Service<&Params>` supertraits and the
      `impl<T: Service<…>> XQuery for T` blanket bridges are deleted. All 58
      `oxidauth-postgres` impls and 105 `oxidauth-services` mocks implement the
      named traits directly, bodies unchanged (same SQL, same
      `#[tracing::instrument]` names); use-case, postgres-test and fixture call
      sites call the named methods instead of `.call(&params)`.
    - **`oxidauth-api/src/middleware/can.rs` rebuilt** as a direct stateless
      check: `can()` = `parse` + split entitlements + `validate`, the
      `CanLayer`/`CanService` composition deleted; `CanError` (same variants,
      same mapping) is now API-local. The `/can/{permission}` endpoint, the
      OXA-000056 statelessness invariant and the allow/deny twin tests are
      preserved.
    - **Warning noise retired with the trait:** the ~114 `use of deprecated`
      warnings across kernel/repository/services dropped to zero
      `Service`-related (measured `cargo build -p oxidauth-repository -p
      oxidauth-postgres -p oxidauth-kernel | grep -c 'use of deprecated'` = 3,
      all pre-existing and unrelated: the xlib `provider::Provider` deprecation
      in `oxidauth-kernel/src/provider/mod.rs`). All suppression attrs went
      with it: 46 `#![allow(deprecated)]` (postgres test mods) and 1
      `#[allow(deprecated)]` (services bootstrap) deleted — tree-wide
      `allow(deprecated)` under `src/oxidauth` is now 0 (api's 2 had already
      gone with the can.rs rebuild).
    - **Semver — breaking removal for external consumers of `oxidauth-kernel`,
      rides the unpublished 0.9.0; no extra version bump.** The `Service`
      trait did exist in the only published `oxidauth-kernel` version (0.1.0,
      2023); the `#[deprecated]` attributes were working-tree-only (never
      committed, absent from HEAD), so the "removed for 2.0" note never
      shipped a deprecation. `oxidauth-repository` was never published. The
      removal ships in the pre-1.0 0.9.0 line; in-tree consumers are fully
      migrated (unit 500 / DB 163 green, kernel 48→46 = exactly the two
      service.rs tests).
