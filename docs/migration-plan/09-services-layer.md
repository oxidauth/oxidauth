# 09 — `usecases` → `services` layer

**Status**: `reviewed` (walkthrough 2026-09-29: approved with tweak — trait definitions STAY in kernel)
**Depends on**: 08
**Risk**: medium — renames ripple into provider wiring and api handlers.

## Goal

Layer 4 gets the template name and shape: `oxidauth-usecases` →
**`oxidauth-services`**, holding the use cases. **Service traits stay in the
kernel** (review decision — documented deviation from the template).

Template shape for reference (xlib services doc-comment, parkinglot
`admin-services/src/property.rs`) — note we deliberately invert the trait
location; the impl pattern is ours already:
```rust
pub type CreateEntityService = Arc<dyn CreateEntityTrait>;
#[async_trait] pub trait CreateEntityTrait: Send + Sync + 'static { … }  // ← kernel in our variant
pub struct CreateEntityUseCase<R> { repository: R }   // impl over R: InsertQuery
```

## Changes

1. `git mv src/oxidauth/oxidauth-usecases src/oxidauth/oxidauth-services`;
   package `oxidauth-usecases` → `oxidauth-services`; update reverse deps
   (`oxidauth-api`, interim `oxidauth-rs` if not yet dropped in plan 07).
2. **Named traits, defined in kernel** (review decision 2026-09-29):
   `oxidauth-kernel/src/<domain>/*.rs` today define
   `pub type CreateUserService = Arc<dyn Service<&CreateUser, Response = User,
   Error = BoxedError>>;`. Redefine **in place** as named traits + alias —
   same symbol names, new shape:
   - `#[async_trait] pub trait CreateUserServiceTrait: Send + Sync + 'static
     { async fn create_user(&self, params: &CreateUser) -> Result<User,
     BoxedError>; }` (kernel file)
   - `pub type CreateUserService = Arc<dyn CreateUserServiceTrait>;` (kernel
     file — published consumers keep working against the same alias names)
   - `oxidauth-services/src/<domain>/<endpoint>.rs` contains only the
     `CreateUserUseCase` struct + `impl CreateUserServiceTrait for …`
   - input structs + domain types unchanged in kernel
   - Handlers switch `service.call(&x)` → `service.create_user(&x)`
     (dropping the generic `Service::call` indirection) — mechanical, ~40
     call sites in `oxidauth-api/src/server/api/v1/**`.
3. **Kernel deprecations** (still published — keep compiling, flag for 2.0):
   - `#[deprecated]` on `oxidauth_kernel::service::{Service, CanLayer,
     CanService}` **only** — the `XService` aliases are NOT deprecated; they
     are redefined (item 2) and remain the public dispatch surface.
   - **Move `CanLayer`/`CanService` permission middleware**: these are
     server-side composition (`oxidauth-permission::parse_and_validate` +
     entitlement checks). Copy them into
     `oxidauth-api/src/middleware/can.rs` (used by api handlers); kernel
     versions deprecated in place. (`oxidauth-rs` has its own axum
     extractors; unaffected.)
4. Use-case internals unchanged: `AuthenticateUseCase`'s 8 generic params
   keep working — now typed against the repository traits (plan 08) instead
   of `Database`.
5. `#[tracing::instrument(name = "CreateUserUseCase::create_user", skip(self))]`
   on every trait impl (template/parkinglot instrument naming convention).
6. Deps of `-services`: `oxidauth-kernel`, `oxidauth-repository`,
   `async-trait`, `tracing`, `argon2`, `boringauth`, `reqwest` (OAuth profile
   fetch legitimately lives here), `jsonwebtoken`/`rsa` via kernel,
   `tracing-bunyan-formatter` **remove** (logging is api-layer concern).
   `serde_json`, `chrono`, `uuid`, `url` as used.

## Verification

- `grep -rn 'oxidauth_usecases' src/ bin/` → 0.
- `grep -rn 'Service::call\|\.call(&' src/oxidauth/oxidauth-api/src` → 0
  (all handlers on named methods).
- `cargo check --workspace` + deprecation warnings appear **only** inside
  kernel itself.
- hurl suite green — service dispatch is behavior-preserving.
- `cargo doc -p oxidauth-services` renders the trait + alias pairs.

## PR note

changelog `<id>-services-layer`. Split PRs by domain cluster if review needs
it (same as plan 08 guidance). Status flips with the final PR.

## Review notes

- 2026-09-29 walkthrough: **trait definitions stay in kernel** (template puts
  them in the services crate; rejected to preserve the published kernel
  surface). Use cases live in `-services`; handlers move to named methods.
  Remainder approved as written.
