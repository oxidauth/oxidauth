# 07 — Split DTO crate: `oxidauth-http` (DTOs) ↔ `oxidauth-api` (server)

**Status**: `reviewed` (walkthrough 2026-09-29: approved — option (A) flattened DTO paths, breaking accepted; parkinglot unaffected per import audit)
**Depends on**: 03 (xlib http), 04
**Risk**: highest of the early plans — public API churn on `oxidauth-http`
(crates.io 0.8.0) and import rewrites in `oxidauth-rs`. Two commits: RENAME,
then EXTRACT.

## Goal

Template layering separates **`-http` = DTOs** from **`-api` = handlers**.
Today the server crate `oxidauth-http` holds both, and the client crate
`oxidauth` (`-rs`) imports DTOs through `oxidauth_http::server::api::v1::<domain>::<endpoint>::*Req/*Res`
(53+ files, e.g. `client/mock.rs` imports `CreateUserReq` from the server
module tree). Target:

```
src/oxidauth/oxidauth-http   (NEW DTO crate, version line 0.9.0)
├── src/lib.rs               re-export modules: users, auth, authorities, permissions,
│                            roles, public_keys, refresh_tokens, can, settings,
│                            invitations, totp, __meta + `pub use http::{Response, ...}` (xlib)
├── src/users.rs             CreateUserReq/Res, FindUserByIdReq/Res, UpdateUserBodyReq, …
├── src/auth.rs              RegisterReq/Res, AuthenticateReq/Res, oauth2 redirect/callback DTOs, …
└── …                        one module per api/v1 domain, one section per endpoint file
src/oxidauth/oxidauth-api    (RENAMED current server crate, new name)
├── src/main.rs              unchanged module set (middleware, provider, server)
├── src/server/api/v1/…      handlers keep handle() + PERMISSION + routers ONLY
└── src/response.rs          DELETED — `oxidauth_http::response::Response` becomes a
                             re-export of xlib `http::Response` (identical fields; keep
                             `error()/payload()/unauthorized()/fail()` builders — port
                             them to the xlib type if xlib lacks any builder today)
```

## Changes — commit 1: rename server crate

1. `git mv src/oxidauth/oxidauth-http src/oxidauth/oxidauth-api`; package +
   bin name `oxidauth-http` → `oxidauth-api` in its `Cargo.toml` (bin key:
   `[[bin]] name = "oxidauth-api"` or package-name-derived). Update reverse
   deps: nothing in-workspace depends on the server except the rs crate's
   DTO imports (fixed in commit 2). `docker-compose.yml` command
   `cargo run --bin oxidauth-http` → `oxidauth-api` (interim; rewritten
   plan 10).
2. Metadata: keep `license = "GPL-3.0"`, homepage/repository; description
   "http server for oxidauth". New crates.io name `oxidauth-api`;
   `bin/publish.sh` (plan 11 updates) publishes it.

## Changes — commit 2: extract DTOs

3. Create new `src/oxidauth/oxidauth-http` crate (`version = "0.9.0"`,
   edition 2021 here — bump in plan 14; keep GPL-3.0 + publish metadata so
   existing consumers get a drop-in upgrade path). Deps: `oxidauth-kernel`,
   `serde`, `chrono`, `uuid`, `url`, `rust_decimal` + xlib http
   (`http = { path = "../../xlib/http" }`) — **no axum, no tokio**.
   `lib.rs` re-exports `pub use http::Response;` so client imports of
   `oxidauth_http::response::Response` can become
   `oxidauth_http::Response` (support both during deprecation: keep a
   `pub mod response { pub use http::Response; }` shim + `#[deprecated]`).
4. Move every `#[derive(Serialize, Deserialize)] pub struct *Req/*Res/*Body*`
   from handler files into the DTO crate, **flattened one module per domain**:
   `oxidauth_http::users::create_user::{CreateUserReq, CreateUserRes}` —
   keeping the endpoint leaf file name so the mapping is 1:1 (mechanical
   grep-driven move: `grep -rn 'pub struct .*Req\b\|pub struct .*Res\b'
   src/oxidauth/oxidauth-api/src/server`).
   Path delta for consumers: drop the `server::api::v1::` prefix.
   Add `#[deprecated]`-free hard break — documented in the crate changelog;
   this is why the DTO crate goes to 0.9.0 (next major candidate when
   kernel goes 2.0).
5. Handler files keep: `handle()`, `PERMISSION` consts, router `mod.rs`,
   `pub use oxidauth_kernel::<domain>::<endpoint>::*` param re-exports stay in
   kernel (they already live there — e.g. `CreateUser` input struct stays in
   kernel; only wire-DTOs move).
6. `oxidauth-rs`: rewrite all `use oxidauth_http::server::api::v1::X::Y` →
   `use oxidauth_http::X::Y` (sed-able: `perl -pi -e
   's/oxidauth_http::server::api::v1::/oxidauth_http::/g'`); drop
   `oxidauth-usecases` and `axum`-server-side deps from rs where the DTO
   split makes them unnecessary (audit `cargo tree -p oxidauth` — target:
   client/wasm builds carry **no tokio server features**; axum stays only for
   the `axum::extract` FromRef integration feature-gated `server`).
7. `middleware/permission_extractor.rs`: unchanged in this plan (server
   extractors live in `-api`; they now fetch DTOs from the DTO crate — mostly
   no-op imports).
8. `__meta` shape to template: `oxidauth_http::__meta::healthcheck::HealthcheckRes
   { version, healthy }` DTO; api handler returns it on `GET
   /api/v1/__meta/healthcheck` (rename from `health_check` — snake_case route
   is off-convention; keep `live_check` handler but register as
   `livecheck` too, keep old routes as aliases **one release** with a
   `#[deprecated]` note in README, then drop). Healthcheck body replaces bare
   200/500: `healthy: db.ping().is_ok()`, `version: env!("CARGO_PKG_VERSION")`.
   `__meta` prefix (double underscore) is the template convention — already
   in use here. ✓

## Verification

- `cargo check --workspace --all-features` green.
- `cargo tree -p oxidauth -F wasm` contains no `oxidauth-api`, no tokio.
- Compile-level contract test (throwaway): client snippet from parkinglot's
  provider crate — `oxidauth::OxidAuthClient::from_ref(&provider)` — still
  builds against the new tree; `oxidauth::{OxidAuthClient, OxidAuthClientTrait,
  OxidAuthClientMock, permissions}` public exports unchanged.
- Response envelope on the wire unchanged: run server + hurl suite;
  byte-compare `users.hurl`/`authenticate.hurl` JSON keys
  (`success/payload/errors/warnings/notices/status_code`).
- `cargo publish -p oxidauth-http --dry-run` succeeds (DTO crate standalone).

## PR note

changelog `<id>-http-dto-split`. This is the plan to review hardest; flip
Status → `done` only after both commits merge together.
