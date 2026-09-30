# oxidauth-http

Wire DTOs (request/response types) for [oxidauth](https://oxidauth.rs).

This crate holds only serde request/response types plus the shared `Response`
envelope re-exported from the `http` xlib. Handlers and routers live in
`oxidauth-api`. There is deliberately **no axum/tokio dependency** — client and
wasm consumers can depend on this crate without pulling server runtime code.

## 0.9.0 breaking changes

- DTO module paths dropped the `server::api::v1::` prefix:
  `oxidauth_http::server::api::v1::users::create_user::CreateUserReq` →
  `oxidauth_http::users::create_user::CreateUserReq`.
- The server crate was renamed `oxidauth-http` → `oxidauth-api`.
- `oxidauth_http::response::Response` is a deprecated shim; use
  `oxidauth_http::Response`.
- `__meta::healthcheck::HealthcheckRes { version, healthy }` replaces the bare
  status-code health response.
