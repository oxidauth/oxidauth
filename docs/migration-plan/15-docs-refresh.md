# 15 — Docs + README refresh

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — runnable-code-block acceptance rule in force)
**Depends on**: all (docs describe the finished shape; skeleton from plan 01)
**Risk**: low.

## Goal

Documentation speaks in the new conventions and matches the tree.

## Changes

1. **Root `README.md`** (rewrite plan-01 skeleton) — project-template shape:
   - title + one-liner ("self-hosted auth service; project wrapping the
     `oxidauth` stack")
   - 8-layer table (kernel/repository/postgres/services/http/api/rs/web) with
     the **this-stack** column: which layers exist, why `web` is absent, where
     `permission`/`seedz` fit
   - Getting Started: copy `.env`, `docker compose up -d`, seed (plan 13
     command), `bin/hurl.sh`, `bin/unit_test.sh`
   - "Available Stacks": `oxidauth` (+ how to add a second stack via
     `cargo generate` + the link-stack-to-project checklist: workspace
     members, compose `include:`, `bin/deploy.sh` call, helm exclude)
   - Development: `cargo fmt`, `cargo clippy`, script table (`bin/*.sh`
     purpose + when to run)
2. **`src/oxidauth/README.md`** (stack readme) — stack-template README shape:
   layer diagram (kernel ← repository ← postgres/services ← http ← api,
   kernel+http ← rs), Provider pattern section (postgres + services init
   files), crates table, "Adding entities" walkthrough updated to this repo's
   actual chain: kernel type → repository trait (+`.sql` dirs in
   postgres) → `Pg<Entity>Repository` → services trait/UseCase → api handler
   → DTO in `oxidauth-http` → client method in `oxidauth-rs`.
3. **Client-facing docs** for the breaking plan-07 paths: short migration
   note (old `oxidauth_http::server::api::v1::users::create_user::CreateUserReq`
   → `oxidauth_http::users::create_user::CreateUserReq`;
   `oxidauth_http::response::Response` → `oxidauth_http::Response`) in
   `docs/CLIENT_MIGRATION.md` — parkinglot is the reference consumer.
4. **Existing docs**: `docs/AUTHORITIES.md`, `docs/OAUTH.md`, `rfcs/*` stay
   (living design history — templates don't have an opinion); add
   `docs/SECURITY_REPORT.md` link from README's security section (file moved
   plan 01). Replace remaining references to old paths
   (`grep -rn 'oxidauth-http/\|oxidauth-usecases' docs/ rfcs/ README.md` in
   this repo — rfcs get edits only for *path* references, never content).
5. **`changelogs/`**: one entry per merged migration PR (plans say this);
   add `changelogs/README.md` blurb describing the migration arc + this
   folder's role, cross-linking `docs/migration-plan/`.
6. **This migration-plan folder**: flip every plan's status; add a
   "Migration complete" date line to README.

## Verification

- All README code blocks execute as written (fresh clone, fresh `.env`):
  compose up → seed → healthcheck curl → one hurl file → `bin/unit_test.sh`.
- `grep -rn 'oxidauth-usecases\|oxidauth-telemetry\|dev.Dockerfile\|bin/hurl-tests' README.md docs/ --include='*.md' -l`
  → only migration-plan/history files match (living docs do not).
- Stack README layer diagram matches `cargo tree` edges exactly
  (`cargo tree -e normal -p oxidauth-api` spot-check).

## PR note

changelog `<id>-docs-refresh`; Status → `done`.
