- [90000009](https://www.pivotaltracker.com/story/show/90000009) - rename
  `oxidauth-usecases` → `oxidauth-services`; kernel `Service<&X>` aliases
  redefined as named async traits + `Arc<dyn XServiceTrait>`; `CanLayer`/
  `CanService` copied into `oxidauth-api` middleware
    - **BREAKING (public API, for 2.0):** `oxidauth_kernel::service::Service`
      is `#[deprecated(since = "0.5.0")]` — the generic `service.call(&x)`
      dispatch is retired. Every `pub type XService = Arc<dyn for<'a>
      Service<&'a X, Response = R, Error = BoxedError>>` alias (68 of them)
      is redefined **in place, same alias name** as
      `#[async_trait] pub trait XServiceTrait: Send + Sync + 'static { async
      fn x_method(&self, params: &X) -> Result<R, BoxedError>; }` +
      `pub type XService = Arc<dyn XServiceTrait>;`. Aliases are NOT
      deprecated; consumers keep the same type names but call named methods
      (`create_user`, `authenticate`, `fetch_setting`, …). Traits stay in the
      kernel (review decision — deliberate deviation from the template, which
      puts them in the services crate).
    - **BREAKING:** crate rename `oxidauth-usecases` → `oxidauth-services`
      (`src/oxidauth/oxidauth-usecases` → `src/oxidauth/oxidauth-services`,
      package `oxidauth-services`); reverse deps flipped: `oxidauth-api`
      (provider wiring, bootstrap in `main.rs`) and `oxidauth-rs` (keeps the
      dependency; its `pub use oxidauth_usecases::…` re-export surfaces now
      read `oxidauth_services::…` and export the identical items — the
      `auth::strategies::*` glob and
      `username_password::registrar::UsernamePasswordRegisterParams` are
      unchanged in the crate). `oxidauth-services` drops
      `tracing-bunyan-formatter` (logging is an api-layer concern).
    - `oxidauth-services`: every `impl Service<&X> for XUseCase` → `impl
      XServiceTrait for XUseCase` (61 impls), bodies verbatim; the leading
      `&'a` lifetime and `type Response/Error` lines are gone; every trait
      impl's `#[tracing::instrument]` renamed to the template convention
      `XUseCase::x_method`. Service-typed fields (e.g. `accept_invitation`'s
      `UpdateUserService`) call named methods now; repository (plan-08
      `*Query`) fields intentionally keep `.call`.
      `SudoUserBootstrapUseCase` implements `BootstrapServiceTrait::bootstrap`
      and its 19 provider-fetched service calls are named.
    - `oxidauth-api`: all 58 handler call sites in `src/server/**` moved from
      `.call(&x)` to the named method (plus 1 in
      `middleware/permission_extractor.rs` and 1 in `main.rs`); `grep
      '\.call(&' src/oxidauth/oxidauth-api/src` → 0. Provider store lines keep
      the unchanged `XService` alias names (58 `provider.store::<…>` blocks —
      the reviewer-flagged duplicate `CreateRolePermissionGrantService` store
      block from plan 08 is deleted, 59 → 58). `CanLayer`, `CanService`,
      `CanError`, `ExtractPermissions` copied into
      `oxidauth-api/src/middleware/can.rs` (api-local, incl. unit tests);
      they compose over the still-living kernel `Layer` trait and reference
      the deprecated `Service` under a local `#[allow(deprecated)]`.
      `oxidauth-api` gains `async-trait`.
    - `oxidauth-kernel`: `#[deprecated]` on `service::Service` (plus
      `CanLayer`/`CanService`); `Layer`, `CanError`, `ExtractPermissions` and
      all `XService` aliases stay live. The old blanket
      `users::create_user::CreateUserTrait` (marker over `Service<&CreateUser>`)
      is deleted — superseded by `CreateUserServiceTrait`; its only consumer
      (`invitations::create_invitation` bound) migrated.
      `totp_secrets::create_totp_secrets_by_authority_id` was already in the
      target shape (`CreateTotpSecretsTrait`) and keeps its pre-existing
      published trait name.
    - **Retained `Service` dependents (2.0/plan-14 scope, untouched per
      review):** `oxidauth-postgres` keeps its 58 `impl Service<&X> for
      Pg<Entity>Repository` blocks and `oxidauth-repository` keeps its
      plan-08 `trait XQuery: for<'a> Service<&'a X, …>` supertrait blanket
      traits (94 naming lines) — the query-side removal is plan 14's story.
      Because of that supertrait chain the kernel deprecation warnings will
      surface in three crates, not just kernel: ~38 naming kernel files
      (`service.rs` + the domain files that `pub use … service::Service`),
      ~59 repository files, ~59 postgres files.
