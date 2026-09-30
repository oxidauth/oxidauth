# OXA-000021 — select_authority_by_client_key returns RowNotFound for missing keys; the `Option<Authority>` contract is unreachable

**Original ID:** DATA-8 · **Severity:** P2 · **Type:** bug · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #17 of 27 · reviewed 2026-09-29 · SCHEDULED — schedule BEFORE OXA-000003


## Locations

Register line (`BUGS_AND_NOTES.md:40`): `oxidauth-postgres/src/authorities/select_authority_by_client_key/:71` — "`Response = Option<Authority>` but query is `fetch_one` — missing key surfaces as `RowNotFound` error; the `None` branch is unreachable dead API."

Known drift pattern, confirmed here: `:71` is the `BUG(pinned)` comment marker inside the test module (`mod.rs:71-73`), not the product code. The defect itself is the `fetch_one` call at `mod.rs:19` (with the lying `type Response = Option<Authority>` at `:9`). The marker text is an accurate description of the defect, so the register wording is correct even though the line pointer is test-side.

Verified current locations (crate paths relative to repo root; everything under `src/oxidauth/`):

| What | Path |
|---|---|
| The defect | `src/oxidauth/oxidauth-postgres/src/authorities/select_authority_by_client_key/mod.rs:6-25` — `type Response = Option<Authority>` `:9`; `fetch_one` `:19`; unconditional `Ok(Some(authority))` `:24` |
| Query | `src/oxidauth/oxidauth-postgres/src/authorities/select_authority_by_client_key/select_authority_by_client_key.sql:1-3` — `SELECT * FROM authorities WHERE client_key = $1`; at most one row by construction (`client_key UUID UNIQUE`, `migrations/20221020180349_create_authorities.sql:4`) |
| Contract (trait) | `src/oxidauth/oxidauth-repository/src/authorities/select_authority_by_client_key.rs:6-17` — `SelectAuthorityByClientKeyQuery` requires `Response = Option<Authority>, Error = BoxedError` (`:7`, blanket impl `:11-17`) |
| Request DTO | `src/oxidauth/oxidauth-kernel/src/authorities/find_authority_by_client_key.rs:8-10` — `FindAuthorityByClientKey { client_key: Uuid }` |
| Domain error intended for the miss | `src/oxidauth/oxidauth-kernel/src/authorities/mod.rs:159-164` (`AuthorityNotFoundError::ClientKey(Uuid)`, `#[derive(Debug)]`), constructors `:166-178`, Display `"authority not found by client_key: {id}"` `:180-191` |
| Pinned repository test | `select_authority_by_client_key/mod.rs:69-85` `it_should_error_not_return_none_for_missing_client_key` (marker `:71-73`; `expect_err` `:74-79`; `format!("{err:?}").contains("RowNotFound")` `:81-84`) |
| Correct sibling (fix template) | `src/oxidauth/oxidauth-postgres/src/authorities/select_authority_by_strategy/mod.rs:15-19` (query block; `fetch_optional` at `:18`) + `:21-23` (`result.map(TryInto::try_into).transpose()?`); its test `it_should_map_strategy_enum_and_return_none_when_absent` `:83`, assert `absent.is_none()` `:111` |
| HTTP 400-not-404 convention (live-probed) | `docs/migration-plan/11-scripts-and-tests.md:112-118` — not-found rows are 400 with `RowNotFound`/… debug strings, "never 404 except unmatched routes" |
| Live pin on this exact path | `src/oxidauth/hurl/tests/authenticate.hurl:49-62` — unknown client key → `HTTP/1.1 400` + `jsonpath "$.errors[0].debug" contains "RowNotFound"` |

Consumer call sites (all in `src/oxidauth/oxidauth-services/src/`), verified one by one:

| Use case | Call site | `None` handling today |
|---|---|---|
| authenticate | `auth/authenticate.rs:107-111` | `.await?` then `.ok_or_else(\|\| AuthorityNotFoundError::client_key(params.client_key))?` |
| register | `auth/register.rs:82-86` | identical `ok_or_else(AuthorityNotFoundError::client_key)` |
| authenticate_or_register (AOR) | `auth/authenticate_or_register.rs:129-137` | `let Some(authority) = ....await? else { return Err("could not find authority by client key") }` — string error, not `AuthorityNotFoundError` |
| oauth2 redirect | `auth/strategies/oauth2/redirect.rs:49-55` | identical `ok_or_else(AuthorityNotFoundError::client_key)` |
| update_password | `auth/strategies/username_password/update_password.rs:87-96` | `let Ok(Some(authority)) = authority_res else { Err("Failed to find authority by client key") }` — collapses Err **and** `None` (and connection failures) into one string |
| totp validate | `totp/validate.rs:86-92` | identical `ok_or_else(AuthorityNotFoundError::client_key)` |
| create_user_authority | `user_authorities/create_user_authority.rs:54-58` | identical `ok_or_else(AuthorityNotFoundError::client_key)` |

HTTP surfaces reached by those use cases (all in `src/oxidauth/oxidauth-api/src/server/api/v1/`):

| Route handler | Error mapping |
|---|---|
| `auth/authenticate.rs:36-43` | `Response::bad_request().error(err.into_error())` — raw error into the 400 body |
| `auth/register.rs:42` | `bad_request().error(err.into_error())` |
| `auth/oauth2/redirect.rs:36` | `bad_request().error(err.into_error())` |
| `auth/oauth2/callback.rs:33-103` (drives AOR) | service error → `ERROR_RESPONSE` (`:26-27`, static text "OAuth2 Error - unable to authenticate…"); underlying error only logged (`:95-100`). A bare `&str` through axum `IntoResponse` carries no explicit status [INFERENCE: axum's default is 200 text/plain; the file's `BAD_REQUEST` tests at `:271-307` exercise query-extractor rejections, not this branch] |
| `totp/validate.rs:73` | `bad_request().error(err.into_error())` |
| `users/authorities/create_user_authority.rs:62-69` | `bad_request().error(err.into_error())` |
| `auth/username_password/update_password.rs:23-26` | errors → `Response::success().payload({ success: false })` — no error string leaks at all |

`err.into_error()` is `IntoOxidAuthError for BoxedError` (`src/oxidauth/oxidauth-kernel/src/error.rs:32-45`): the 400 body embeds `display: format!("{}", err)` **and** `debug: format!("{:?}", err)` verbatim — for sqlx that is `"database row not found"` / `"RowNotFound"`.

## Problem

The repository service promises `Option<Authority>` but implements it with `fetch_one`, which fails on zero rows:

1. Missing `client_key` → sqlx returns `sqlx::Error::RowNotFound` → `mod.rs:20` propagates it via `?` as the `BoxedError` **before** line 24 ever runs. `Ok(…)` is only ever `Ok(Some(…))` (`:24` is unconditional); `Ok(None)` is unreachable in production.
2. Therefore every `None` branch downstream is dead against the real repository. All seven consumers first do `.await?` (or bind the `Result`), so the `RowNotFound` `Err` short-circuits ahead of their `ok_or_else` / `let Some … else` arms. `AuthorityNotFoundError::ClientKey` — purpose-built for this case (`kernel/src/authorities/mod.rs:163`, Display `:187-188`) and imported by six use cases — is **never constructed in any production flow**.
3. The `None` contract is real everywhere else: every service test mock returns `Ok(None)` for the missing case (verified: `auth/authenticate.rs:405` `AuthorityState::Missing => Ok(None)`, `auth/authenticate_or_register.rs:492` `AuthorityMode::Missing => Ok(None)`, `auth/register.rs:280`, `auth/strategies/oauth2/redirect.rs:154-156`, `auth/strategies/username_password/update_password.rs:260-262`, `totp/validate.rs:288`, `user_authorities/create_user_authority.rs:212`). The service layer is unit-tested against a contract the postgres implementation violates.

Net effect: a legitimate business condition ("unknown client key") is reported as a raw database-layer error, and the domain error type the codebase already wrote for it is dead code.

## Analysis

**Mechanism.** In the pinned sqlx (lockfile `sqlx 0.8.6`), `fetch_one` is literally `fetch_optional` plus `ok_or(Error::RowNotFound)` (`sqlx-core/src/query_as.rs:160-170`) — the zero-row case is *defined* as the error this ticket complains about, and sqlx 0.8 has no dedicated ≥2-rows error variant (moot here anyway: `client_key` is `UNIQUE`, so the query returns at most one row). `fetch_optional` is the correct combinator for "zero or one", and its zero case feeds `Option::None`, which `mod.rs:22-24` already type-checks against but never exercises.

**Precedent in the same crate.** `select_authority_by_strategy` — same table, same `Response = Option<Authority>` trait shape — already does it right: `fetch_optional` (`:18`) + `result.map(TryInto::try_into).transpose()?` (`select_authority_by_strategy/mod.rs:21-23`), with a test that asserts the absent case is `None` (`:83`, `:111`). Also `settings/select_setting_by_key`, `permissions/select_permission_by_parts`, `users/select_user_by_username_query` use `fetch_optional` (grep). `select_authority_by_client_key` is the odd one out among by-key lookups that may legitimately miss.

**What callers actually see today, per flow (real repository):**

| Flow (missing/unknown client_key) | Error the caller receives today | Post-fix |
|---|---|---|
| `POST /auth/authenticate` | 400, `errors[0].debug` = `RowNotFound`, `display` = `database row not found` — pinned at `hurl/tests/authenticate.hurl:62` | 400, `errors[0].debug` = `ClientKey(00000000-…)` (derived `Debug`), `display` = `authority not found by client_key: 00000000-…` |
| `POST /auth/register` | 400, same raw `RowNotFound` body (not hurl-covered) | same domain-error body |
| `GET/POST /auth/oauth2/redirect` | 400, raw `RowNotFound` body | domain-error body |
| `POST /totp/validate` | 400, raw `RowNotFound` body | domain-error body |
| `POST /users/{user_id}/authorities` | 400, raw `RowNotFound` body | domain-error body |
| OAuth2 `GET /auth/oauth2/callback/{client_key}` (AOR) | `RowNotFound` reaches the handler via `.await?` (`authenticate_or_register.rs:134`) and is swallowed into the generic `ERROR_RESPONSE` browser text | `None` → the `else` arm at `:135-137` finally executes; browser text unchanged |
| `POST /auth/username_password/update_password` | `let Ok(Some(..)) else` (`update_password.rs:94-96`) already flattens `Err` and `None` alike → 200 `{ success: false }` | identical user-visible result (different internal path) |

So the fix is body-text-only on five 400 endpoints, arm-revival on one, and no-op on two. Status codes do not move — the "not-found is 400, never 404" convention (`docs/migration-plan/11-scripts-and-tests.md:112-118`) is preserved because all handlers keep `Response::bad_request()`; only the embedded error string changes from DB-internals to domain wording.

**Error-fidelity loss (the deeper harm).** Because `RowNotFound` arrives as `Err`, flows that try to classify the miss cannot distinguish "key unknown" from "database down":

- AOR's `let-else` (`:129-137`) can only ever mean "DB said no" — the intended "authority doesn't exist" reading never fires; a transient outage and a deleted authority are byte-identical.
- `update_password`'s `let Ok(Some(..)) else` (`:94-96`) already conflates both, and today *everything* takes the `Err` path — its own distinction between dead repo and missing key is structurally impossible to hit.
- Auth flows with a stale key produce `RowNotFound` in client bodies — a DB driver's error enum leaking through an HTTP API (minor information disclosure about the storage stack, and pure noise to clients who mapped `client_key` to "404-ish, my login config is wrong").

**Cross-tickets.**

- **OXA-000003 (SEC-3, client_key regenerated by `PUT /authorities`)**: its §Problem (`OXA-000003:63-66`) and §Verification (`:114-115`) already cross-reference DATA-8 as the reason a stale key after rotation reports a raw DB string instead of the clean `AuthorityNotFoundError::ClientKey`. This ticket *is* the lookup-side half of that story; fixing it makes OXA-000003's confusing-error symptom read correctly, but changes nothing about rotation itself. The `.await?` short-circuit also means every OXA-000003 repair flow ("log in with the original key") fails with `RowNotFound` today — same symptom, now correctly attributable.
- **OXA-000016 (DATA-3, NULL client_key overwrite)**: a NULL-`client_key` row can never match `WHERE client_key = $1`, so victims of that corruption surface through *this* API — pre-fix as `RowNotFound` errors, post-fix as honest `None`/`AuthorityNotFoundError`. Neither ticket's fix depends on the other.
- **OXA-000012 / OXA-000019** — unrelated (jwks listing panic, private-key ordering); no overlap with this module.

**Who is affected.** Any client whose `client_key` is unknown, rotated (OXA-000003), or rows corrupted (OXA-000016): the login-critical endpoints (authenticate, register, AOR callback, redirect, TOTP validate) plus `POST /users/{id}/authorities`. The default authority seeded from `OXIDAUTH_DEFAULT_CLIENT_KEY` is the highest-traffic key — OXA-000003 documents whole-toolchain lockout scenarios keyed to exactly this lookup failing.

## Impact

- **Everyday unknown-key requests** (typos, stale env vars, post-rotation keys): correct status (400) but the error body names a SQL driver (`RowNotFound` / "database row not found") instead of the domain condition; `AuthorityNotFoundError::ClientKey` never appears in production logs or bodies despite six use cases being written to emit it.
- **Classification blindness**: "authority missing" is not representable as `Ok(None)`, so no in-tree or embedding caller can tell "not found" from "database error" on this service — the entire point of the `Option` return is void.
- **Test-signal distortion**: service tests pass against mocks that implement a contract the shipping repository violates; the pg test (`mod.rs:69-85`) actively pins the violation.
- **Live suite coupling**: `hurl/tests/authenticate.hurl:62` bakes the bug into the acceptance suite; the fix owns one assertion flip there (below). Other `RowNotFound` hurl assertions (`tests/users.hurl:29,146`, `tests/roles.hurl:32,104`, `tests/exchange.hurl:67`, `tests/authorities.hurl:50,159`, `tests/public_keys.hurl:54`) belong to *other* `fetch_one` queries and stay green.
- **Not affected**: found-key behavior (zero change); `update_password` 200-envelope and OAuth2 callback generic-text page (identical output); 400-not-404 convention; `oxidauth-kernel/src/error.rs`'s `into_error_envelopes_a_sourceless_error` unit test (it exercises `IntoOxidAuthError` with its own stand-in type, `error.rs:53-67`, not this query).

## Proposed resolution

**Make `Option` real: `fetch_one` → `fetch_optional` (Option A).** The contract is `Option`, every mock and five of seven consumers are already written for it, and the sibling module is the template — this is the minimal, convention-conforming fix. Dropping `Option` from the trait instead (Option B) would require editing the trait (`select_authority_by_client_key.rs:7,14`), the pg impl, seven `From`/call sites, and the seven test mocks, and would discard the domain-error mapping the codebase deliberately wrote. Rejected.

1. **Product fix** — `src/oxidauth/oxidauth-postgres/src/authorities/select_authority_by_client_key/mod.rs:16-25`, exactly mirroring `select_authority_by_strategy/mod.rs:15-25`:

   ```rust
   let result =
       sqlx::query_as::<_, PgAuthority>(include_str!("./select_authority_by_client_key.sql"))
           .bind(params.client_key)
           .fetch_optional(&self.db.read_pool())
           .await?;

   let authority = result.map(TryInto::try_into).transpose()?;

   Ok(authority)
   ```

   No SQL change; no trait/DTO/wire change. `Error = BoxedError` is retained — real DB failures (connection, decode) still arrive as `Err`, which is exactly the distinction `Option` was bought for.

2. **Consumers: zero required edits** — all seven keep compiling and the five `ok_or_else(AuthorityNotFoundError::client_key(..))` arms plus AOR's `let-else` arm become reachable for the first time. One recommended follow-along for message uniformity (behavior-neutral on its route): `authenticate_or_register.rs:136`

   ```rust
   // before: return Err(format!("could not find authority by client key").into());
   return Err(AuthorityNotFoundError::client_key(params.client_key).into());
   ```

   (import `AuthorityNotFoundError` alongside the existing `find_authority_by_client_key::FindAuthorityByClientKey` import at `:12`; `Box<AuthorityNotFoundError>` coerces into `BoxedError` via `.into()`). The callback handler swallows the message either way (`callback.rs:95-101`), so no test depends on the string. Leave `update_password.rs:94-96` alone — collapsing Err+None there into a generic message is that endpoint's existing (defensible, non-leaky) design; tightening it is out of scope for this ticket.

3. **Pinned-test flips owned by this ticket** (marker census in this module: exactly one, `mod.rs:71-73`; grep `BUG(pinned)` under `src/oxidauth/oxidauth-postgres/src/authorities` — the other markers belong to OXA-000003/OXA-000016):

   - `it_should_error_not_return_none_for_missing_client_key` (`mod.rs:69-85`): rename to `it_should_return_none_for_missing_client_key`; delete the `BUG(pinned)` comment `:71-73`; replace `expect_err` + `contains("RowNotFound")` (`:74-84`) with the sibling's shape (`select_authority_by_strategy/mod.rs:111`): `let absent = repo(&pool).call(&FindAuthorityByClientKey { client_key: Uuid::new_v4() }).await.expect("query should succeed"); assert!(absent.is_none(), "absent client_key must resolve to \`None\`");`
   - `it_should_find_authority_by_client_key` (`:54-67`): unchanged, must stay green (both its `expect`s still hold).
   - **hurl flip**: `src/oxidauth/hurl/tests/authenticate.hurl:49-62` — status block stays `400`; change `jsonpath "$.errors[0].debug" contains "RowNotFound"` → `jsonpath "$.errors[0].display" contains "authority not found by client_key"` (Display is the stable, human-facing field; the `debug` field becomes `ClientKey(…)` and is a derive artifact). Update the file's header comment if it names the old string.
   - Register/docs hygiene once green: drop `BUGS_AND_NOTES.md:40`; `docs/migration-plan/11-scripts-and-tests.md:112-118` stays as written (convention intact — that flow is still 400, just with a different string, and the sentence enumerates other queries' strings). OXA-000003's §Verification `:114-115` "expect 400/`RowNotFound`" expectation for the rotation repro becomes "400 + `authority not found by client_key`" — coordinate wording with that ticket's owner.

4. **Compat.** Library-level: `Response` is unchanged, so external wiring against `SelectAuthorityByClientKeyQuery` is source-compatible; behavior delta is `Err(RowNotFound)` → `Ok(None)` for zero rows — strictly the documented contract. HTTP: status codes unchanged on every route; five 400 bodies change the error string (clients matching on `"RowNotFound"` for *this* endpoint would need updating — the in-tree suite is the single authenticate.hurl assertion above; no SDK/test code greps found for that string against these routes). Callback, update_password, and found-key paths are byte-identical.

## Verification

- **Targeted repository test (the proof):** `cargo test -p oxidauth-postgres authorities::select_authority_by_client_key` against the compose DB (`./src/oxidauth/database_test.sh` runs the crate with `.env` + `MIGRATIONS_ENABLED=true`). Expect `it_should_find_authority_by_client_key` and the renamed `…_return_none_…` green.
- **Negative check that the flip is real:** temporarily restore `fetch_one` — the renamed test must go red with `RowNotFound` — then restore the fix.
- **Service suite untouched:** `bin/unit_test.sh` (or `cargo test -p oxidauth-services`) — all mocks already return `Ok(None)`, so every authority-not-found test (e.g. the `AuthorityState::Missing` paths) must pass before *and* after with no edits; AOR message change (step 2) must not break any test grep.
- **Live repro (before/after):** start the api stack, then
  `curl -s -X POST $BASE/api/v1/auth/authenticate -d '{"client_key":"00000000-0000-0000-0000-000000000000","params":{"username":"x","password":"y"}}'`
  — pre-fix body contains `"debug":"RowNotFound"`; post-fix contains `"display":"authority not found by client_key: 00000000-…"` and the api log shows the `authenticate` span erroring with `ClientKey(...)`, never sqlx. Spot-check the same on `/auth/oauth2/redirect` and `/totp/validate` bodies; `/auth/username_password/update_password` still returns the `{ "success": false }` envelope.
- **Full hurl suite:** `./src/oxidauth/hurl.sh` — only `tests/authenticate.hurl`'s flipped assertion may differ from the pre-fix baseline; the other `RowNotFound` assertions (users/roles/exchange/authorities/public_keys) target other queries and stay green.

## Decision (2026-09-29) — ACCEPTED (Option A: make `Option` real), scheduled; no implementation started
- Reviewed jointly (scout re-verification of all 7 consumers + owner ruling 2026-09-29). Option A chosen over Option B (dropping `Option` from the trait): the contract is real everywhere else — all seven service mocks return `Ok(None)`, five consumers already carry `ok_or_else(AuthorityNotFoundError::client_key)` arms — the repository is the sole violator. Confirmed: `fetch_one` = `fetch_optional` + `ok_or(RowNotFound)` in pinned sqlx 0.8.6; `client_key` is UNIQUE so ≤1 row; `select_authority_by_strategy` (same table) is the working template (`fetch_optional` + `.map(TryInto::try_into).transpose()?`).
- Step 1 approved verbatim: `fetch_one` → `fetch_optional` at `select_authority_by_client_key/mod.rs:16-25`; no SQL/trait/DTO change; `Error = BoxedError` retained — real DB failures stay `Err`, which is the distinction `Option` was bought for.
- Step 2 approved incl. the AOR unification: `authenticate_or_register.rs:136` string error → `AuthorityNotFoundError::client_key(params.client_key).into()` (behavior-neutral on its route — callback handler swallows the message either way, scout-verified). `update_password.rs:94-96`'s deliberate Err+None flatten stays untouched (its own non-leaky design; tightening is another ticket's business).
- Step 3 flip ownership confirmed exhaustive: ONE pg marker (`mod.rs:71-73`, test rename + sibling `is_none()` assertion shape), ONE hurl flip (`authenticate.hurl:62`: `display contains "authority not found by client_key"` — Display is the stable field; `debug` becomes derive-artifact `ClientKey(…)`). **All other `RowNotFound` hurl assertions (users/roles/exchange/authorities/public_keys) target different `fetch_one` queries — hands-off.** Service suite must pass before AND after with zero edits (mocks already speak Option).
- Implementation erratum (2026-09-29, found by implementer, owner-approved mid-flight): the "zero service-test edits" claim was false — `authenticate_or_register.rs:903-907`'s test asserted the exact string step 2 unifies. That ONE assertion flipped to `contains("authority not found by client_key")` (short-circuit `log.ops()` invariant untouched); remainder of the suite passed 269/0 unchanged before and after.
- Status codes never move — 400-not-404 convention (`docs/migration-plan/11-scripts-and-tests.md:112-118`) intact; the fix is body-wording on five endpoints, arm-revival on AOR, no-op on callback/update_password; found-key paths byte-identical.
- Cross-ticket effect recorded for OXA-000003's owner: its Verification "expect 400/`RowNotFound`" rotation repro becomes "400 + `authority not found by client_key`" once this lands — that is WHY this ticket schedules first. OXA-000016's NULL-corruption rows surface through this API pre-fix as `RowNotFound`, post-fix as honest `None`; neither fix depends on the other. Register line `BUGS_AND_NOTES.md:40` struck when green.
- Acceptance: targeted repository test green (`it_should_return_none_for_missing_client_key` + existing found-test untouched); negative check — restoring `fetch_one` reddens the renamed test; `oxidauth-services` suite unchanged-green; curl repro on authenticate/redirect/totp shows the domain-error display, update_password still `{success:false}`; full hurl suite with exactly the one flipped assertion.
