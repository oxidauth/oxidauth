# Oxidauth

Self-hosted authentication service — a project wrapping the `oxidauth` stack
(JWT auth, authorities, roles, permissions, OAuth2, TOTP, refresh tokens).

## Architecture

Stacks follow the 8-layer architecture; the last column is what **this
project** actually ships. The 8 layers are a menu, not a mandate.

| # | Layer      | Crate                 | In this stack                                                                      |
|---|------------|-----------------------|------------------------------------------------------------------------------------|
| 1 | kernel     | `oxidauth-kernel`     | ✅ domain types + named service traits (depends on `oxidauth-permission`)          |
| 2 | repository | `oxidauth-repository` | ✅ one query trait per operation                                                    |
| 3 | postgres   | `oxidauth-postgres`   | ✅ `Pg<Entity>Repository` impls + sqlx migrations                                    |
| 4 | services   | `oxidauth-services`   | ✅ `<Op>UseCase` impls (the pre-0.9 "usecases" crate, renamed)                     |
| 5 | http       | `oxidauth-http`       | ✅ DTO-only wire crate (server moved out in 0.9.0)                                  |
| 6 | api        | `oxidauth-api`        | ✅ axum server (`oxidauth-http` was the server crate's name until the 0.9 split)    |
| 7 | rs         | `oxidauth-rs`         | ✅ exists — the **published client crate is named `oxidauth`**                      |
| 8 | web        | —                     | ❌ absent: this service ships no UI; the helm chart keeps `web.enabled: false`      |

Support crates: `oxidauth-permission` is a **stack** crate
(`src/oxidauth/oxidauth-permission` — permission-token parsing/validation;
the kernel depends on it), while `src/seedz` is **project-level** (local-dev
fixture seeder, `ENVIRONMENT=local` guarded). `oxidauth-cli` and
`oxidauth-import-export` are kept stubs. The shared xlib crates
(`src/xlib/{http,postgres,provider,telemetry}`) are vendored from the project
template. Crate-level details, layer diagram, and the Provider pattern live in
[`src/oxidauth/README.md`](src/oxidauth/README.md).

## Getting Started

Prerequisites: Docker, Rust (rustup honors `rust-toolchain.toml`), `hurl`
(for the API test suite only).

```bash
# Configure environment
cp example.env .env
```

Then **edit `.env`** and set the two deterministic-bootstrap pins — the
default authority's client key (any UUID, e.g. `$(uuidgen)`) and the
bootstrap admin password:

```bash
OXIDAUTH_DEFAULT_CLIENT_KEY=...
OXIDAUTH_DEFAULT_ADMIN_PASSWORD=...
```

The server's first boot provisions (once) the signing keypair, the
`oxidauth:**:**` permissions, the `oxidauth:admin` role/user, and the default
authority using exactly these values; `src/oxidauth/hurl.sh` logs in with
them, so the suite is reproducible. Leave them unset for a random bootstrap —
then hurl runs won't know the credentials.

```bash
# Start the stack (postgres on host :5434; the api publishes its port 80 on
# an EPHEMERAL host port — always read it from compose, never hardcode)
docker compose up -d

# Wait for the api to come up (first run builds the dev image and compiles
# the workspace inside the container — give it a few minutes, then retry):
curl -fsS "http://127.0.0.1:$(docker compose port oxidauth-api 80 | head -1 | sed 's/.*://')/api/v1/__meta/healthcheck"
# → {"success":true,"payload":{"version":"0.9.0","healthy":true}}
```

Optional but recommended for dev boxes — idempotent demo fixtures (a second
authority, demo users/roles/permissions/grants). Run the server **once**
before the first seed (bootstrap creates what the fixtures build on):

```bash
docker compose --profile seed run --rm seedz
```

Test the live API end-to-end (17 hurl files, two passes — the second pass
only passes if the first cleaned up after itself), and the non-DB unit tests:

```bash
bin/hurl.sh
bin/unit_test.sh
```

## Available Stacks

- **oxidauth** (`src/oxidauth/`) — authentication & identity: kernel,
  repository, postgres, services, http, api, rs (as `oxidauth`), permission,
  plus the cli/import-export stubs, its own `docker-compose.yml`,
  `hurl/` suite, and `helm/` chart. See
  [`src/oxidauth/README.md`](src/oxidauth/README.md).

To add a second stack:

```bash
cargo generate --git git@git.vizerapp.cloud:freshbrewlabs/stack-template.git \
  --name mystack --allow-commands
# the template lives in the stack/ subfolder; cargo-generate locates it;
# --allow-commands runs the post-generation hook that renames inside helm/
mv mystack src/mystack
```

Then link it into the project (the `link-stack-to-project` checklist):

1. **Workspace members** — add `"src/mystack/*"` to `members` in the root
   `Cargo.toml`, and extend `exclude` for its non-crate siblings
   (`src/mystack/helm`, any `hurl` dir — cargo globs need manifests).
2. **Compose** — add `- ./src/mystack/docker-compose.yml` to the `include:`
   list in the root `docker-compose.yml` (each stack owns its own compose
   file and services; the root declares shared volumes).
3. **Deployment** — add a second helm release block to `bin/deploy.sh` (or a
   second call if you refactor it into a `deploy <stack> <env>` function):
   today it renders one hardcoded `helm upgrade` for the oxidauth stack.
   Per-stack test/build scripts ARE auto-discovered — `bin/build.sh`,
   `bin/hurl.sh`, and `bin/database_test.sh` find `build.sh` / `hurl.sh` /
   `database_test.sh` under `./src`; unit tests run workspace-wide, no
   per-stack script.
4. **Helm** — nothing to do in this chart: each stack owns its
   `src/mystack/helm/` chart and its `values-<env>.yaml` overlay; you wire
   the release into `bin/deploy.sh` (step 3), so a stack is simply not
   deployed until you add it there.

## Development

The workspace is edition 2024 and `rust-toolchain.toml` pins **1.98.0** —
rustup installs it on first `cargo` command, matching the dev image's rustc
(`RUST_DEV_IMAGE_VERSION=v1.98.0`). Production image builds pin their own
base (`v1.89.0` toolchain in `build/Dockerfile` args); image *tags* are crate
versions (`bin/crate_version.sh`), not the toolchain number.

```bash
# Format — repo convention: rustfmt.toml uses nightly-only options and pins
# required_version = ">=1.11.0", so the fmt lane is a pinned nightly, NOT
# plain `cargo fmt`: stable rustfmt ignores the nightly-only options and
# silently reformats to a different style. NEVER run plain `cargo fmt` in
# write mode; older NIGHTLY rustfmt (<1.11.0) hard-errors on the version
# guard; today's stable ignores the pin — hence NEVER plain write mode.
cargo +nightly-2026-09-27 fmt --all
```

```bash
# Lint
cargo clippy --workspace --all-targets
```

Clippy honesty note: the tree currently carries ~700 `deprecated` warnings
from the kernel `Service`/`Provider` shims (marked `since = "0.5.0"`, gone at
2.0 — see below). Errors are what matter; the flood is expected until 2.0.

Plain `cargo check` does not compile `#[cfg(test)]` code — never quote it as a green signal for test-bearing crates; compile-only proof is `cargo test --no-run` or `cargo check --all-targets`, and the real gates are `bin/unit_test.sh` + `bin/database_test.sh` (together they cover the crates `unit_test.sh` alone excludes).

Useful scripts (all idempotent, all runnable from the repo root):

| Script                       | What / when                                                                                   |
|------------------------------|------------------------------------------------------------------------------------------------|
| `bin/unit_test.sh`           | every test that needs no database (excludes the `*-postgres` crates)                          |
| `bin/database_test.sh`       | the postgres suites — needs `docker compose up -d` (reads `.env`)                             |
| `bin/hurl.sh`                | live-API suite against the running stack (port from compose, secrets from `.env`)             |
| `bin/reset-db.sh`            | drop/create/migrate the dev database (stop `oxidauth-api` first)                              |
| `bin/version.sh`             | show all crate versions; `bin/version.sh 0.9.1` moves the whole workspace at once             |
| `bin/crate_version.sh NAME`  | one crate's version — the same number the image tag derives from                              |
| `bin/build.sh`               | run every stack's `build/build.sh` (multi-arch images; push is an explicit `PUSH=1` gate)     |
| `bin/deploy.sh <env>`        | helm deploy to the `fbl-k3s` cluster (production asks to confirm; needs `crypt-keeper decrypt`) |
| `bin/uninstall.sh <env>`     | helm uninstall the same release                                                               |
| `bin/publish.sh`             | publish the eight crates to crates.io in dependency order; refuses a dirty tree                |

## 2.0 follow-ups

The 0.9 line keeps deprecated surfaces compiling so published consumers aren't
broken twice. What dies (or changes shape) at a coordinated 2.0:

- **Deprecated kernel machinery** — `Service<Request>::call`, the
  `Service<&X>`-style repository query aliases, and kernel's own `Provider`.
  The pattern today is named methods: kernel `<Op>ServiceTrait` +
  `<Op>Service = Arc<dyn ...>`, wired in `oxidauth-api/src/provider/`.
  Details: migration plan 04/09 execution notes.
- **xlib publish names** — four of the eight published crates depend on the
  vendored xlib crates by their bare template names (`http`, `postgres`,
  `provider`, `telemetry`); on crates.io those resolve to foreign crates.
  2.0 must reserve/rename them or stop depending across that line —
  migration plan 14 execution note.
- **wasm `Send` wall** — the client's `#[async_trait]` traits box futures as
  `Send`, but reqwest on wasm is `!Send`, so `oxidauth --features wasm`
  cannot compile as designed; the fix is a `?Send`/module-gated client
  redesign — migration plan 14 execution note.

## Documentation

- [Client migration 0.8 → 0.9](docs/CLIENT_MIGRATION.md) — breaking DTO /
  crate-path changes for consumers (parkinglot et al.)
- [Authorities](docs/AUTHORITIES.md) · [OAuth2 flow](docs/OAUTH.md) — design
  docs (living history: rfcs under `rfcs/`)
- [Security report](docs/SECURITY_REPORT.md) — audit findings and their fix
  status
- [Migration plan](docs/migration-plan/README.md) — how this repo was
  restructured onto the project/stack template conventions (plans 01–15)

## Security

Vulnerability findings and their remediation context are tracked in
[`docs/SECURITY_REPORT.md`](docs/SECURITY_REPORT.md) — an audit with
pointers into the current code. Unresolved serious issues should go to the
private issue tracker before any public channel.
