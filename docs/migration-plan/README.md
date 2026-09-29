# Migration Plan — Align oxidauth with freshbrewlabs project/stack templates

Goal: restructure this repo from a flat 11-crate workspace into the newer
**project-wraps-stacks** convention defined by:

- `~/dev/freshbrewlabs/project-template` (cargo-generate project wrapper)
- `~/dev/freshbrewlabs/stack-template/stack` (8-layer stack generator)
- verified against freshly generated output (`cargo generate`, cargo-generate 0.23.14)
  and two real projects built from them: `~/dev/freshbrewlabs/parkinglot` and `~/dev/freshbrewlabs/aka`.

## Target layout

```
oxidauth/                          # PROJECT (wraps stacks)
├── Cargo.toml                     # members: src/xlib/*, src/oxidauth/*, src/seedz ; exclude src/oxidauth/helm
├── docker-compose.yml             # include: - ./src/oxidauth/docker-compose.yml ; shared-vol
├── example.env  .env(gitignored)  # ENVIRONMENT, RUST_LOG, DOCKER_PLATFORM, RUST_DEV_IMAGE_VERSION,
│                                  # POSTGRES_*, DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, pepper
├── crypt-keeper.toml              # team
├── secrets/.gitkeep
├── bin/                           # build.sh database_test.sh deploy.sh hurl.sh unit_test.sh uninstall.sh
│   │                              # version.sh crate_version.sh (parkinglot additions)
├── devops/
│   ├── postgres/init.sh           # SERVICES loop: per-db user+database provisioning
│   └── helm/values-{staging,production}.yaml
├── src/
│   ├── xlib/                      # vendored shared crates from project-template
│   │   ├── http/                  #   Response<P> envelope + data.rs (LoadingState/SaveState/…)
│   │   ├── postgres/              #   database! macro (read+write pools, PgError, PingTrait, mock, MIGRATOR)
│   │   ├── provider/              #   type-erased Provider (TypeId→Arc<dyn Any>) + FromRef<Provider> for OxidAuthClient
│   │   └── telemetry/             #   get_logging_envs / get_subscriber(name, version, env, filter, sink)
│   ├── oxidauth/                  # STACK "oxidauth" (crates keep the oxidauth- prefix = stack name ✓)
│   │   ├── oxidauth-kernel/       #   pure domain (Provider/service style removed over time)
│   │   ├── oxidauth-repository/   #   query traits, one trait per operation
│   │   ├── oxidauth-postgres/     #   database! macro + Pg<Entity>Repository impls + migrations/
│   │   ├── oxidauth-services/     #   (was oxidauth-usecases) use cases + service traits
│   │   ├── oxidauth-http/         #   DTO-only crate (Req/Res moved out of the server) — name reused
│   │   ├── oxidauth-api/          #   (was oxidauth-http) axum server, provider/, server/, build/
│   │   ├── oxidauth-rs/           #   published client (pkg name `oxidauth`), unchanged externally
│   │   ├── oxidauth-permission/   #   stack-internal support crate (token compare + benchmarks)
│   │   ├── oxidauth-cli/          #   stub kept (review 2026-09-29) — future CLI feature
│   │   ├── oxidauth-import-export/ #  stub kept (review 2026-09-29) — future feature
│   │   ├── docker-compose.yml     #   per-stack dev compose (postgres + oxidauth-api watchexec)
│   │   ├── helm/                  #   stack helm chart (from stack-template)
│   │   ├── hurl/                  #   hurl tests + hurl.sh
│   │   └── README.md              #   stack readme (stack-template format)
│   └── seedz/                     # (was oxidauth-seed) Seeder trait + SeedRunner — LOCAL DEV FIXTURES ONLY
│                                  #   (ENVIRONMENT=local guarded); provisioning stays server-side (bootstrap)
```

Dropped crates: `oxidauth-telemetry` only (→ xlib/telemetry, plan 06).
`oxidauth-cli` + `oxidauth-import-export` are **kept** (empty stubs, relocated
with the stack — review decision 2026-09-29). No `web` layer: the
auth service has no UI; the 8 layers are a menu, not a mandate (parkinglot's
`app` stack ships web-only, aka's ships api+web — partial stacks are precedented).

## External-consumer constraints (why ordering matters)

- `oxidauth` (the `-rs` crate, pkg name **oxidauth**) and `oxidauth-kernel` are
  published (crates.io) AND pinned by consumers by git rev
  (`parkinglot/src/xlib/provider/Cargo.toml`: `oxidauth = { git = …, rev = "f358259" }`).
  The `oxidauth::{OxidAuthClient, axum::FromRef}` surface + `ExtractJwt/ExtractEntitlements/ExtractUserId`
  are **platform contract** — the new xlib/provider literally `impl FromRef<Provider> for OxidAuthClient`.
- Plan never edits the rs crate's public API beyond import paths. Kernel's
  `provider::Provider` and `service::Service` stay (marked deprecated) until a
  coordinated 2.0 of the published crates; only the server stops using them here.

## Statuses

Every numbered plan file starts with a status block. Allowed values:

| Status        | Meaning                                                        |
|---------------|----------------------------------------------------------------|
| `not-started` | written, not yet walked through in review                      |
| `reviewed`    | content approved in the joint walkthrough; ready to be picked up |
| `in-progress` | picked up, being worked on                                     |
| `blocked`     | picked up, waiting on something (note why in the file)         |
| `done`        | merged + verified                                              |

Flow: `not-started → reviewed → in-progress → done` (either pre-done state may
become `blocked`). The 2026-09/10 walkthrough flips plans to `reviewed` (or
records edits/holds directly in the plan file under a `## Review notes` section).
Update the `Status:` line in the plan file AND the table below in the same PR
that does the work (repo changelog convention `changelogs/<id>-<slug>.md` still applies).

## Plans (execute in order)

| #  | Plan | Status | Depends on |
|----|------|--------|------------|
| 01 | [Project root scaffolding](01-project-root-scaffolding.md) | `done` | — |
| 02 | [Move crates into src/ layout](02-move-crates-into-src.md) | `done` | 01 |
| 03 | [Vendor xlib crates](03-vendor-xlib-crates.md) | `done` | 02 |
| 04 | [Provider wiring: xlib provider + provider/ split](04-provider-wiring.md) | `done` | 03 |
| 05 | [Postgres via database! macro](05-postgres-database-macro.md) | `reviewed` | 04 |
| 06 | [Telemetry via xlib + boot sequence](06-telemetry-boot-sequence.md) | `reviewed` | 04 |
| 07 | [Split DTO crate: oxidauth-http ↔ oxidauth-api](07-split-http-dto-crate.md) | `reviewed` | 03, 04 |
| 08 | [Repository traits → Pg repositories](08-postgres-repository-structs.md) | `reviewed` | 05 |
| 09 | [usecases → services layer](09-services-layer.md) | `reviewed` | 08 |
| 10 | [Compose + dev environment](10-compose-dev-environment.md) | `reviewed` | 05, 06 |
| 11 | [Scripts + hurl/tests conventions](11-scripts-and-tests.md) | `reviewed` | 02, 07 |
| 12 | [Helm chart + deploy pipeline](12-helm-deploy.md) | `reviewed` | 11 |
| 13 | [seedz (local-dev fixtures; bootstrap untouched)](13-seedz-dev-seeding.md) | `reviewed` | 05 |
| 14 | [Edition 2024 + dependency/version alignment](14-edition-deps-versions.md) | `reviewed` | 02 (best after 09) |
| 15 | [Docs + README refresh](15-docs-refresh.md) | `reviewed` | all |

Grouping for parallel pickup: {01} → {02} → {03} → {04} → {05, 07} may proceed
in parallel after 04 → {06, 08} → {09, 10, 13} → {11, 12, 14} → 15.
Never reorder 02 after 03+ (all later diffs assume the src/ tree), and do 07's
rename before 07's DTO extraction (two commits, one plan).

## Evidence baseline

Regenerate the comparison baseline any time templates change:

```bash
cargo generate --path ~/dev/freshbrewlabs/project-template --name tmpdemo --destination /tmp/migcmp --force
cargo generate --path ~/dev/freshbrewlabs/stack-template/stack --name tmpstack \
  --destination /tmp/migcmp/tmpdemo/src --force --allow-commands
```

Note: raw `cargo generate` does **not** wire workspace members/compose includes
into the project — that is the `link-stack-to-project` skill's job; `parkinglot`
shows the wired end state (`members = ["src/xlib/*", "src/admin/*", "src/app/*", "src/seedz"]`).
