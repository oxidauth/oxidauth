# 13 — seedz (local development seeding only)

**Status**: `reviewed` (walkthrough 2026-09-29: revised + approved — seedz local-dev fixtures only, bootstrap untouched)
**Depends on**: 05
**Risk**: low. **Zero server-boot behavior change** — bootstrap is explicitly
left alone (see separation note).

## Goal

`src/seedz/` becomes the template's project-level seeder (Seeder trait +
SeedRunner), used **only for local development data**.

**Separation of concerns (decided, not open):**
- **seedz = local dev fixtures.** Demo authorities/users/roles for hacking
  locally (web probes, manual testing, screenshots). Guarded so it can never
  point at staging/production by accident. Never invoked by the server.
- **bootstrap = first-run provisioning.**
  `oxidauth-usecases/src/bootstrap/mod.rs` (`SudoUserBootstrapUseCase`,
  called from `main.rs` boot) creates, once, the things a *new instance of
  any environment* needs to function: JWT key pair
  (`first_or_create_public_key`), `oxidauth:**:**` +
  `oxidauth:totp_code:validate` permissions, `oxidauth:admin` role + grant,
  default `username_password` authority (client key/TTLs from env), sudo user
  `oxidauth:admin` (password from `OXIDAUTH_DEFAULT_ADMIN_PASSWORD`), and a
  `"bootstrap"` **setting** that makes every later boot a no-op
  (`check_bootstrap_setting`). That is product provisioning, not seed data —
  it stays exactly where it is. (Parkinglot runs its *own*
  `src/seedz/src/oxidauth.rs` against *its* local stack — a consumer's seeder
  is not this repo's seedz.)

## Changes

1. **seedz skeleton** (template `src/seedz/src/lib.rs`, verbatim API):
   ```rust
   #[async_trait] pub trait Seeder: Send + Sync { async fn seed(&self, write: &PgPool,
       read: &PgPool) -> Result<(), BoxedError>; }
   pub struct SeedRunner { seeders: Vec<Box<dyn Seeder>> }   // add_seeder / run
   ```
   `[[bin]] name = "seedz"`. Deps: `sqlx`, `tokio`, `oxidauth-kernel`,
   `oxidauth-postgres` (`Database`/`MIGRATOR` from plan 05). Flow: read
   `DATABASE_URL`/`READ_DATABASE_URL` → run `MIGRATOR` → seeders.
2. **Dev-only guard**: first line of the bin — require `ENVIRONMENT=local`
   (else exit(1) with a loud message: "seedz is for local development;
   refusing to run against ENVIRONMENT=<x>"). The template ships no guard;
   we add one because *this* seedz targets a real auth database.
3. **`src/seedz/src/fixtures.rs`** — idempotent dev data only (find-then-insert,
   deterministic `f000…`-style UUIDs per parkinglot convention):
   - a second demo authority (`local-dev`, username_password) so multi-authority
     flows can be exercised
   - 2–3 demo users with distinct role/permission combinations (viewer/editor),
     never colliding with bootstrap's `oxidauth:admin` or hurl fixtures
   - one sample setting row
   No signing keys, no admin role/user, no default authority — bootstrap owns
   those; seeding them would fight the provisioning path.
4. **No change to bootstrap or boot.** `main.rs` keeps
   `bootstrap.call(&BootstrapParams)`; the bootstrap module rides along with
   the crate rename in plan 09 (`oxidauth-services/src/bootstrap/`), and the
   kernel `BootstrapService` trait stays. The old draft of this plan removed
   bootstrap from boot + added a `BOOTSTRAP_ON_BOOT` flag — **withdrawn**.
   Open follow-up *only if someone asks*: expose bootstrap as an explicit
   `oxidauth-api` subcommand so provisioning isn't boot-coupled; needs its own
   decision, not part of this migration.
5. Delete the `oxidauth-seed` stub body (plan 02 moved the dir; the `add()`
   test stub is replaced wholesale here).
6. **Compose**: add one-shot service to `src/oxidauth/docker-compose.yml`
   (parkinglot ships seedz as a compose service):
   ```yaml
   seedz:
     profiles: [ "seed" ]
     working_dir: /home/rust/src/oxidauth
     command: /bin/bash -c 'cargo run --bin seedz'
     image: registry.vizerapp.cloud/lib/rust-dev:$RUST_DEV_IMAGE_VERSION
     depends_on: [ postgres ]
     environment:
       DATABASE_URL: postgresql://postgres:***@postgres:5432/oxidauth
       MIGRATIONS_ENABLED: "true"
       ENVIRONMENT: local
     env_file: [ ../../.env ]
     volumes:
       - "shared-vol:/home/rust/shared_target"
       - ../../:/home/rust/src/oxidauth:cached
   ```
   Run: `docker compose --profile seed run --rm seedz` (from repo root —
   include semantics from plan 10 make it appear).

## Verification

- **Guard**: `ENVIRONMENT=production cargo run --bin seedz` → exits 1, DB
  untouched; with `ENVIRONMENT=local` it proceeds.
- **Bootstrap untouched** (`git diff` after this plan): zero lines changed in
  `oxidauth-usecases/src/bootstrap/` (modulo plan 06/09 file moves) and the
  `bootstrap` call remains in `main.rs` boot.
- **Fresh DB flow**: `bin/reset-db.sh` → boot server once (bootstrap provisions
  keys/permissions/admin/authority exactly like today; `bootstrap` setting
  written) → `--profile seed run seedz` **twice** → second run changes no row
  counts (idempotency).
- `bin/hurl.sh` still green after seeding (no fixture-username collisions).
- Server restart log on a seeded DB: bootstrap logs its no-op path (setting
  found) — no duplicate authority.

## PR note

changelog `<id>-seedz-dev-seeding`; README "Available Stacks/dev setup" mention
handled by plan 15. Status → `done`.
