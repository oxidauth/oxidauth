- [90000007](https://www.pivotaltracker.com/story/show/90000007) - split the
  DTO crate `oxidauth-http` from the renamed server crate `oxidauth-api`
    - **BREAKING (crate rename):** the server crate
      `src/oxidauth/oxidauth-http` (package `oxidauth-http`) is renamed
      `src/oxidauth/oxidauth-api` (package `oxidauth-api`, version 0.8.0,
      bin name follows the package — `cargo run --bin oxidauth-api`).
      `docker-compose.yml`, `bin/build-server.sh` (binary, Cargo.toml path,
      Dockerfile path, image name), and the crate's `Dockerfile`
      (`/bin/oxidauth-api`) follow. The telemetry service label and boot log
      are now `oxidauth-api`.
    - **BREAKING (DTO paths):** new crate `src/oxidauth/oxidauth-http`
      (package `oxidauth-http`, version **0.9.0**, GPL-3.0 + publish
      metadata, deps: `oxidauth-kernel`, xlib `http`, `serde`, `serde_json`,
      `chrono`, `rust_decimal`, `url`, `uuid` — **no axum, no tokio**) now
      owns every wire DTO. All `oxidauth_http::server::api::v1::<domain>::
      <endpoint>::*` paths drop the `server::api::v1::` prefix:
      `oxidauth_http::users::create_user::CreateUserReq`. Mechanical rewrite
      for consumers: `perl -pi -e
      's/oxidauth_http::server::api::v1::/oxidauth_http::/g'`. No
      back-compat re-exports (documented hard break, next major candidate).
    - **BREAKING (Response):** the server crate's `src/response.rs` is
      deleted; `oxidauth_http::Response` is a re-export of the xlib
      `http::Response` envelope (identical JSON keys:
      `success/payload/errors/warnings/notices`).
      `oxidauth_http::response::Response` remains as a `#[deprecated]` shim.
      The old crate type's no-arg `Response::fail()` (= BAD_REQUEST envelope)
      has no same-signature twin on the xlib type; every call site became
      `Response::bad_request()` — byte-identical body and 400 status.
    - **BREAKING (status honesty):** the deleted response type's
      `IntoResponse` ignored the stored status and returned 200 unless
      `errors` was set. xlib's `IntoResponse` honors the stored status, so
      wire-visible deltas are: `Response::unauthorized()` now answers
      **401** (was 200 with `success:false`), an error-free
      `Response::bad_request()` now answers **400** (was 200), and
      `.status(…)` now actually applies. Envelope JSON keys unchanged.
    - **BREAKING (healthcheck body):**
      `GET /api/v1/__meta/health_check` / `live_check` are renamed to
      `__meta/healthcheck` / `__meta/livecheck` (old paths registered as
      aliases for one release, `#[deprecated]` note in the api crate
      README). Responses replace the bare status codes with
      `oxidauth_http::__meta::healthcheck::HealthcheckRes { version,
      healthy }` / `__meta::livecheck::LivecheckRes { version, healthy }`
      inside the standard `Response` envelope: healthy = `db.ping().is_ok()`
      (200 healthy / 500 db-down, as before), livecheck is always 200
      `healthy: true` (no db ping).
    - moved inventory: 56 DTO leaf modules (pub structs `*Req/*Res/*Body*`
      and `pub type` Req/Res aliases, 1:1 with the endpoint leaf file
      names); private `type *Req = kernel` aliases (`DeleteAuthorityReq`,
      `FindAuthorityByIdReq`) and the oauth2 `PathParams` path extractor
      stay in `oxidauth-api`; kernel param structs (e.g. `CreateUser`,
      `ListAllUsers`) stay in `oxidauth-kernel` — handler files keep their
      `pub use oxidauth_kernel::…::*` re-exports, `handle()`, `PERMISSION`
      consts, and routers only
    - moved helper DTOs used inside wire structs: `UpdateUserUser`
      (users/update_user), `UserAuthorityParams`
      (users/authorities/create_user_authority). Fields of moved structs
      that handlers read across the crate boundary became `pub`
      (`UpdateUserPathReq.user_id`, `UpdateUserAuthorityPathReq.
      user_id/authority_id`) — wire shape unchanged.
    - `oxidauth-rs`: imports rewritten to the new DTO paths; kernel-owned
      types previously reached through server glob paths
      (`CreateAuthority`, `UpdateAuthority`, `CreateRole`, `UserKind`,
      `AuthorityStrategy`, `AcceptInvitationParams`,
      `AcceptInvitationUserParams`, `CreateInvitationParams`) now come from
      `oxidauth_kernel` directly; `prelude::parse_and_validate` re-exports
      `oxidauth_permission` instead of the server crate. `axum`/
      `axum-extra` are now `optional` behind the `server` feature (which
      gates the `axum::extract::FromRef` integration module and stays in
      `default`); `tokio` narrowed from `full` to `sync + macros +
      rt-multi-thread` (client uses `tokio::sync`, `test_client` bin needs
      the runtime). `oxidauth-usecases` stays — the client re-exports the
      strategy params (`oxidauth_usecases::auth::strategies`) as public
      API; the DTO split does not make that dep unnecessary.
    - no xlib ports were needed: xlib `http::Response` already had every
      builder (`success/fail/unauthorized/internal_error/bad_request/
      status/payload/error/warning/notice`)
    - docs left for plan 15 (non-functional): `docs/migration-plan/*`,
      `docs/SECURITY_REPORT.md`, `rfcs/4-forgot-password-flow/rfc.md`
      prose still says `oxidauth-http` for the server
