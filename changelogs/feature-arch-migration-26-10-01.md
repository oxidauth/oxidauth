# feature/arch-migration — consolidated changelog (delta vs `main`)

Single consolidated changelog for branch `feature/arch-migration` against `main`
(merge-base `0891252` v0.8.0): 1,280 files changed, +64,200 / -21,754.
Replaces the 46 per-PR changelog files this branch added to `changelogs/`;
main's pre-existing entries are untouched.

## Breaking changes roll-up

- **90000004** — `SudoUserBootstrapUseCase::new` takes xlib `provider::Provider` instead of `oxidauth_kernel::provider::Provider`; kernel Provider deprecated (`since = "0.5.0"`).
- **90000007** — server crate `oxidauth-http` renamed to `oxidauth-api`; new DTO crate `oxidauth-http` owns all wire DTOs and every `oxidauth_http::server::api::v1::` path drops that prefix (no back-compat re-exports); server `src/response.rs` deleted, `oxidauth_http::Response` is now the xlib envelope; error responses honor real status codes (401/400 where they used to answer 200); health routes renamed `__meta/healthcheck` / `__meta/livecheck` (old paths aliased for one release) and now return `HealthcheckRes`/`LivecheckRes` envelopes.
- **90000008** — internal API: `oxidauth-postgres` no longer implements any query service on the `Database` type; per-entity `Pg<Entity>Repository` structs replace them.
- **90000009** — crate rename `oxidauth-usecases` → `oxidauth-services`; `oxidauth_kernel::service::Service` deprecated, the 68 `XService` aliases become named-method `XServiceTrait`s (same alias names, `call(&x)` dispatch retired).
- **90000014** — every crate moves to edition 2024, MSRV 1.85 pinned by `rust-toolchain.toml`, workspace unifies on 0.9.0, tower-http 0.5 → 0.6.
- **OXA-000005** — anonymous callers of `forgot_password` previously received the target user's live TOTP code and triggered a refresh-token wipe; now 401. Callers need the new `oxidauth:auth:forgot_password` permission — upgrading deployments must seed/grant it (bootstrap only seeds on empty DB).
- **OXA-000008** — `Password` no longer implements `serde::Serialize`; consumers serializing it stop compiling.
- **OXA-000009** — `Disabled` accounts are now refused at login, refresh, 2FA validation, and password recovery (they previously passed all four); check `SELECT count(*) FROM users WHERE status = 'disabled'` before deploy. Invitation accept ignores client-supplied `status`.
- **OXA-000013** — `DELETE /v1/users/{user_id}` for a user with a pending invitation returns 400 (SQLSTATE 23503); revoke the invitation first.
- **OXA-000021** — error envelope on five routes changes from raw sqlx `"RowNotFound"` to domain `"authority not found by client_key: <uuid>"`.
- **OXA-000023** — zero-match refresh-token deletion is now success (was 400 `"database row not found"`); dead kernel alias `DeleteRefreshTokenByUserIdService` deleted.
- **OXA-000024** — Rust path `oxidauth_postgres::public_keys::select_public_key_by_user_id` removed; repoint to `select_public_key_by_id`.
- **OXA-000027** — SDK `Client::update_user`/`Client::update_role` now send PUT (were POST); mock-based consumers must update mounts.
- **OXA-000028** — `get_jwt()` returns the fresh token after refresh instead of the stale one.
- **OXA-000032** — `ClientErrorKind::EmptyPayload` method-label fields corrected (`create_invitation`, `find_invitation`, `list_all_authorities`).
- **OXA-000033** — `TotpSettings` serializes snake_case tokens (`"disabled"`, `{"enabled":{…}}`); PascalCase decode aliases kept; settings rows migrated in place.
- **OXA-000039** — `PermissionsResponse.permissions` and the JWT entitlements claim now carry one entry per distinct permission (was one per grant path).
- **OXA-000048** — status-parse error text changes from `failed to parse user_kind, unknown:` to `failed to parse user_status, unknown:`; repoint log-based alerts.
- **OXA-000069** — `oxidauth-kernel/src/service.rs` deleted (`Service`, `Layer`, `CanLayer`, `CanService`, `CanError`, `ExtractPermissions`).
- **OXA-000071** — kernel `Provider` deleted; the xlib `provider` crate is the only provider. Both removals ride unpublished `oxidauth-kernel` 0.9.0.

## Architecture & infrastructure migration (plans 00-16)

- [90000000](https://www.pivotaltracker.com/story/show/90000000) - fix wildcard permission matching
    - Fixed `validate_single` calling `compare` with granted/challenge arguments swapped, breaking all wildcard grants (`**:**:**`, `oxidauth:**:read`) — `CanService` returned `CanError::Unauthorized` for every superuser
    - Restored `Password`'s `******` debug mask (previously leaked password length into every tracing span)
    - Added `macros` and `rt` features to `oxidauth-kernel`'s `tokio` dep so `cargo test -p oxidauth-kernel` works standalone
    - No signature changes in `oxidauth-permission` or `oxidauth-kernel`

- [90000001](https://www.pivotaltracker.com/story/show/90000001) - Project root scaffolding
    - Replaced `.gitignore` with project-template set (target/, dist/, *.env with example.env exception, IDE/OS, helm values, *.log), kept tmp/**
    - Added crypt-keeper.toml (team = "freshbrewlabs") and secrets/.gitkeep
    - Added unified env contract: General/Database/Oxidauth sections, Postgres on 5434, DATABASE_URL/READ_DATABASE_URL, MIGRATIONS_ENABLED, password pepper, image v1.98.0
    - Added devops/postgres/init.sh and devops/helm values-staging.yaml / values-production.yaml from project-template
    - Added root README.md skeleton (8-layer architecture, getting started, available stacks)
    - Moved SECURITY_REPORT.md to docs/

- [90000002](https://www.pivotaltracker.com/story/show/90000002) - Move crates into src/ layout
    - git mv'd oxidauth-{kernel,repository,postgres,usecases,http,rs,permission,cli,import-export} into src/oxidauth/ (crate names unchanged; stub crates oxidauth-cli + oxidauth-import-export kept)
    - git mv'd oxidauth-http/hurl -> src/oxidauth/hurl and oxidauth-seed -> src/seedz
    - oxidauth-telemetry stays at repo root until plan 06; now an explicit workspace member since it cannot match the src/oxidauth/* glob
    - Root Cargo.toml: members = ["src/oxidauth/*", "src/seedz", "oxidauth-telemetry"], exclude = ["src/oxidauth/hurl", "src/oxidauth/helm"], resolver = "2"
    - Repointed oxidauth-http's oxidauth-telemetry path dep to ../../../oxidauth-telemetry; updated bin/{hurl-tests.sh,reset-db.sh,publish.sh,build-server.sh} paths

- [90000003](https://www.pivotaltracker.com/story/show/90000003) - Vendor xlib crates from project-template
    - cp -R'd src/xlib/{http,postgres,provider,telemetry} from project-template src/xlib (11 files, byte-identical except doc-header and provider self-dep removal)
    - Each crate's src/lib.rs gained header: `//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.`
    - src/xlib/provider decoupled from oxidauth: removed self-dependency loop, FromRef impl, and use statements; Provider + ProviderError and all 3 provider unit tests kept verbatim
    - Root Cargo.toml: added "src/xlib/*" to members (first entry)
    - Nothing consumes xlib yet; take() gets adopted server-side in plan 04

- [90000004](https://www.pivotaltracker.com/story/show/90000004) - Provider wiring: xlib provider + provider/ split
    - oxidauth-http now builds its dependency container from vendored xlib provider::Provider; 637-line provider::setup() becomes template triad provider/{mod.rs, postgres.rs, services.rs} with init() -> postgres::init() (connect, ping, migrate, store Database) -> services::init()
    - Handler/middleware use provider.fetch_unchecked::<T>() with panic-on-missing semantics identical to old kernel fetch; crate::provider::Provider re-export keeps handler imports unchanged
    - BREAKING (oxidauth-usecases consumers): SudoUserBootstrapUseCase::new now takes &provider::Provider (xlib crate) instead of oxidauth_kernel::provider::Provider
    - oxidauth_kernel::provider::Provider marked #[deprecated(since = "0.5.0", note = "use the provider crate (xlib)")] and stays until coordinated 2.0 of published crates

- [90000005](https://www.pivotaltracker.com/story/show/90000005) - oxidauth-postgres adopts the xlib `database!` macro
    - oxidauth-postgres/src/lib.rs now declares pub const MIGRATOR = sqlx::migrate!("./migrations") and calls postgres::database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, MIGRATOR), gaining read/write pool separation, MIGRATIONS_ENABLED gating, PingTrait, DatabaseBuilder, mock support, and test-facing MIGRATOR
    - New path dep: postgres = { version = "0.1.0", path = "../../xlib/postgres" }
    - Pool-access sweep: 33 modules use self.write_pool() for insert/update/delete/upsert/transactional, 29 modules use self.read_pool() for select_*; self.pool removed entirely
    - BEHAVIOR CHANGE: migrate() is now gated on MIGRATIONS_ENABLED — missing MIGRATIONS_ENABLED env var is a hard PgError::MissingEnvVar fail-fast at boot. Every deployment MUST set MIGRATIONS_ENABLED=true where app runs migrations (dev), MIGRATIONS_ENABLED=false where jobs/helm do (prod). READ_DATABASE_URL is optional, falls back to DATABASE_URL
    - Removed 47 no-op mod tests blocks (50 stub fns); no real #[sqlx::test] bodies existed to convert
    - oxidauth-api health.rs/live.rs import PingTrait for ping(); Database::from_env now returns Result<Database, PgError>

- [90000006](https://www.pivotaltracker.com/story/show/90000006) - oxidauth-http boots on the xlib `telemetry` crate; `oxidauth-telemetry` deleted
    - oxidauth-api (was oxidauth-http) gains telemetry = { version = "0.1.0", path = "../../xlib/telemetry" } dep; boot sequence: telemetry::get_logging_envs()? -> telemetry::get_subscriber(name, version, environment, env_filter, stdout) -> info!("starting oxidauth-http") -> telemetry::init_subscriber(...)
    - Removed direct deps: oxidauth-telemetry, tracing-subscriber, tracing-bunyan-formatter, tracing-log from oxidauth-http — they live in xlib telemetry now; tracing itself kept for info! macro
    - Deleted root oxidauth-telemetry/ crate and its explicit workspace-member line; no other crate, script, Dockerfile, or CI referenced it
    - BEHAVIOR CHANGE: ENVIRONMENT is now REQUIRED at boot — missing ENVIRONMENT is a hard fail-fast (telemetry::EnvVarError::Environment) before any provider work. Old crate needed no env and hardcoded level INFO. RUST_LOG now drives EnvFilter (module filters like RUST_LOG=oxidauth_http=debug honored); when unset it falls back to INFO (no error event — runs pre-subscriber, an eprintln! notice instead); corrupt non-UTF-8 value aborts boot

- [90000007](https://www.pivotaltracker.com/story/show/90000007) - split the
    - BREAKING (crate rename): src/oxidauth/oxidauth-http (package oxidauth-http) renamed src/oxidauth/oxidauth-api (package oxidauth-api, version 0.9.0, bin name follows package — cargo run --bin oxidauth-api). docker-compose.yml, bin/build-server.sh (binary, Cargo.toml path, Dockerfile path, image name), and crate's Dockerfile (/bin/oxidauth-api) follow. Telemetry service label and boot log now oxidauth-api.
    - BREAKING (DTO paths): new crate src/oxidauth/oxidauth-http (package oxidauth-http, version 0.9.0, GPL-3.0 + publish metadata, deps: oxidauth-kernel, xlib http, serde, serde_json, chrono, rust_decimal, url, uuid — no axum, no tokio) now owns every wire DTO. All oxidauth_http::server::api::v1::<domain>::<endpoint>::* paths drop the server::api::v1:: prefix: oxidauth_http::users::create_user::CreateUserReq. Mechanical rewrite: perl -pi -e 's/oxidauth_http::server::api::v1::/oxidauth_http::/g'. No back-compat re-exports (hard break, next major candidate).
    - BREAKING (Response): server crate's src/response.rs deleted; oxidauth_http::Response is a re-export of the xlib http::Response envelope (identical JSON keys: success/payload/errors/warnings/notices). oxidauth_http::response::Response remains as a #[deprecated] shim. Old crate type's no-arg Response::fail() (= BAD_REQUEST envelope) has no same-signature twin on the xlib type; every call site became Response::bad_request() — byte-identical body and 400 status.
    - BREAKING (status honesty): xlib's IntoResponse honors the stored status, so wire-visible deltas: Response::unauthorized() now answers 401 (was 200 with success:false), an error-free Response::bad_request() now answers 400 (was 200), and .status(…) now actually applies. Envelope JSON keys unchanged.
    - BREAKING (healthcheck body): GET /api/v1/__meta/health_check / live_check renamed to __meta/healthcheck / __meta/livecheck (old paths registered as aliases for one release, #[deprecated] note in the api crate README). Responses replace bare status codes with oxidauth_http::__meta::healthcheck::HealthcheckRes { version, healthy } / __meta::livecheck::LivecheckRes { version, healthy } inside the standard Response envelope: healthy = db.ping().is_ok() (200 healthy / 500 db-down, as before), livecheck is always 200 healthy: true (no db ping).
    - Moved inventory: 56 DTO leaf modules (pub structs */Req/*Res/*Body* and pub type Req/Res aliases, 1:1 with the endpoint leaf file names); private type */Req = kernel aliases (DeleteAuthorityReq, FindAuthorityByIdReq) and the oauth2 PathParams path extractor stay in oxidauth-api; kernel param structs (e.g. CreateUser, ListAllUsers) stay in oxidauth-kernel — handler files keep their pub use oxidauth_kernel::…* re-exports, handle(), PERMISSION consts, and routers only
    - oxidauth-rs: imports rewritten to the new DTO paths; kernel-owned types previously reached through server glob paths (CreateAuthority, UpdateAuthority, CreateRole, UserKind, AuthorityStrategy, AcceptInvitationParams, AcceptInvitationUserParams, CreateInvitationParams) now come from oxidauth_kernel directly; prelude::parse_and_validate re-exports oxidauth_permission instead of the server crate. axum/axum-extra now optional behind the server feature (which gates the axum::extract::FromRef integration module and stays in default); tokio narrowed from full to sync + macros + rt-multi-thread (client uses tokio::sync, test_client bin needs the runtime). oxidauth-usecases stays — the client re-exports the strategy params (oxidauth_usecases::auth::strategies) as public API; the DTO split does not make that dep unnecessary.

- [90000008](https://www.pivotaltracker.com/story/show/90000008) - postgres:
    - BREAKING (internal API): oxidauth-postgres no longer implements any query service on the Database type. grep -rn 'impl.*for Database' -> 0. Database keeps only its database!-macro surface (from_env, pools, migrate, PingTrait).
    - Each of the 16 entity modules (users, authorities, user_authorities, roles, role_role_grants, role_permission_grants, permissions, user_permission_grants, user_role_grants, refresh_tokens, public_keys, private_keys, totp_secrets, invitations, settings, auth) now declares pub struct Pg<Entity>Repository { db: Database } + new(db) (#[derive(Debug, Clone)]): PgUserRepository, PgAuthorityRepository, PgUserAuthorityRepository, PgRoleRepository, PgRoleRoleGrantRepository, PgRolePermissionGrantRepository, PgPermissionRepository, PgUserPermissionGrantRepository, PgUserRoleGrantRepository, PgRefreshTokenRepository, PgPublicKeyRepository, PgPrivateKeyRepository, PgTotpSecretRepository, PgInvitationRepository, PgSettingRepository, PgAuthRepository.
    - All 62 query modules re-point their impl receivers from Database to the entity struct: impl Service<&X> for Database -> impl Service<&X> for Pg<Entity>Repository (and the method-form traits InsertUserAuthorityQuery, UpdatePermission, InsertTotpSecretsQuery, SelectWhereNoTotpSecretByAuthorityIdQuery -> impl <Trait> for Pg<Entity>Repository). 62 old Database impls deleted == 62 new struct impls; the oxidauth-repository traits (blanket …Query markers over Service<…> plus the four method-form traits) remain untouched.
    - Pool access: every self.write_pool()/self.read_pool() became self.db.write_pool()/self.db.read_pool(); the per-file write/read split from plan 05 is preserved exactly.
    - Provider wiring (oxidauth-api/src/provider/services.rs): every use-case arg that was a raw db.clone() is now Pg<Entity>Repository::new(db.clone()) mapped to the trait each parameter requires (AuthenticateUseCase's 7, RegisterUseCase's 6, ExchangeRefreshTokenUseCase's 7, etc.). Store order and count unchanged: 59 provider.store::<…> lines before and after; 105 db.clone() occurrences before and after, now all inside 105 Pg<Entity>Repository::new(...) calls.

- [90000009](https://www.pivotaltracker.com/story/show/90000009) - rename
    - BREAKING (public API, for 2.0): oxidauth_kernel::service::Service is #[deprecated(since = "0.5.0")] — the generic service.call(&x) dispatch is retired. Every pub type XService = Arc<dyn for<'a> Service<&'a X, Response = R, Error = BoxedError>> alias (68 of them) is redefined in place, same alias name as #[async_trait] pub trait XServiceTrait: Send + Sync + 'static { async fn x_method(&self, params: &X) -> Result<R, BoxedError>; } + pub type XService = Arc<dyn XServiceTrait>. Aliases are NOT deprecated; consumers keep the same type names but call named methods (create_user, authenticate, fetch_setting, …). Traits stay in the kernel (review decision — deliberate deviation from the template, which puts them in the services crate).
    - BREAKING: crate rename oxidauth-usecases -> oxidauth-services (src/oxidauth/oxidauth-usecases -> src/oxidauth/oxidauth-services, package oxidauth-services); reverse deps flipped: oxidauth-api (provider wiring, bootstrap in main.rs) and oxidauth-rs (keeps the dependency; its pub use oxidauth_usecases::… re-export surfaces now read oxidauth_services::…) and export the identical items — the auth::strategies::* glob and username_password::registrar::UsernamePasswordRegisterParams are unchanged in the crate. oxidauth-services drops tracing-bunyan-formatter (logging is an api-layer concern).
    - oxidauth-services: every impl Service<&X> for XUseCase -> impl XServiceTrait for XUseCase (61 impls), bodies verbatim; the leading &'a lifetime and type Response/Error lines are gone; every trait impl's #[tracing::instrument] renamed to the template convention XUseCase::x_method. Service-typed fields (e.g. accept_invitation's UpdateUserService) call named methods now; repository (plan-08 *Query) fields intentionally keep .call. SudoUserBootstrapUseCase implements BootstrapServiceTrait::bootstrap and its 19 provider-fetched service calls are named.
    - oxidauth-api: all 58 handler call sites in src/server/** moved from .call(&x) to the named method (plus 1 in middleware/permission_extractor.rs and 1 in main.rs); grep '\.call(&' src/oxidauth/oxidauth-api/src -> 0. Provider store lines keep the unchanged XService alias names (58 provider.store::<…> blocks — the reviewer-flagged duplicate CreateRolePermissionGrantService store block from plan 08 is deleted, 59 -> 58). CanLayer, CanService, CanError, ExtractPermissions copied into oxidauth-api/src/middleware/can.rs (api-local, incl. unit tests); they compose over the still-living kernel Layer trait and reference the deprecated Service under a local #[allow(deprecated)]. oxidauth-api gains async-trait.
    - oxidauth-kernel: #[deprecated] on service::Service (plus CanLayer/CanService); Layer, CanError, ExtractPermissions and all XService aliases stay live. The old blanket users::create_user::CreateUserTrait (marker over Service<&CreateUser>) is deleted — superseded by CreateUserServiceTrait; its only consumer (invitations::create_invitation bound) migrated. totp_secrets::create_totp_secrets_by_authority_id was already in the target shape (CreateTotpSecretsTrait) and keeps its pre-existing published trait name.
    - Retained Service dependents (2.0/plan-14 scope, untouched per review): oxidauth-postgres keeps its 58 impl Service<&X> for Pg<Entity>Repository blocks and oxidauth-repository keeps its plan-08 trait XQuery: for<'a> Service<&'a X, …> supertrait blanket traits (94 naming lines) — the query-side removal is plan 14's story.

- [90000010](https://www.pivotaltracker.com/story/show/90000010) - compose
    - The stack owns its dev compose (src/oxidauth/docker-compose.yml); the root docker-compose.yml shrinks to an include: of it plus the shared shared-vol declaration (include semantics share it — defined exactly once).
    - dev.Dockerfile deleted. The api service now builds devops/docker/oxidauth-dev/Dockerfile (tagged oxidauth-dev:local) FROM registry.vizerapp.cloud/lib/rust-dev:$RUST_DEV_IMAGE_VERSION — the hand-rolled dev image only added watchexec-cli, which is already baked into rust-dev; the overlay adds exactly the two missing dev CLIs: sqlx-cli (pinned to the workspace sqlx 0.8.6, --no-default-features --features postgres,rustls — 0.8's migrator is built-in, no migrate feature) for bin/reset-db.sh, and hurl (official GitHub release binary, arch-selected via TARGETARCH, 8.0.1 — upstream dropped static-musl assets, gnu builds ship instead) for the plan-11 hurl suite.
    - Postgres provisioning via devops/postgres/init.sh mounted at /docker-entrypoint-initdb.d/init.sh (ro) with SERVICES=oxidauth: the container boots with POSTGRES_DB = postgres (the superuser role/db the image creates) and init.sh creates database + role oxidauth (password oxidauth) — the role is owner of its database with CREATEDB, which is what makes bin/reset-db.sh (sqlx drop/create) work with plain .env credentials; the old implicit flow got this because docker-postgres makes POSTGRES_USER a superuser. Multi-service local stacks (oxidauth + admin side by side) therefore get one provisioning path. The init script only runs on an empty data volume — switching to it needs docker compose down -v once.
    - Conventions kept: postgres host port 5434 (.env URLs point at 127.0.0.1:5434), cargo target dir on named volume shared-vol (CARGO_TARGET_DIR=/home/rust/shared_target, survives up/down), repo mounted at /home/rust/src/oxidauth, api on ephemeral host port 80 (docker compose port oxidauth-api 80) with the api.oxidauth.localhost network alias + VIRTUAL_HOST for local proxy routing. env_file: ../../.env — both DATABASE_URL and READ_DATABASE_URL (…@postgres:5432/oxidauth) are service-env overrides of the host URLs in .env: with the plan-05 read/write pool split, leaving the read URL pointing at 127.0.0.1:5434 makes the in-container read pool hang and the boot dies with PoolTimedOut.
    - DOCKER_PLATFORM (.env/example.env) now defaults to empty = host-native platform (compose omits platform:), replacing the pinned linux/amd64; pin it only to force cross-arch emulation.
    - bin/reset-db.sh no longer hardcodes DATABASE_URL=…@127.0.0.1:5432; it sources the repo-root .env (set -a; source …; set +a) and keeps the sqlx drop/create/migrate flow against the mapped 5434 port. Stop oxidauth-api first (docker compose stop oxidauth-api) — sqlx cannot drop the database while the api pool holds connections.

- [90000011](https://www.pivotaltracker.com/story/show/90000011) - bin/
    - Deterministic local auth: .env gains OXIDAUTH_DEFAULT_CLIENT_KEY (UUID) + OXIDAUTH_DEFAULT_ADMIN_PASSWORD, the env pins the bootstrap seeds (authority client_key + oxidauth:admin password; random when unset). example.env documents them as commented-out template lines. With them set, a fresh bin/reset-db.sh + docker compose up -d boots a stack the hurl suite can log into without scraping the boot log.
    - Root bin/ now follows the template: build.sh (finds build.sh under ./src; REGISTRY registry.vizerapp.cloud/oxidauth, rust v1.89.0, debian 12.12 — finds nothing until plan 12 ships oxidauth-api/build/build.sh), unit_test.sh (cargo test --workspace --exclude *-postgres --exclude postgres = zero-database), database_test.sh / hurl.sh (find-exec loops -> src/oxidauth/*.sh), plus parkinglot version.sh (cargo set-version --workspace wrapper: show / --dry-run / apply — bin/version.sh show prints today's divergent set) and crate_version.sh (one crate's version via cargo metadata).
    - Deleted bin/hurl-tests.sh (-> the hurl.sh pair), bin/build-server.sh (cross-build + docker buildx multi-arch logic moves to plan 12; its oxidauth-builder builder-name pattern is preserved in the plan-11 Execution note) and bin/cargo-watch.sh (the dev loop is compose watchexec now). README.md example updated; the repo has no .github/workflows, so nothing referenced the removed names.
    - bin/publish.sh republished for the new tree: topological order permission -> kernel -> repository -> postgres -> services -> http -> rs -> api (oxidauth-rs publishes as crate oxidauth; permission is the only root — the kernel path-depends on it), versions read via bin/crate_version.sh, and a final semver git tag v<oxidauth-api version> — gated on a clean working tree (cargo publishes the tree; a dirty run could not honestly tag HEAD).
    - src/oxidauth/hurl.sh replaces bin/hurl-tests.sh: resolves the target dynamically — OXIDAUTH_HURL_HOST/_PORT/_SCHEME override, else docker compose port oxidauth-api 80 (the stack publishes on an ephemeral host port) — injects host/scheme/stamp/api_version plus the login pair admin_client_key/admin_password (read from the gitignored .env, never committed to hurl/variables-local) over the variables file, runs public_keys_create.hurl then the suite twice with one shared stamp so pass 2 only passes if pass 1's deletes stuck (the old cleanup-verification trick).
    - src/oxidauth/database_test.sh sources .env, sets MIGRATIONS_ENABLED=true and runs cargo test -p oxidauth-postgres -p postgres -- --nocapture.
    - The hurl suite (17 files) was rewritten whole against the post-plan-07 contract: every guarded call sends Authorization: Bearer {{jwt}} from a per-file POST /auth/authenticate with the pinned credentials (the old files were pre-auth and used the removed strategy register body); statuses match the handlers, not wishes — bare 401 from the jwt extractor (no/foreign token), 400 envelope + RowNotFound/PermissionNotFoundError/SettingNotFoundError/Strategy(Oauth2)/duplicate key debug strings for service errors, 422 for deserializer rejections (missing jwt_nbf_offset, bad uuid), count == 0 list assertions replaced with seeded-state assertions, grant payloads pinned to the {permission|role|child,grant} / {user_role|user_permission} shapes, and the authority client_key regeneration on update asserted as-is. New tests/healthcheck.hurl + tests/livecheck.hurl assert the plan-07 {success, payload:{version, healthy}} envelope with version cross-checked against bin/crate_version.sh oxidauth-api (placed in tests/ so the hurl/tests/*.hurl glob actually runs them). invitations/totp/oauth2 stay uncovered, as before — no honest-assertable flow existed for them pre-migration either.

- [90000012](https://www.pivotaltracker.com/story/show/90000012) - Helm chart +
    - src/oxidauth/helm/ from the stack-template chart with {{project-name}} -> oxidauth; _helpers.tpl keeps the oxidauth.* helper names.
    - Template rework: api image/port moved under api.* (api.image.repository/tag, api.containerPort: 80); service.targetPort follows api.containerPort. The server binds $PORT — chart env.PORT pins "80" so the bind and containerPort agree. The shipped service 80 -> 8080 mismatch is gone. Web half DISABLED, NOT deleted: web.enabled: false gates web-deployment/-service/-ingress.yaml (the flag did not exist in the template); a future admin-UI stack re-enables without chart surgery. web-deployment image ref fixed to web.image.* (the template pointed it at the api image). configmap.yaml now ranges over .Values.env — the template rendered only ENVIRONMENT/RUST_LOG while the Deployment configMapKeyRefs EVERY env key, so any extra key (DATABASE_URL…) booted pods against dangling keyRefs. ingress.yaml honors ingress.enabled (the value existed, the gate did not). env defaults: MIGRATIONS_ENABLED: "false" in-cluster (xlib migrate() treats any non-"true" as a skip — the hard error is a MISSING var — so boot is a no-op-migrate, verified in src/xlib/postgres/src/lib.rs; migrations run through sqlx-cli/bin/reset-db.sh), MIGRATIONS_PATH -> /etc/oxidauth/oxidauth-api/migrations (where the image copies them), and OXIDAUTH_USERNAME_PASSWORD_PEPPER DELIBERATELY absent from chart defaults — an empty default would silently hash with an empty pepper; only the encrypted env values set it.
    - oxidauth-api/build/{Dockerfile,build.sh} (multi-stage rust-base -> debian, --release --bin oxidauth-api, migrations dir copied to /etc/oxidauth/oxidauth-api/migrations for in-container sqlx-cli despite migrate! embedding, EXPOSE 80); the pre-split oxidauth-api/Dockerfile (single-stage tmp/$TARGETPLATFORM stager from the deleted bin/build-server.sh) is DELETED — no live refs. Root .dockerignore added (workspace-root build context must not drag target/, .env, secrets, .enc). build.sh resolves the repo root from its own location (runs from anywhere), tags semver + :latest + git-sha (pair preserved from the old build-server.sh), reuses the oxidauth-builder docker-container builder, and defaults to a DRY multi-arch build (--output type=oci tarball — the docker-container driver cannot export a manifest list to the local store): pushing to registry.vizerapp.cloud/oxidauth requires PUSH=1, an explicit human gate.
    - bin/deploy.sh + bin/uninstall.sh (parkinglot pattern): namespace $ENV-oxidauth, release $ENV-oxidauth-oxidauth, values layered devops-first (-f ../../../devops/helm/values-$ENV.yaml -f values-$ENV.yaml from src/oxidauth/helm), production confirmation prompt verbatim. kube context = fbl-k3s (the plan's freshbrewlabs placeholder): every freshbrewlabs project deploys to that k3s context — parkinglot/bin/deploy.sh proves it. Deploy itself remains a human gate; uninstall.sh also pins the context so a stale current-context can't aim the uninstall at the wrong cluster.
    - devops/helm/values-{staging,production}.yaml SANITIZED — the plan-01 copies were byte-identical project-template output carrying another project's values (staging|production-statuswrangler namespaces, project: statuswrangler, a stray defaultReplicas the chart never reads, statuswrangler affinity-comment keys). Now: namespace: staging-oxidauth|production-oxidauth, api.image.repository: registry.vizerapp.cloud/oxidauth/oxidauth-api (tag = crate version), registryCredentials.project: oxidauth (drone-bot bot creds kept — org-wide registry bot, same credentials parkinglot deploys with, inside the encrypted file), env = RUST_LOG/ENVIRONMENT + DATABASE_URL and OXIDAUTH_USERNAME_PASSWORD_PEPPER as marked CHANGE_ME placeholders (real values unknown until the in-cluster DB exists). All four values files (devops + chart values-{staging,production}.yaml) encrypted with crypt-keeper (keybase, team freshbrewlabs) — .enc committed-form, plaintext gitignored per plan-01 .gitignore; round-trip decrypt verified.

- [90000013](https://www.pivotaltracker.com/story/show/90000013) - seedz:
    - src/seedz/ rewritten wholesale (the oxidauth-seed stub's add() body dies here): package seedz with [[bin]] name = "seedz". lib.rs is the template's project-level seeder API — Seeder trait (seed(&self, write: &PgPool, read: &PgPool)) + SeedRunner (add_seeder/run, sequential) — with BoxedError re-exported from oxidauth_kernel::error instead of the template's local definition.
    - main.rs: dev-only guard first — unless ENVIRONMENT=local, prints "seedz is for local development; refusing to run against ENVIRONMENT=<x>" and exits 1 before telemetry, before any DB handle (verified against production: exit 1, zero rows written). Then telemetry boot (xlib crate, like the api), oxidauth_postgres::Database::from_env() (DATABASE_URL + READ_DATABASE_URL), database.migrate() (plan-05 API: hard-fails on missing MIGRATIONS_ENABLED, skips unless "true"), then the fixtures seeder via SeedRunner. No hand-rolled pools; MIGRATOR rides the Database. Server boot untouched: oxidauth-api/src/main.rs and oxidauth-services/src/bootstrap/ carry no changes (bootstrap stays the provisioning path; seedz never runs from the server).
    - fixtures.rs — idempotent dev data via find-then-insert on each table's natural key (name / username / realm+resource+action / PK); every helper RESOLVES the existing row's live id and threads it into dependent grant inserts, so pre-existing rows — earlier seed runs OR rows created by hand/API under the same names (random UUIDs) — are reused, never duplicated and never an FK abort; deterministic f0000000-0000-4000-8000-0000000000XX UUIDs (parkinglot convention) on fresh inserts keep re-runs convergent. Inventory: authority local-dev (username_password, enabled, settings JSON mirroring what bootstrap's AuthoritySettings serializes, fixed dev password_salt, pinned client_key f0…02 for probes), users seedz:viewer / seedz:editor / seedz:auditor (kind human) attached to local-dev, covering the three grant combinations (role-only; role + direct demo:reports:export; direct-only demo:audit:read), roles seedz:viewer / seedz:editor over demo realm:resource:action permissions, and setting seedz:sample. No signing keys, no oxidauth:admin, no default authority — bootstrap owns those; no credentials either (passwords are argon2+pepper at registration, this seeder never fakes them).
    - Stack compose: one-shot seedz service, profiles: [seed], depends_on: postgres: {condition: service_healthy} (matches the api; PgPool::connect is eager with no retry, so even a one-shot must not race postgres warmup), in-cluster DB URLs, MIGRATIONS_ENABLED="true", ENVIRONMENT=local, shared-vol cargo cache; run with docker compose --profile seed run --rm seedz (repo root). Plan deviation: image is the locally-built oxidauth-dev:local (same build block as oxidauth-api, compose dedupes) instead of the raw rust-dev registry tag — plan 10 standardized this stack's dev image on the local build; plus CARGO_TARGET_DIR (the plan snippet mounted shared-vol but omitted the var; without it the mount is pointless).
    - Verified fresh-DB end to end: reset-db -> boot (bootstrap provisions, setting written) -> seedz x2 (second run: all-skip/reuse, row counts identical across users/roles/authorities/settings/permissions/grants) -> api restart logs "bootstrap already completed", authorities stay 2 (no duplicate) -> hurl suite green (1+17+17) on the seeded DB. Hand-created-collision run (role/permission made via psql with random UUIDs before first seed): seedz succeeds, grants point at the live rows, counts stable. cargo check -p seedz + fmt green.

- [90000014](https://www.pivotaltracker.com/story/show/90000014) - **BREAKING**
    - BREAKING: every crate moves to edition = "2024" (MSRV 1.85+ enforced by the new rust-toolchain.toml pin), all fifteen workspace members unify on 0.9.0, and tower-http crosses 0.5 -> 0.6. Consumers on old toolchains or pre-0.9 crate versions must move with it.
    - rust-toolchain.toml added: channel = "1.98.0" — the registry dev image's rustc (rust-dev:v1.98.0), which rustup installs and activates repo-wide (verified: rustc -V -> 1.98.0 under the pin). Templates ship no toolchain file; this repo adds one deliberately so CI, the image, and cargo fix/fmt lanes agree on the compiler the edition story assumes. rustfmt.toml already demanded edition/style 2024 — the crates have now caught up with the formatter instead of the other way round.
    - Edition sweep: cargo fix --edition --allow-dirty --all-targets --workspace run workspace-wide (toolchain 1.98.0) before the manifest flips; then all eleven edition = "2021" manifests set to "2024" (the four vendored xlib crates were already 2024). Post-switch residue: one unused import (a self-referencing glob in oxidauth-postgres/src/private_keys/select_most_recent_private_key/) removed; zero gen-keyword, RPIT-capture, or unsafe-attr-ordering fallout, exactly as the plan predicted. The tail_expr_drop_order future-incompat hit only the seedz/oxidauth-api main tails (sqlx pool temporaries now drop before function locals) — inspected: the earlier Drop only closes idle pool connections at process exit, behavior-safe.
    - Dependency alignment (manifest floors; lock = latest compatible). axum 0.8.9 — api at the template baseline (default-features = false, http1,json,macros,matched-path,original-uri,tower-log,query,tokio; the old api pile differed only by form/tracing, which no code uses: no Form< extractor anywhere), xlib/http keeps its HEAD feature shape; note the oxidauth client's server feature changed [] -> ["dep:axum","dep:axum-extra"] (wasm wiring) — --no-default-features consumers no longer see the axum extractor surface; tower-http 0.6 with cors only (the fs feature is gone: ServeDir/ServeFile grep = zero usages); sqlx 0.8.6 everywhere including seedz and xlib/postgres; tokio 1.53; uuid 1.26 / serde 1.0.229 / chrono 0.4.45 / async-trait 0.1.92 — each now exactly one version in Cargo.lock (uuid's double-lock is repaired); reqwest 0.12.28 (json+rustls-tls, no default features, unchanged shape); mockall stays 0.14; jsonwebtoken/rsa/rust_decimal/url bumped within their majors only — majors are the 2.0 decision, not this plan's.
    - Version unification: all fifteen members -> 0.9.0 (kernel/postgres/permission/services/seedz/cli/import-export 0.4/0.2 -> 0.9, repository 0.2.0 -> 0.9.0, api 0.8.0 -> 0.9.0, rs 0.4.0-rc2 -> 0.9.0, xlib crates included 0.1.0 -> 0.9.0 — the single-workspace-version story only holds if the vendored crates ride the same train; they are never published from here, so bumping them costs nothing and keeps bin/version.sh the one tool). bin/version.sh 0.9.0 itself refused to bootstrap: cargo-edit resolves the workspace before rewriting, and every cross-crate path-dep already pinned 0.9.0 (see below), so pre-bump members could not resolve. The script's own documented fallback — rewrite the package version lines directly, refresh the lock via cargo metadata — was applied; bin/version.sh (show mode) now prints the uniform 0.9.0 set.
    - Cross-crate path deps: uniformly { version = "0.9.0", path = ... }. Where the key existed (oxidauth-http's 0.9.0, xlib deps at 0.1.0) it was bumped; where members had path-only deps (kernel->permission, api->kernel/services/postgres/permission, repository/postgres/services/rs->members) the version key was added — a path-only dependency is stripped from the packaged manifest, so those crates could never publish (publish.sh's whole premise). The xlib path style (path+version) is the template's; it is now the whole repo's.
    - Publish metadata completed for the two bare manifests: oxidauth-repository and oxidauth-services gain license = "GPL-3.0", description/homepage/repository/readme, and the two missing README.md files (cargo package refuses without them). Kept per the plan: [profile.dev.package.num-bigint-dig] opt-level = 3, GPL-3.0 licenses, publish metadata; no [workspace.dependencies], no [workspace.lints] introduced (both template-prohibited).
    - wasm target wiring (the wasm feature has never compiled — see the Execution note gate table): uuid gained a js randomness backend and the oxidauth client a per-target tokio split (macros,rt,sync common, rt-multi-thread native-only; wasm rejects it at compile time), gloo-timers 0.3 -> 0.4. oxidauth-permission/oxidauth-http/oxidauth-kernel now check clean on wasm32-unknown-unknown; the client crate still does not, and no dependency bump can fix that: the #[async_trait] client/service traits box futures as Send, while reqwest on wasm routes through js_sys::futures (Rc<RefCell<..>>, never Send). Redesign (?Send traits or module-gating the reqwest client for wasm) belongs to the 2.0 plan alongside the deprecated-shim removal.

- [90000015](https://www.pivotaltracker.com/story/show/90000015) - docs + README refresh (migration plan 15, final)
    - Root README.md rewritten to the project-template shape: 8-layer table with a this-stack column (rs exists as the published oxidauth crate; no web layer), getting started (.env + the two OXIDAUTH_DEFAULT_* bootstrap pins, compose up, healthcheck curl on the ephemeral docker compose port mapping, seedz, hurl, unit tests), Available Stacks + the link-stack-to-project checklist, Development (1.98.0 toolchain pin, the cargo +nightly-2025-07-01 fmt --all convention, honest clippy deprecation note), the bin script table, and a 2.0 follow-up section (deprecated kernel shims, xlib publish names, the wasm Send wall) each pointing at its plan execution note.
    - New src/oxidauth/README.md: stack README with the layer diagram verified against cargo tree edges (services does NOT depend on postgres; rs pulls services only for strategy param types), the api Provider-pattern walkthrough (named-alias wiring), crates table, and the eight-step adding-entities chain (kernel type -> repository trait -> Pg repository + .sql -> services UseCase -> provider block -> api handler -> oxidauth-http DTO -> rs client method), plus the hurl suite pointer.
    - New docs/CLIENT_MIGRATION.md for 0.8->0.9 consumers (parkinglot): server::api::v1:: prefix dropped from DTO paths, response::Response -> root Response, oxidauth-usecases -> oxidauth-services (client package stays oxidauth), generic Service::call -> named methods, uniform 0.9.0 version floor.
    - docs/SECURITY_REPORT.md code paths refreshed to the 0.9 layout (findings/line numbers unchanged, provenance note added); rfcs/4-forgot-password-flow/rfc.md path references updated (paths only); docs/OAUTH.md example redirect URIs now name api.oxidauth.localhost; README gains a Security section linking the report. docs/AUTHORITIES.md and the other rfcs carry no stale paths.
    - changelogs/README.md now describes the migration arc and the 900000NN <-> plan NN mapping; docs/migration-plan/README.md gains the completion date and its pickup grouping note is now historical.

- [90000016](https://www.pivotaltracker.com/story/show/90000016) - helm chart
    - Post-review follow-up to migration plan 12 (found by the final security review of the migration arc, review approve 0-must-fix). Two remediations:
    - api env secrets moved out of the ConfigMap: New secret.yaml renders <fullname>-env (Opaque, stringData, helm.sh/resource-policy: keep) for the env keys produced by the new oxidauth.secretEnvKeys helper — default set DATABASE_URL, READ_DATABASE_URL, OXIDAUTH_USERNAME_PASSWORD_PEPPER, overridable wholesale via values.envSecretKeys. The helper returns a JSON OBJECT (helm 4's fromJson only decodes objects and the sprig list-membership form broke, so the templates use dict + hasKey). configmap.yaml renders the complement, deployment.yaml splits its env list secretKeyRef vs configMapKeyRef on the same predicate — Secret and ConfigMap together still cover exactly .Values.env, no dangling refs by construction. Only keys ACTUALLY present in .Values.env land in the Secret: the chart default ships just DATABASE_URL: "", and the pepper is never fabricated (still set only in the encrypted devops values). Staging/production renders verified: DATABASE_URL + OXIDAUTH_USERNAME_PASSWORD_PEPPER moved ConfigMap->Secret; non-secret keys (PORT/ENVIRONMENT/RUST_LOG/MIGRATIONS_*) byte-identical.
    - api ingress TLS + body-size posture: ingress.yaml now renders spec.tls from ingress.tls (standard {hosts, secretName} blocks), adds nginx.ingress.kubernetes.io/force-ssl-redirect: "true" when ingress.forceSsl, and the proxy-body-size default drops from "0" (UNLIMITED upload) to "1m". All three overridable via ingress.annotations (user map mergeOverwrites the chart defaults). Defaults stay render-compatible: forceSsl: false + tls: [] ship no TLS/redirect on plain chart installs. The encrypted devops staging/production values set forceSsl: true with a tls: [] placeholder + comment — the TLS Secret itself is cluster-side wiring (cert-manager issuer or a manually created Secret in the release namespace). All four values .enc re-encrypted via crypt-keeper (team freshbrewlabs), round-trip decrypt byte-compared.

## Kernel, HTTP API & security fixes (OXA tickets)

- [OXA-000005](https://www.pivotaltracker.com/story/show/OXA-000005) - forgot_password stops being an anonymous TOTP oracle: route gated with `oxidauth:auth:forgot_password`, failure paths blinded (Step 1 hotfix only)
    - Route `POST /api/v1/auth/username_password/forgot_password` now requires bearer auth with permission `oxidauth:auth:forgot_password` (ExtractJwt + ExtractEntitlements gate)
    - Anonymous callers receive 401 (no/invalid bearer) instead of leaking the user's live TOTP code and triggering refresh-token wipe
    - Authenticated but unpermitted callers receive 401 with `{"success":false}` gate envelope
    - Failure paths (unknown user, no TOTP secret, refresh-delete failure) now return generic `200 {"success":true}` with no error text, closing an existence/enrollment oracle
    - `oxidauth-services::bootstrap` gains `FORGOT_PASSWORD_PERMISSION = "oxidauth:auth:forgot_password"` and seeds it into the permissions tree (find-or-create, `first_or_create_permissions`) on empty DB
    - Upgrading deployments must add the permission string via permissions CRUD/migration and grant it to support roles
    - Breaking for callers that exploited or depended on the anonymous leak; SDK always sent a bearer and keeps working if credentials hold the new permission

- [OXA-000008](https://www.pivotaltracker.com/story/show/OXA-000008) - `Password` no longer serializes the raw secret, and `UpdatePasswordParams` masks every secret field in `Debug`
    - `oxidauth_kernel::Password` no longer implements `serde::Serialize`; only `Deserialize` survives
    - Bootstrap handoff rewritten: `bootstrap::first_or_register_user` builds explicit JSON from raw `String`; `UsernamePasswordRegisterParams::to_value()` and the struct's `Serialize` derive removed (struct now `Deserialize`-only)
    - `UpdatePasswordParams::Debug` now hand-written and constant-masked; `code`, `password`, `password_conf` render as `"******"` in all three tracing layers (api handler, service use case, SDK wrapper)
    - Breaking for external `oxidauth-kernel` consumers that serialized a `Password` (compilation error, no source consumers in workspace)
    - Log readers see masked `UpdatePasswordParams` shape where plaintext passwords used to appear in tracing output
    - No wire, DB, or API change; `user_authorities.params` still stores `{password_hash}` only

- [OXA-000009](https://www.pivotaltracker.com/story/show/OXA-000009) - `users.status` becomes enforceable: four credential gates refuse `disabled` accounts (login, refresh, 2FA validation, password recovery); the invitation flow stops honouring a client-supplied status
    - Account-disable control now enforced server-side: four credential gates refuse `Disabled` accounts (login, refresh, 2FA validation, password recovery)
    - `authenticate`: `SelectUserByIdQuery` moved out of `TotpSettings::Enabled` arm, runs on every login; disabled refused before signing key, permission tree, and refresh insert
    - `exchange_refresh_token`: one `SelectUserByIdQuery` per refresh rejects disabled after expired-token hygiene; the valid token row is NOT deleted or rotated
    - `totp/validate`: gate added after code check (wrong code still answers "invalid totp code"), before any JWT material
    - `username_password/update_password`: gate after user-authority lookup and before TOTP secret fetch; disabled account cannot spend/confirm a reset code
    - All refusals log `tracing::warn!` with `user_id` for auditability; every failure, including "account is disabled", still returns 200 `{success:false}`
    - `Invited` accounts keep authenticating and renewing (only `Disabled` fails closed)
    - `oxidauth_kernel::AcceptInvitationUserParams.status` field removed; `POST /api/v1/invitations/{invitation_id}` can no longer self-assign status (transitions belong to admin `update_user` only)
    - Breaking on deploy: `SELECT count(*) FROM users WHERE status = 'disabled'` first — those accounts logging in successfully today will start failing (login, refresh, 2FA validation, recovery)
    - Breaking for callers that relied on inviting with a preset status

- [OXA-000012](https://www.pivotaltracker.com/story/show/OXA-000012) - jwks listing skips malformed key rows instead of panicking
    - `list_all_public_keys` now skips malformed rows instead of panicking (previously took down all authenticated traffic)
    - Rows that are not standard base64 or whose decoded bytes are not UTF-8 are excluded and logged via `tracing::error!` carrying `key_id` and the decode error
    - `ExtractJwt` calls this listing on every bearer-token request before looking at the token; one bad row previously caused every protected endpoint to return connection reset + panic backtrace
    - With all rows bad, listing is empty and protected routes return normal 401
    - No API/wire format change: raw PEM stays, handler 400 shape and extractor 401 mapping untouched
    - No migration ships; operator guidance: delete or re-seed any row named by the error logs as `base64(PEM)`

- [OXA-000013](https://www.pivotaltracker.com/story/show/OXA-000013) - `invitations.user_id` gains the missing FK `invitations_users_fk ... ON DELETE RESTRICT` (register SEC-13)
    - `invitations.user_id` gains FK `invitations_users_fk ... ON DELETE RESTRICT` (SEC-13)
    - Column was `NOT NULL` and indexed but unconstrained, allowing invitations for deleted/unknown users that consumed on accept then failed on `user_authorities_users_fk`, surfacing unrecoverable 400
    - Storage migration: `20260930180000_add_invitations_users_fk.sql` first DELETEs dangling invitations, then adds the FK; existing `invitations_user_id` index retained for child-row check
    - RESTRICT chosen over CASCADE to prevent `DELETE /users/{id}` from silently revoking pending invitations (entitlement cross-leak against `oxidauth:invitations:delete`)
    - Breaking for user deletion: `DELETE /v1/users/{user_id}` for a user with a pending invitation now returns 400 (SQLSTATE 23503, `invitations_users_fk` in error payload)
    - Runbook: revoke the invitation first (`DELETE /v1/invitations/{id}`, permission `oxidauth:invitations:delete`), then delete the user
    - Ops: cleanup DELETE discards pre-existing dangling rows on upgrade
    - no API/DTO/Rust-surface change; violations surface through existing `BoxedError` path

- [OXA-000014](https://www.pivotaltracker.com/story/show/OXA-000014) - remove the dead `update_permission` vertical instead of fixing it (register DATA-1)
    - Deletes the dead `update_permission` module and its zero-caller trait from `oxidauth-repository`
    - Removed: `oxidauth-postgres/src/permissions/update_permission/` (`mod.rs`, `update_permission.sql`) and `oxidauth-repository/src/permissions/update_permission.rs`
    - Deleted public trait `UpdatePermission` plus `UpdatePermissionParams` and `UpdatePermissionError` — had zero in-repo callers (no kernel request, service use case, HTTP DTO, api route, SDK wrapper, or hurl test)
    - The SQL targeted the wrong table (`UPDATE authorities SET realm/resource/action`) so the code path could never succeed
    - No schema or migration change; no route/DTO/SDK surface to remove
    - Permissions remain create/find-by-parts/list/delete only; renaming stays DELETE + re-POST

- [OXA-000019](https://www.pivotaltracker.com/story/show/OXA-000019) - the JWT signing key is now picked deterministically: `ORDER BY created_at DESC, id DESC`, and key inserts stamp `statement_timestamp()`
    - JWT signing key selection now deterministic: `ORDER BY created_at DESC, id DESC` (total order on own)
    - `created_at` previously carried no uniqueness constraint, causing non-deterministic picks across replicas and plan changes
    - `insert_public_key.sql` changed from `NOW()` (transaction start, constant per batch) to `statement_timestamp()` (per-statement)
    - Within a batch, the semantically newest key now wins by `created_at` alone; ties fall back to greatest `id` (uuid byte-wise comparison)
    - App's `POST /api/v1/public_keys` path (one auto-committed INSERT per request) is behaviorally unchanged
    - Consumer-visible delta: previously ambiguous ties now resolve to greatest `id` for out-of-band rows
    - Ties already in production data are not repaired; tokens carry no `kid` so verification still accepts every served key

- [OXA-000021](https://www.pivotaltracker.com/story/show/OXA-000021) - unknown `client_key` reports the domain error, not raw `RowNotFound`
    - `select_authority_by_client_key` now uses `fetch_optional` instead of `fetch_one`, correctly returning `None` for missing keys instead of `sqlx::Error::RowNotFound`
    - Unknown `client_key` now reports domain error, not raw sqlx error
    - 400 error envelope on five endpoints changed: `POST /auth/authenticate`, `POST /auth/register`, `GET|POST /auth/oauth2/redirect`, `POST /totp/validate`, `POST /users/{user_id}/authorities`
    - Error now carries `display: "authority not found by client_key: <uuid>"` / `debug: "ClientKey(<uuid>)"` instead of sqlx text (`"database row not found"` / `"RowNotFound"`)
    - Consumers matching `"RowNotFound"` on these routes must repoint at the domain wording
    - Status codes do not move (400-not-404 convention holds)
    - No SQL/trait/DTO/wire change; only error body wording

- [OXA-000023](https://www.pivotaltracker.com/story/show/OXA-000023) - bulk refresh-token revocation reports every deleted row; zero tokens is success, not an error
    - `delete_refresh_token_by_user_id` now returns `Vec<RefreshToken>` instead of a single `RefreshToken` (fetch_one → fetch_all)
    - Bulk revocation now reports every deleted row; zero tokens is success with empty `Vec`, never an error
    - Zero-match users previously hard-errored with `sqlx::Error::RowNotFound`
    - `tracing::info!(count = deleted.len())` logs the deletion count for auditability
    - One consumer-visible delta: `POST /api/v1/auth/username_password/forgot_password` for a user with a valid TOTP secret and zero live refresh tokens now returns 200 + code instead of 400 `"database row not found"`
    - Breaking internal API: dead kernel alias `oxidauth_kernel::refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserIdService` deleted (zero implementors/consumers workspace-wide)
    - No SQL change, no wire/DTO/migration change

- [OXA-000024](https://www.pivotaltracker.com/story/show/OXA-000024) - **BREAKING (internal API)** `oxidauth-postgres`: misnamed module `select_public_key_by_user_id` renamed to `select_public_key_by_id`
    - `oxidauth-postgres::public_keys::select_public_key_by_user_id` module renamed to `select_public_key_by_id`
    - The module always queried by primary key (`WHERE id = $1`); `public_keys` has never had a `user_id` column
    - Pure rename: no SQL, trait, DTO, wire, or span change
    - Zero in-workspace consumers of the old path
    - Two `#[sqlx::test]`s pass byte-identical; stale `BUG(pinned)` misnomer comment deleted

- [OXA-000026](https://www.pivotaltracker.com/story/show/OXA-000026) - list-all responses are now deterministically ordered: every SELECT-all gained `ORDER BY created_at ASC, id ASC`
    - All five list-all repository queries now gain `ORDER BY created_at ASC, id ASC` for deterministic JSON array order
    - Affected routes: `GET /api/v1/users`, `/api/v1/authorities`, `/api/v1/public_keys`, `/api/v1/permissions`, `/api/v1/roles`
    - Queries: `select_all_users_query.sql`, `select_all_authorities.sql`, `select_all_public_keys.sql`, `select_all_permissions.sql`, `select_all_roles.sql`
    - `id` mandatory tie-break: `CURRENT_TIMESTAMP`/`NOW()` is transaction start time so bulk seeds tie on `created_at`
    - `updated_at` never used (mutable by unrelated UPDATEs)
    - Bootstrap: `public_keys.pop()` now deterministically picks the NEWEST signing key at boot (last row with ASC ordering)
    - No client may legitimately have depended on old non-order; no shape, route, type, or schema change
    - Deliberately NOT in scope: no pagination, no index, no service-layer sorting

- [OXA-000027](https://www.pivotaltracker.com/story/show/OXA-000027) - fix `update_user`/`update_role` SDK verb drift: POST → PUT (register CLI-1)
    - `Client::update_user` and `Client::update_role` SDK wrappers changed from POST to PUT
    - Previously sent POST to `/api/v1/users/{id}` and `/api/v1/roles/{id}` which the api mounts only as GET/PUT/DELETE
    - Every call failed with 405 against a live server; `Client::request` deserialized the reply unconditionally so callers saw `ClientErrorKind::Other("failed to deserialize response")` with the 405 and its `Allow` header swallowed
    - Both wrappers now send `.put(...)`; route, payload, bearer header and DTOs are byte-identical
    - Test-only break: contract test pins (`update_user_route_contract`, `update_role_route_contract`) and consumers that copied our `"POST"` must repoint at `PUT`
    - Repaired SDK updates now reach the handler, including OXA-000015's `update_user` null-overwrite fix
    - No public API/DTO/signature change server-side

- [OXA-000028](https://www.pivotaltracker.com/story/show/OXA-000028) - fix stale `get_jwt()` after refresh: `Client::refresh` now writes `state.raw_jwt` (register CLI-2)
    - `Client::refresh` now writes `state.raw_jwt` on validated success, fixing stale token returned by `get_jwt()`
    - Previously `state.jwt`, `state.refresh_token`, and the bearer client were updated but `state.raw_jwt` was never written
    - `get_jwt()` kept handing out the expired token string after every successful refresh, while `get_jwt_decoded()` and the `Authorization` header carried the fresh one
    - The write sits in the same validated (`Some(jwt)`) arm as other state writes, after key verification — mirroring `auth()`
    - A token that fails decode is never stored raw; `NoJwtFound`-on-rejected-auth preserved
    - No public API/DTO/signature change: `get_jwt()`, `refresh()`, and `ClientTrait` untouched
    - Test-only break: consumers who mirrored our stale-string assertions must update to expect fresh token from `get_jwt()`

- [OXA-000029](https://www.pivotaltracker.com/story/show/OXA-000029) - SDK re-auth restored: `refresh()` now validates its jwt against the raw-PEM wire format `GET /public_keys` actually serves
    - New kernel helper `Jwt::decode_with_flexible_public_keys` (`oxidauth-kernel/src/jwt/mod.rs`):
    - Tries real wire format first (raw PEM, from `ListAllPublicKeysUseCase` base64-decoded rows)
    - Falls back to base64-decoded PEM leg for version-skew insurance
    - Skips keys that verify nothing; unmatched walk ends with `"no valid public key found"` sentinel
    - `Client::refresh()` hand-rolled base64 decode loop deleted (`oxidauth-rs/src/client/mod.rs`): the per-key `BASE64_STANDARD.decode` loop silently skipped every raw-PEM key (PEM armor outside base64 alphabet)
    - Against a live server, `refresh()` can now validate its JWT; expiry-driven `get_jwt`/`request` no longer dies with `Other("failed to validate jwt")`
    - One `decode_with_flexible_public_keys` call now mirrors `auth()`'s shape
    - No public SDK API change; `Other("failed to validate jwt")` error shape preserved
    - `auth()` continues using `decode_with_public_keys` (unchanged)

- [OXA-000032](https://www.pivotaltracker.com/story/show/OXA-000032) - fix client METHOD error labels: `create_invitaion` typo and copy-pasted misnames (register CLI-6)
    - Fixed copy-pasted `METHOD` constants in three SDK client methods:
    - `Client::create_invitation`: `"create_invitaion"` → `"create_invitation"` (typo fix)
    - `Client::find_invitation`: `"create_invitaion"` → `"find_invitation"` (copy-paste fix)
    - `Client::list_all_authorities`: `"find_authority_by_strategy"` → `"list_all_authorities"` (copy-paste fix)
    - Empty-payload errors previously named the wrong method; `find_invitation` blamed `create_invitation`, and `list_all_authorities` blamed the unrelated `find_authority_by_strategy` endpoint
    - `ClientErrorKind::EmptyPayload` is public with `&'static str` method field
    - Consumers string-matching the typo'd labels or rendered `Display` sentences see corrected behavior — the corrected label is what any such matcher wanted
    - no other change: route, verb, DTOs, signatures, `RESOURCE` values (invitations stay `Resource::User`) untouched

- [OXA-000033](https://www.pivotaltracker.com/story/show/OXA-000033) - `TotpSettings` wire tokens renamed to snake_case with permanent decode aliases (register CLI-7)
    - `TotpSettings` now emits snake_case tags: `"disabled"` / `{"enabled":{…}}` instead of PascalCase `"Disabled"` / `{"Enabled":{…}}`
    - Kernel enum carries `#[serde(rename_all = "snake_case")]`; struct-variant fields (`totp_ttl`, `webhook`, `webhook_key`) are unchanged
    - Decode stays dual-accept permanently: `#[serde(alias = "Disabled")]` / `#[serde(alias = "Enabled")]` keep old PascalCase tokens decoding
    - Storage migration: `20260930120000_rename_totp_settings_tags_to_snake_case.sql` backfills `authorities.settings` — scalar `"Disabled"` → `"disabled"` via `jsonb_set`, and `{"Enabled":{…}}` → `{"enabled":{…}}`; both forms covered, idempotent
    - Breaking for external assertions on emitted tokens: anything asserting response/request carries `"Disabled"` or `"Enabled"` key must flip to snake_case
    - Senders unaffected (aliases accept both forever); emitters and verifier tests must update

- [OXA-000039](https://www.pivotaltracker.com/story/show/OXA-000039) - permission-tree flatten dedupes entitlements: a permission reachable through N grant paths now contributes ONE entry to `PermissionsResponse.permissions` and to every minted JWT
    - `UserNode::permissions()` / `RoleNode::permissions()` in `oxidauth-kernel/src/auth/tree/mod.rs` now deduplicate via a shared `HashSet<Uuid>` keyed on `permission.id`, so each distinct permission appears exactly once in the flat list
    - **Breaking note:** `PermissionsResponse.permissions` and the `txt …`/`gz …` JWT claim now carry one entry per *distinct* permission instead of one per grant path — claim/array bytes shrink only when duplicates existed
    - Affected mint paths: `authenticate`, `register`/`authenticate_or_register`, `exchange_refresh_token`, `totp/validate`; per-mint byte savings of `Σ(len(perm)+1)` raw (~×4⁄3 base64url encoded in the JWT)
    - Ordering: first-occurrence-wins, stable (nested-before-direct, declaration-order); never sorts, never moves entries
    - `PermissionsResponse.tree` is unchanged — per-path grant DTOs preserved for audit view
    - Authorization semantics unchanged: `oxidauth_permission::validate` short-circuiting any-match is multiset-safe
    - Callers inherit the fix unchanged: `oxidauth-postgres` tree queries, four JWT mint paths, `oxidauth-rs` `ExtractEntitlements`
    - No migration, no schema change, no wire-format change

- [OXA-000040](https://www.pivotaltracker.com/story/show/OXA-000040) - `Entitlements::Gz` is now symmetric: the variant holds the wire payload (base64-of-gzip) on both the mint side and the decoded side, so decoded `Gz` claims re-serialize/re-sign into tokens every verifier accepts
    - `Entitlements::Gz` now holds the wire payload (base64-of-gzip) on both the mint side and the decoded side in `oxidauth-kernel/src/jwt/mod.rs`
    - **Zero wire drift:** the claim format (`"txt …"` / `"gz <base64-of-gzip>"`) is unchanged; issued-token bytes are byte-identical before/after — tokens minted pre-fix decode post-fix and vice versa
    - `Entitlements::decode` still validates via `BASE64_STANDARD.decode` + `GzDecoder::read_to_string` but stores the verbatim base64 payload in `Gz(..)` instead of decompressed plaintext
    - `as_vec` inflates `Gz` on demand; malformed payload now yields `None` instead of a garbage entry
    - Both extractors (`oxidauth-api` `ExtractEntitlements`, `oxidauth-rs` `ExtractEntitlements`) keep their fail-closed `unwrap_or_default()` shape
    - `Serialize`/`Deserialize` are now correct under the identity contract: `Deserialize ∘ Serialize` is identity; re-issuing a decoded `Jwt` without rebuilding entitlements no longer mints valid-signature/undecodable-claims tokens
    - Consumer-visible delta: decoded claims' `Gz(..)` payload is now the base64 wire payload instead of decompressed plaintext — reading via `as_vec()` is unaffected

- [OXA-000046](https://www.pivotaltracker.com/story/show/OXA-000046) - wire `EnvVarError::RustLog`; corrupt `RUST_LOG` now fails fast at boot (register SRV-10, Option A)
    - Adds `EnvVarError::RustLog` variant so corrupt `RUST_LOG` (non-UTF-8 value) now fails fast at boot instead of silently defaulting to `INFO`
    - **Behavior change:** `oxidauth-api` and `seedz` abort at `telemetry::get_logging_envs()?` and exit non-zero, printing `Error: RustLog(NotUnicode(..))` to the terminal
    - **stderr change:** unset `RUST_LOG` still defaults to `INFO` but now announces via `eprintln!` (the old in-crate `error!`/`info!` pair was structurally invisible before any subscriber existed)
    - `telemetry::get_subscriber` no longer masks corrupt `RUST_LOG` — it panics on parity with the boot abort
    - `EnvVarError` enum, `Display`, and `Error` impls unchanged; the new `RustLog` variant is reachable from both binaries
    - Docs: `src/oxidauth/helm/README.md` env table now states `RUST_LOG` is optional (unset ⇒ `INFO`) and non-UTF-8 aborts boot

- [OXA-000048](https://www.pivotaltracker.com/story/show/OXA-000048) - fix ParseUserStatusErr error label (register SRV-12)
    - `ParseUserStatusErr`'s `Display` previously reported `failed to parse user_kind, unknown: <input>` for every rejected *status* token (copy-paste error)
    - Now correctly reads `failed to parse user_status, unknown: <input>`
    - **Breaking note:** external alert rules grepping `failed to parse user_kind` for status parse failures must repoint at `failed to parse user_status`
    - No API/DTO/signature change; `UserStatus::FromStr` now matches `ENABLED`/`INVITED`/`DISABLED` consts instead of raw literals (behavior-identical)

- [OXA-000050](https://www.pivotaltracker.com/story/show/OXA-000050) - duplicate username reports `UserAlreadyExistsError`, not raw Postgres text
    - New kernel error `oxidauth_kernel::users::UserAlreadyExistsError` (shape: `UserAlreadyExistsError::username(&Username) -> Box<Self>`, same as `UserNotFoundError` / `UserAuthorityNotFoundError`)
    - `PgUserRepository::insert_user` maps SQLSTATE `23505` on `users_username_key` constraint to this error
    - **Consumer-visible delta on `POST /api/v1/auth/register` and `POST /api/v1/users`:** status code unchanged (400), body text changes from raw SQL (`SQLSTATE 23505`, internal constraint name, Postgres DETAIL) to sanitized `"username is already taken"`
    - `Display` is sanitized (no SQLSTATE, no constraint name, no username echo); `Debug` keeps the type name and username for tracing
    - `UserAlreadyExistsError` has **no** `source()` to prevent the driver text from leaking into the error envelope
    - Only SQLSTATE `23505` on `users_username_key` is translated; every other DB error propagates raw and byte-identically (e.g. `VARCHAR(64)` overflow `22001` stays raw)
    - A `users_pkey` collision (client-supplied `id` re-POSTed) stays raw — it's an id clash, not a taken username

- [OXA-000062](https://www.pivotaltracker.com/story/show/OXA-000062) - unblock the fmt lane: bump `required_version` 1.8.0 → `>=1.11.0` on a canonical dated nightly and reformat the workspace (register T-2)
    - `rustfmt.toml` `required_version` bumped from exact `"1.8.0"` to `">=1.11.0"` to unblock the fmt lane across current toolchains
    - Canonical fmt toolchain is now **`nightly-2026-09-27`** (rustfmt 1.11.0-nightly)
    - Whole workspace reformatted in one pass (`cargo +nightly-2026-09-27 fmt --all`): 228 `.rs` files; formatting only, zero semantic edits (drift was import regrouping, chain/line re-wrapping, empty-item braces, hex-literal uppercasing)
    - `rust-toolchain.toml` header and `README.md` convention docs updated to cite `cargo +nightly-2026-09-27 fmt --all`
    - No deprecated/removed config keys detected; `unstable_features = true` and `style_edition = "2024"` stay
    - Warning: plain `cargo fmt` is NEVER the fmt lane — stable-channel rustfmt ignores nightly-only options and reformats differently (334 hunks across 192 files of churn)

- [OXA-000069](https://www.pivotaltracker.com/story/show/OXA-000069) - plan-09: every named site moved off the kernel `Service` trait, then the trait was removed (supersedes OXA-000067's T-7 rewrite)
    - **Breaking removal:** `oxidauth-kernel/src/service.rs` deleted — `Service`, `Layer`, `CanLayer`, `CanService`, `CanError`, `ExtractPermissions` and the `pub mod service;` declaration
    - 37 `pub use crate::service::Service;` re-exports across kernel request modules removed; `dev_prelude` narrowed to `pub use crate::error::BoxedError;`
    - 47 `oxidauth-repository` trait files + `oxidauth-repository/src/prelude.rs` dropped `service::Service` re-exports
    - 57 named `*Query` traits now own real async methods (e.g. `InsertRoleQuery::insert_role(&self, params: &CreateRole)`); `Service<&Params>` supertraits and `impl<T: Service<…>> XQuery for T` blanket bridges deleted
    - All 58 `oxidauth-postgres` impls and 105 `oxidauth-services` mocks implement named traits directly, bodies unchanged
    - `oxidauth-api/src/middleware/can.rs` rebuilt as a direct stateless check: `can()` = `parse` + split entitlements + `validate` — `CanLayer`/`CanService` composition deleted; `CanError` is now API-local
    - ~114 `use of deprecated` warnings retired; 46 `#![allow(deprecated)]` and 1 `#[allow(deprecated)]` deleted — tree-wide `allow(deprecated)` under `src/oxidauth` is now 0
    - Semver: breaking removal rides the unpublished `0.9.0` of `oxidauth-kernel`

- [OXA-000071](https://www.pivotaltracker.com/story/show/OXA-000071) - the kernel `Provider` is gone; the xlib `provider` crate is the only Provider
    - `oxidauth-kernel/src/provider/mod.rs` and its `pub mod provider;` declaration deleted — the kernel DI corpse is fully retired
    - Plan 04 had moved every consumer to the vendored xlib `provider::Provider`; `oxidauth-api/src/provider/mod.rs` keeps re-exporting the xlib type as `crate::provider::Provider`
    - Deprecation noise at zero: `cargo build --workspace | grep -c 'use of deprecated'` = 0
    - 14 `oxidauth-postgres` test mods received glob imports to fix 16 pre-existing `E0599` compile errors (sibling seed/re-read traits used without importing)
    - **Docs:** `docs/CLIENT_MIGRATION.md` §4 updated — `Service`/`Provider` are both now removed, xlib `provider::Provider` is the only one
    - Semver: breaking removal rides the unpublished `0.9.0` of `oxidauth-kernel`

## Client SDK, wasm & oxidauth-web

- **auto-jwt-refresh** - oxidauth-rs proactive jwt auto-refresh
    - `oxidauth` client now rotates the session JWT *before* it expires, on a schedule derived from the token's own `exp` claim (stamped from the authority's `jwt_ttl`)
    - New `pub const DEFAULT_JWT_REFRESH_BUFFER` (15s) and `Client::with_refresh_buffer(Duration)` override; the buffer is the lead time before `exp` at which the session rotates
    - Every build arms a background schedule on session transition (`auth`, `refresh`, `recover_jwt`): sleeps until `exp - buffer`, exchanges the refresh token, reschedules off the rotated `exp`; `logout` disarms; failed exchange retires the task (request-time refresh takes over)
    - Request-time threshold in `check_auth_state`/`recover_jwt` moves from "expired" to "inside the buffer"
    - Wasm arming is cooperative: `spawn_local` offers no abort handle, so `arm`/`disarm` bump a shared generation counter; sleep uses `gloo_timers::future::TimeoutFuture`
    - Native-only dep: `parking_lot` (the task-slot mutex); the async state lock stays `tokio::sync::RwLock`
    - `Client::refresh` delegates to a free `refresh_session`; `get_public_keys` delegates to a free `fetch_public_keys`

- **jwt-decode-unverified-base64url** - `Jwt::decode_unverified` accepts real (unpadded) JWT segments
    - `Jwt::decode_unverified` previously used `base64`'s **padded** prelude engine (`BASE64_URL_SAFE`), which rejected both unpadded segments (2/3 of real tokens → `Invalid padding`) and padded segments (`=` → `InvalidByte`)
    - **Fix:** decode `payload.trim_end_matches('=')` with `BASE64_URL_SAFE_NO_PAD` — the RFC 7515 wire format, with trailing `=` tolerated from non-conforming producers
    - **Affected only:** wasm/client build of `oxidauth-rs` — `verify_auth_jwt` (`auth()` died with `Other("failed to parse jwt")`) and `verify_session_jwt` (`refresh()`/`recover_jwt()` added wasted refresh round-trips)
    - `server`-feature paths that verify through `jsonwebtoken`'s decoder are untouched
    - All segments that decoded before still decode identically; nothing on the wire, mint side, or signature-verifying paths changes

- **oxidauth-virtual-host** - aka/dory virtual-host routing
    - Removes every `ports:` mapping from the stack docker-compose; nothing publishes a host port anymore
    - Routes everything through the dev proxy (aka / dory) via `VIRTUAL_HOST`:
    - API → `http://api.oxidauth.localhost`
    - Dev console → `http://app.oxidauth.localhost` (new in-container `trunk serve` service; production-posture nginx smoke moves to profile `web` as `oxidauth-web-prod` on `http://web.oxidauth.localhost`)
    - Postgres → `postgres.oxidauth.localhost:5432` via aka's raw TCP route (`VIRTUAL_PROTO: tcp`), replacing the 5434 host mapping
    - `.env`/`example.env` host-side URLs now point at the postgres virtual host instead of `127.0.0.1:5434`
    - Restores the oxidauth-web production build lane (`build/Dockerfile`, `build/nginx/`, `build/build.sh` with the plan-12 push gate) and adds the `devops/docker/leptos` trunk dev image

- **oxidauth-web** - oxidauth-web admin console
    - Adds the `oxidauth-web` crate: a Leptos CSR admin console (trunk-built) for authorities, users, roles, permissions, and invitations, gated on an authenticated SDK session
    - Makes the published `oxidauth` client usable from the browser:
    - `#[cfg_attr]` wasm split across client traits (`async_trait(?Send)` on wasm, `tracing::instrument` compiled out on wasm)
    - Wasm build persists the JWT + refresh token to LocalStorage and rehydrates/refreshes the session on page load
    - GET/HEAD requests no longer send a JSON body (fetch transport rejects bodies on those verbs)
    - `ClientError`'s `Display` now appends the full source chain
    - SDK addition: `delete_invitation` client wrapper (console needs it; the API route already existed)

- **oxidauth-web-26-10-01** - oxidauth-web admin console
    - Adopts the `47-crossbuck` brand: `logo.svg` is the all-lowercase wordmark (true x-band o, single-storey a) with an oversized 1.2x railroad crossbuck in the x slot; `favicon.svg` is the two-board sign alone (no rivet bolts)
    - Wordmark at geometry v5: uniform tracking +3 on every letter gap; wordmark centers horizontally on sidebar axis; lockup replaces `<Logo /> + "oxidauth" text-span` headers on login card, config-error card, and sidebar
    - Brand house files under `brand/logos/` (`47-crossbuck-{lockup,favicon}[-reverse].svg`, `CONCEPTS.md`, `contact-sheet-final.html`)
    - Create affordances: list headers for users, roles, authorities, and invitations now read "+" instead of "New <entity>"; permissions inline add row also uses "+"; each carries words in `aria-label`/`title`
    - Edit and Detail links use plain button box (`a.btn`, `btn btn-primary` pairing dropped); `.row-actions` is a flex row with gap
    - All destructive buttons (`Delete`, `Revoke`) use `btn btn-danger`; `.btn-danger` rule paints every delete red, redundant `danger` spellings dropped
