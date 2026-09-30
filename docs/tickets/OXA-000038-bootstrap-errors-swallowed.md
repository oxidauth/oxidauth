# OXA-000038 — Bootstrap swallows the totp-permission create failure and a malformed pinned client key, then permanently marks itself complete

**Original ID:** SRV-2 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; both swallows verified intact at HEAD — `_totp_permission` drop at :266, triple-collapse of unset/non-UTF8/malformed `OXIDAUTH_DEFAULT_CLIENT_KEY` to None at :397-402, sentinel permanence at :99-103. Fix A/B/C recipes written & sound (propagate create Err; explicit match w/ boot-abort on malformed key; `create_fails_for` mock); retry-after-failure already safe (main aborts pre-sentinel) — revisit before prod pushes; hazard shrinks OXA-000021 surface)


## Locations

Register cites `oxidauth-services/src/bootstrap/mod.rs:1641,1522` — **marker drift**: both cited lines are `BUG(pinned)` comments inside the test module, not the defects. Real sites:

- `oxidauth-services/src/bootstrap/mod.rs:266-280` — `first_or_create_permissions` discards the totp create/find result in `let _totp_permission = ...` (TODO at `:256` admits it); the second (admin) block at `:282-304` is the only one whose `Result` is returned. Caller at `:130` binds the admin permission only.
- `oxidauth-services/src/bootstrap/mod.rs:397-402` — `first_or_create_authority` reads `OXIDAUTH_DEFAULT_CLIENT_KEY` through `env::var(..).ok().map(parse).transpose().ok().flatten()`; a parse error (and a non-UTF-8 value) collapses to `None` with no log.
- `oxidauth-services/src/bootstrap/mod.rs:101-105,195-201` — the sentinel gate ("bootstrap already completed") and `save_bootstrap_setting` at the end of the sequence: once reached, the run never repeats.
- `oxidauth-services/src/authorities/create_authority.rs:29-32` — backfills `req.client_key` with `Uuid::new_v4()` when `None`, so the authority persists with a *generated* key.
- `oxidauth-api/src/main.rs:30-34` — sole invocation: `bootstrap(&BootstrapParams).await?` — a returned `Err` aborts boot (process exits non-zero, server never starts).
- Downstream readers: `oxidauth-services/src/auth/authenticate.rs:167-174` (embeds the `TOTP_VALIDATE_PERMISSION` string constant into the 2FA-pending JWT), `oxidauth-api/src/server/api/v1/totp/validate.rs:25-36` (checks the JWT's embedded entitlements only), `oxidauth-services/src/role_permission_grants/create_role_permission_grant.rs:61-67` and `oxidauth-services/src/user_permission_grants/create_user_permission_grant.rs:4,64` (resolve grants via `FindPermissionByParts` → `PermissionNotFoundError` when the row is absent).
- Pinned tests: `bootstrap/mod.rs:1501-1525` (`unparseable_client_key_env_is_silently_dropped`, marker `:1522-1523`), `:1618-1656` (`a_failing_create_step_aborts_before_later_steps_and_saves_nothing`, marker `:1641-1643`), abort-suite `:1527-1616`, `missing_env_vars_fall_back_to_generated_material` `:1462-1498` (unset-env pin `:1481-1485`), mock `MockPermissions` `:882-929` (`create_fails: bool` at `:884,919-921`), `Cfg` `:703-738`.

**Register correction:** the `gen_dev_keypair` parenthetical is stale — no such symbol exists anywhere in the codebase (also recorded in OXA-000019; `git log -S gen_dev_keypair` over `src/` finds nothing). Today the client key is backfilled by an infallible `Uuid::new_v4()`, so "authorities persist *without* a client key" cannot literally occur; what does occur is "the authority persists without the client key the operator pinned", which is the real defect at `:397-402`.

## Problem

Two independent error-swallowing sites in the sudo-user bootstrap let a broken provisioning run reach completion, after which the `bootstrap` setting sentinel (`:101-105`) guarantees it is never repaired:

1. **totp permission create failure is invisible.** In `first_or_create_permissions`, the `Result` of finding-or-creating `oxidauth:totp_code:validate` is bound to `_totp_permission` and dropped (`:266-280`). If that create fails while the admin-permission create succeeds, the whole sequence — role, grants, authority, admin user, role grant — completes and `save_bootstrap_setting` persists (`:195-201`). The pinned test at `:1618-1656` demonstrates the masking: with all `create_permission` calls failing, the abort lands on the *second* (admin) create (`:1644-1650`), because the first failure was already thrown away.
2. **A malformed pinned client key is silently reinterpreted as "unset".** `env::var(DEFAULT_CLIENT_KEY).ok()...transpose().ok().flatten()` (`:397-402`) maps three distinct states — unset, set-but-not-a-UUID, set-but-not-UTF-8 — onto the same `None`. `CreateAuthorityUseCase` then fabricates a random v4 UUID (`create_authority.rs:29-32`) and the authority persists under a key nobody configured, with zero log output. Neither site logs anything; the module logs elsewhere (e.g., `error!` at `:429`), so this is deliberate silence, not style.

## Analysis

**Why the totp hole survives reality.** For the sequence to complete while the totp row is missing, the first create must fail and the second must succeed: a transient connection/timeout error landing on exactly the first statement, or a concurrency race — two replicas cold-boot an empty DB; `permissions` has `CONSTRAINT unique_grant_parts UNIQUE(realm, resource, action)` (`oxidauth-postgres/migrations/20221019222410_create_permissions.sql:9`), so replica B's totp create fails on the unique violation (swallowed) while its admin create can still win the window before replica A inserts the admin row.

**What a missing totp row actually breaks.** Not the 2FA hot path: `authenticate.rs:167-174` embeds the hardcoded `TOTP_VALIDATE_PERMISSION` string into the pending-2FA JWT and `totp/validate.rs:25-36` matches only the JWT's embedded entitlements (`oxidauth-permission/src/tokens/mod.rs:72-83`), neither consults the `permissions` table. The damage is to the seed-data contract:
- `oxidauth/hurl/tests/permissions.hurl:58-65` asserts the seeded list contains both permissions (`count >= 2`) — integration CI catches the drift, production does not.
- The permission becomes **ungrantable**: both grant use cases resolve the parts against the DB and return `PermissionNotFoundError` (`create_role_permission_grant.rs:61-67`, `create_user_permission_grant.rs:64`), so no role or user can ever receive `oxidauth:totp_code:validate` until an operator manually creates the row.
- Repair-by-restart is impossible: the sentinel was saved, so `bootstrap()` returns `Ok` at `:101-105` forever.

**Why the client-key hole compounds.** Bootstrap runs exactly once. With the pinned key dropped, the *generated* UUID flows into registration (`:474` — `authority.client_key`), so the admin user registers fine against the wrong key and the sentinel saves. Every later login attempt using the pinned key (the hurl suite injects `admin_client_key` from the same `OXIDAUTH_DEFAULT_CLIENT_KEY` env — `oxidauth/hurl/variables-local:5-9`) targets an authority that does not exist by that key (surfacing as the dead-`None` lookup documented in OXA-000021). Because the sentinel persists, fixing the env var and restarting changes nothing: `first_or_create_authority` finds the existing authority by strategy and returns it — the corrected key is never re-applied; repair requires a manual PUT (itself impaired by OXA-000003's regenerate-on-omit) or SQL.

**Fail-fast is already the contract here.** `main.rs:32-34` propagates any bootstrap `Err` as a boot abort, and the existing suite pins that every *other* non-not-found failure "must abort" and must "NEVER save the bootstrap-complete setting" (`:1527-1616`). The two swallow sites are exceptions to the module's own established semantics. Crash-on-boot is the right trade for this service: every step is `first_or_*` idempotent (no sentinel on failure ⇒ safe retry after fixing the env var or clearing the transient DB fault), while partial provisioning is permanent and silent. A typed `BootstrapError` enum is optional sugar; the module's error currency is `BoxedError` and `main` already prints/propagates it.

## Impact

Who is affected: any operator deploying oxidauth-api cold — the pinned-client-key hole hits operators who set `OXIDAUTH_DEFAULT_CLIENT_KEY` with a typo (the variable is part of the documented local/hurl deployment contract, `hurl/variables-local:5-9`); the totp hole hits any deployment whose first bootstrap create hit a transient DB error or a replica race. Severity P2: no direct security bypass (the totp hot path reads JWT claims, not the table), but both produce **persistently corrupted, self-sealing provisioning state** — admin login permanently broken under the intended key, or a seed permission that can never be granted — with no log line pointing at the cause, discovered only when someone can't log in or a grant 404s.

## Proposed resolution

One PR in `oxidauth-services/src/bootstrap/mod.rs`; both fixes return `Err` so `main.rs` aborts boot before the sentinel is saved (all earlier `first_or_*` steps are retry-safe).

1. **Propagate the totp result** (`:256-280`). Delete the TODO, replace `let _totp_permission = match permission {...};` with a checked binding, e.g. `let _totp_permission = ...await?` at each fallible arm (or `?` on the whole match result), keeping the function's return contract = the admin permission (`:282-304`, caller at `:130` unchanged). Remove the `:1641-1643` marker and flip `a_failing_create_step_aborts_before_later_steps_and_saves_nothing` (`:1618-1656`) to assert the abort lands on `create_permission:{TOTP_VALIDATE_PERMISSION}` — the FIRST create — still with no `save_setting` op.
2. **Make the client-key env explicit** (`:397-402`):
   ```rust
   let client_key = match env::var(DEFAULT_CLIENT_KEY) {
       Ok(value) => Some(value.parse().map_err(|err| {
           error!(message = "invalid OXIDAUTH_DEFAULT_CLIENT_KEY", %value, ?err);
           format!("OXIDAUTH_DEFAULT_CLIENT_KEY must be a UUID, got {value:?}: {err}")
       })?),
       Err(env::VarError::NotPresent) => None, // documented fallback: create use case generates (create_authority.rs:29-32)
       Err(err) => return Err(format!("OXIDAUTH_DEFAULT_CLIENT_KEY unreadable: {err}").into()),
   };
   ```
   Keep `NotPresent` → `None` green (`missing_env_vars_fall_back_to_generated_material`, `:1481-1485`). Treat `NotUnicode` as fatal too — the current `.ok()` conflates it with unset. Remove the `:1522-1523` marker and flip `unparseable_client_key_env_is_silently_dropped` (`:1501-1525`, rename e.g. `unparseable_client_key_env_fails_the_boot`): `bootstrap(..)` must return `Err`, the error message must name `OXIDAUTH_DEFAULT_CLIENT_KEY`, and ops must contain no `create_authority`/`save_setting`.
3. **Strengthen the mock for the flipped pin**: `Cfg.create_fails: bool` (`:713`) fails both creates indistinguishably; change it to `create_fails_for: Option<&'static str>` (permission-string match in `MockPermissions::create_permission`, `:919-921`) so the flipped test pins `create_fails_for: Some(TOTP_VALIDATE_PERMISSION)` → abort AT the totp create, and add an admin-only case proving the second create still aborts too.
4. **Pins to respect**: OXA-000008 flags `:1453-1458` and `:1490-1497` as functional pins (register payload carries the raw admin password/confirmation; generated-password fallback) — neither fix touches the register payload or password path, both stay green untouched. Do not re-scope into OXA-000003 (PUT regenerate-on-omit) except to note there that repairing a wrong-keyed authority after this fix depends on it.

## Verification

- `cargo test -p oxidauth-services --lib bootstrap` — the two flipped tests plus the untouched abort-suite (`:1527-1616`), exact-params (`:1344+`, ops order `:1390-1398` unchanged), and `missing_env_vars_fall_back_to_generated_material` all green.
- New permanent assertions (in the flipped tests, not separate wiring tests): (a) totp-only create failure ⇒ `Err` containing the simulated failure, `ops.last()` starts with `create_permission:oxidauth:totp_code:validate`, no `save_setting`; (b) non-UUID env ⇒ `Err` naming `OXIDAUTH_DEFAULT_CLIENT_KEY`, no `create_authority` op, no `save_setting`. The `NotUnicode` arm is not settable through `std::env::set_var(&str)` on this harness — leave it to code inspection `[unverified at test level]`.
- Boot-path smoke: run `oxidauth-api` against a fresh DB with `OXIDAUTH_DEFAULT_CLIENT_KEY=not-a-uuid` — process must exit non-zero before `starting server...` (`main.rs:36`) with the naming error printed; then fix the value and restart — same `cargo run` succeeds and `GET /api/v1/permissions` lists both seeded permissions.
- Integration: existing `oxidauth/hurl/tests/permissions.hurl:58-65` seed assertion (`count >= 2`) stays green — it is the DB-side proof the totp row exists after a real bootstrap; no hurl changes needed.
