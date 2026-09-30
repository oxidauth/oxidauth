# oxidauth — stack

The authentication & identity stack: axum HTTP server, PostgreSQL storage,
and the published reqwest-based Rust client (`oxidauth`). No web layer — this
service ships no UI (the helm chart keeps `web.enabled: false` for a future
admin-UI stack).

## Architecture

The template's layer order is kernel → repository → postgres/services →
http → api, but the real dependency graph is not a ladder — `services`
deliberately does **not** depend on `postgres` (wiring happens in the api's
provider), and `http` is a leaf DTO crate hanging off kernel that both the
server and the client read. Verified, normal-kind workspace edges:

```text
oxidauth-permission   -> <none>                       (permission-token parser)
oxidauth-kernel       -> oxidauth-permission          (domain types + service traits)
oxidauth-repository   -> kernel                       (query traits, one per operation)
oxidauth-postgres     -> kernel, repository, xlib/postgres   (Pg repositories + sqlx migrations)
oxidauth-services     -> kernel, repository, xlib/provider  (UseCases — NO postgres edge)
oxidauth-http         -> kernel, xlib/http            (wire DTOs + Response re-export)
oxidauth-api          -> http, kernel, permission, postgres, services, xlib/provider, xlib/telemetry
oxidauth (rs)         -> http, kernel, permission, services¹  (the published client crate)
seedz                 -> kernel, postgres, xlib/telemetry     (project-level, not in this dir)

¹ only for re-exported strategy param types (auth::strategies::*); a 2.0
  cleanup candidate so the client stops pulling the services layer.
```

Spot-check it yourself (path deps in the tree *are* the workspace/xlib
edges):

```bash
for c in oxidauth-permission oxidauth-kernel oxidauth-repository oxidauth-postgres \
         oxidauth-services oxidauth-http oxidauth-api oxidauth seedz; do
  deps=$(cargo tree -e normal -p "$c" --depth 1 | sed -nE 's/^[├└]── ([^ ]+) [^ ]+ \(.*\/src\/(.*)\)$/\2/p' | tr '\n' ' ')
  printf '%-22s -> %s\n' "$c" "${deps:-<none>}"
done
```

## Provider pattern

`oxidauth-api` wires the whole object graph through a type-erased
`provider::Provider` (xlib) — `init()` in
[`src/provider/mod.rs`](oxidauth-api/src/provider/mod.rs) runs two steps:

- **[`provider/postgres.rs`](oxidauth-api/src/provider/postgres.rs)** —
  `Database::from_env()` (xlib `postgres` `database!` macro: read + write
  pools), `ping()`, `migrate()` (honors `MIGRATIONS_ENABLED`), then
  `provider.store::<Database>(db)`.
- **[`provider/services.rs`](oxidauth-api/src/provider/services.rs)** —
  one block per use case: build the `Pg<Entity>Repository` instances from the
  stored `Database`, construct the services `UseCase`, and
  `provider.store::<XService>(...)` under the **kernel trait alias**
  (`pub type XService = Arc<dyn XServiceTrait>`). Handlers only ever know
  the kernel trait: `provider.fetch_unchecked::<CreateUserService>()` then
  the **named method** — `service.create_user(&params.user).await`. The
  generic `Service::call` dispatch is deprecated (gone at 2.0); named
  methods are the pattern to copy.

## Crates

| Crate                       | Purpose                                                                          |
|-----------------------------|-----------------------------------------------------------------------------------|
| `oxidauth-kernel`           | domain types, named `<Op>ServiceTrait`s, errors, JWT/crypto helpers                |
| `oxidauth-permission`       | permission-token parse/validate (`oxidauth:**:**` style); kernel depends on it    |
| `oxidauth-repository`       | query traits, one per operation (`InsertUserQuery`, …)                            |
| `oxidauth-postgres`         | `Pg<Entity>Repository` impls, `.sql` files per query, sqlx migrations             |
| `oxidauth-services`         | `<Op>UseCase`s implementing kernel traits over query traits (pre-0.9 "usecases" crate) |
| `oxidauth-http`             | DTO-only wire crate: `XxxReq`/`XxxRes` per endpoint + `Response` envelope         |
| `oxidauth-api`              | axum server, handlers, middleware, provider wiring, `build/` image script         |
| `oxidauth-rs`               | the **published client, package name `oxidauth`** (`OxidAuthClient`, axum extractors, mock feature) |
| `oxidauth-cli`              | stub — kept for a future CLI                                                      |
| `oxidauth-import-export`    | stub — kept for a future feature                                                  |
| `helm/`                     | chart (api only; `web.enabled: false`) — see [`helm/README.md`](helm/README.md)   |
| `hurl/`                     | live-API test suite — see below                                                   |

## Adding an entity

The full chain, using the users slice as the worked example:

1. **kernel type + service trait** — `oxidauth-kernel/src/users/create_user.rs`:
   the domain params (`CreateUser`), `CreateUserServiceTrait` with the named
   method, and `pub type CreateUserService = Arc<dyn CreateUserServiceTrait>`.
2. **repository trait** — `oxidauth-repository/src/users/insert_user.rs`:
   one operation trait (`InsertUserQuery`). These operation traits currently
   ride the deprecated kernel `Service<&Params>` shim — grandfathered, do
   **not** extend that style to new service traits; named methods only.
3. **postgres** — `oxidauth-postgres/src/users/insert_user/`: a directory
   per operation holding `mod.rs` (`impl` the operation trait for
   `PgUserRepository`, built by `PgUserRepository::new(db)`) and the query's
   `.sql` file next to it. Migrations live in `oxidauth-postgres/migrations/`.
4. **services** — `oxidauth-services/src/users/create_user.rs`:
   `CreateUserUseCase<T: InsertUserQuery>` implementing
   `CreateUserServiceTrait` — generic over the query trait, never over
   postgres.
5. **provider wiring** — add the block to
   `oxidauth-api/src/provider/services.rs`: repositories from `db.clone()` →
   `UseCase::new(...)` → `store::<CreateUserService>(...)`.
6. **api handler + route** — `oxidauth-api/src/server/api/v1/users/create_user.rs`:
   extract `CreateUserReq` from `oxidauth_http::users::create_user`, check
   `PERMISSION` via `oxidauth-permission`, fetch the kernel trait, call the
   named method, wrap the result in `Response<CreateUserRes>`.
7. **DTO** — in the `oxidauth-http` crate, `src/users/create_user.rs`: `CreateUserReq` /
   `CreateUserRes` (shared wire types — the server's handlers and the
   client both import these; the server no longer owns DTO definitions).
8. **client method** — `oxidauth-rs/src/client/users/create_user.rs`: a
   `CreateUserTrait` with `create_user<T: Into<CreateUserReq>>(..)` +
   `impl ... for Client` (POST + `handle_response`) and the mirror impl for
   `ClientMock` behind `feature = "mock"`.

Then wire the route into the module's `mod.rs`, and add a `hurl/tests/`
coverage file.

## Development

Everything runs from the **repo root** — this stack's compose file is pulled
in by the root `docker-compose.yml` (`include:`), so `docker compose up -d`
there starts postgres (:5434) + api (ephemeral host port). Run from the root:
`.env`, `bin/hurl.sh`, `bin/unit_test.sh`, `bin/database_test.sh`.

Run the server natively (outside the watchexec dev container):

```bash
set -a; source ../../.env; set +a      # DATABASE_URL etc., literal values
PORT=8080 cargo run --bin oxidauth-api
```

### hurl suite

`src/oxidauth/hurl.sh` (invoked by root `bin/hurl.sh`) runs the 17 files in
`src/oxidauth/hurl/tests/`.

---

**License**: GPL-3.0
