# oxidauth-api

HTTP server for [oxidauth](https://oxidauth.rs) (formerly the `oxidauth-http`
crate — renamed in 0.9.x of the workspace split; wire DTOs now live in the
`oxidauth-http` DTO crate).

## Deprecated routes

The `__meta` probe routes were renamed to match the snake_case-free route
convention. The old paths still work **for one release** and will be removed:

| Deprecated (removed next release) | Replacement                  |
| --------------------------------- | ---------------------------- |
| `GET /api/v1/__meta/health_check` | `GET /api/v1/__meta/healthcheck` |
| `GET /api/v1/__meta/live_check`   | `GET /api/v1/__meta/livecheck`   |

The healthcheck body also changed: it now returns
`oxidauth_http::__meta::healthcheck::HealthcheckRes { version, healthy }`
inside the standard `Response` envelope instead of a bare status code.
