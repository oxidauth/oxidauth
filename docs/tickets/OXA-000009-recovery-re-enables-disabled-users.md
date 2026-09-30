# OXA-000009 — Password recovery (and login) ignores `users.status`; "disable" has no enforcement point

**Original ID:** SEC-9 · **Severity:** P2 (arguably P1 — see Impact) · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved, zero findings; 4-layer Disabled gate A-D live — anti-oracle ordering + unspent-code + no-wipe legs all pinned; plain-string errors per deferred-000042 taxonomy; hurl e2e leg deferred to integration; release note in changelog incl. pre-rollout disabled-row count check)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved; scope = 4-layer gate: A `SelectUserByIdQuery`+Disabled-check in update_password w/ services.rs wiring, B user-load moved out of TOTP arm in authenticate, C status gates in exchange_refresh_token + totp/validate, D drop `status` from `AcceptInvitationUserParams`; ~100 LOC, 5-7 files, no migration; release note — disabled accounts fail auth on deploy; land with 000005-Step1/000042/000008 pass)


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

Recovery path (the item's own file):

- `oxidauth-services/src/auth/strategies/username_password/update_password.rs:77-161` — `UpdatePasswordUseCase::update_password`. Order of operations: confirm-match (`:82-84`) → authority by `client_key` (`:87-96`) → user authority by `(authority_id, username)` (`:98-110`) → TOTP secret by `user_id` (`:112-117`) → zero-tolerance TOTP check, `period(600)` (`:119-131`) → rehash (`:133-141`) → `UpdateUserAuthority` write (`:147-156`). **No read of `users.status` anywhere, and the use case holds no user dependency at all** — `new()` (`:53-66`) takes only `(user_totp_secret, authority_by_client_key, update_user_authority, select_user_authority)`.
- `oxidauth-services/src/auth/strategies/username_password/update_password.rs:33-44,46-66` — the four generic repository bounds; none of them is a user query.
- `oxidauth-api/src/provider/services.rs:103-115` — production wiring: `PgTotpSecretRepository` / `PgAuthorityRepository` / `PgUserAuthorityRepository` ×2. No `PgUserRepository` in this service.
- `oxidauth-api/src/server/api/v1/auth/username_password/update_password.rs:13-27` — axum handler: `State` + `Json(params)` only, **no `ExtractJwt`/`ExtractEntitlements`**; every failure returns HTTP 200 with `{success:false}` (`:23-26`).
- `oxidauth-api/src/server/api/v1/auth/username_password/mod.rs:10-11` — `POST /api/v1/auth/username_password/{forgot_password,update_password}`, mounted with no middleware; `oxidauth-api/src/server/mod.rs:31-38` — the only global layer is `CorsLayer::permissive()`.

Why the update *cannot* carry status:

- `oxidauth-kernel/src/user_authorities/update_user_authority.rs:21-25` — `UpdateUserAuthority { user_id, authority_id, params }`; no status field exists.
- `oxidauth-postgres/src/user_authorities/update_user_authority/update_user_authority.sql:1-5` — `UPDATE user_authorities SET params = $3 WHERE user_id = $1 AND authority_id = $2 RETURNING *`.
- `oxidauth-postgres/migrations/20221020204155_create_user_authorities.sql` — `user_authorities` has **no status column** at all (columns: `user_id, authority_id, user_identifier, params, created_at, updated_at`).
- `oxidauth-postgres/src/user_authorities/update_user_authority/mod.rs:15-26` — plain `sqlx::query_as` + `fetch_one`; `grep -i TRIGGER` over `oxidauth-postgres/migrations/` returns nothing, so no DB trigger can touch `users.status` either.

Where `users.status` *is* written (the only writers in the tree):

- `oxidauth-kernel/src/users/mod.rs:80-91` — `enum UserStatus { #[default] Enabled, Invited, Disabled }` + `ENABLED/INVITED/DISABLED` str consts.
- `oxidauth-postgres/src/users/insert_user/mod.rs:20-24` — `status: None` ⇒ `(&UserStatus::default()).into()` = `'enabled'`; `oxidauth-postgres/src/users/insert_user/insert_user.sql` is a plain `INSERT` (no `ON CONFLICT`, no upsert).
- `oxidauth-services/src/auth/strategies/username_password/registrar.rs:128` and `oxidauth-services/src/auth/strategies/oauth2/registrar.rs:97` — every registration mints `status: Some(UserStatus::default())` (Enabled).
- `oxidauth-services/src/users/update_user.rs:41-64` — admin/user update; backfills `status` from the current row when `None` (`:54-56`), so only an explicit `Some(Disabled)` disables. `oxidauth-postgres/src/users/update_user/update_user.sql:1-9` writes `status = $6` unconditionally (DATA-2 territory).
- `oxidauth-kernel/src/invitations/accept_invitation.rs:27-50` — `AcceptInvitationUserParams.status: Option<UserStatus>` is **request-body data** and is passed verbatim into `UpdateUser`; route `oxidauth-api/src/server/api/v1/invitations/mod.rs:16` (`POST /api/v1/invitations/{invitation_id}`, the same unauthenticated accept flow as SEC-2). An invitee can therefore POST `"status": "enabled"` about themselves.

Where status is *read* for enforcement: **nowhere.**

- `oxidauth-services/src/auth/authenticate.rs:103-132` — authenticate never inspects status; it doesn't even load the user in the non-TOTP branch. The only user fetch (`:158-163`, `FindUserById`) exists to build the 2FA webhook payload (`name`/`email`, `:194-219`), and `user.status` is unused there.
- `oxidauth-services/src/auth/strategies/username_password/authenticator.rs:38-63` — verifies the password hash only; receives no `User`.
- `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs` and `oxidauth-services/src/totp/validate.rs` — grep for `FindUserById|SelectUserById|user_by_id` in both files: **zero hits**. Neither flow loads the user row, so neither can honour a status.
- Lookup SQL is unfiltered: `oxidauth-postgres/src/users/select_user_by_username_query/select_user_by_username_query.sql` (`WHERE username = $1`), `oxidauth-postgres/src/users/select_user_by_id_query/select_user_by_id_query.sql` (`WHERE id = $1`), `oxidauth-postgres/src/user_authorities/select_user_authorities_by_authority_id_and_user_identifier/select_user_authorities_by_authority_id_and_user_identifier.sql` (`WHERE authority_id = $1 AND user_identifier = $2`).
- `oxidauth-postgres/migrations/20221019185709_create_users.sql:4,17` — `status VARCHAR(32) NOT NULL` plus a dedicated `users_status_idx`, which no query in the codebase can use. The gate was planned and never written.

**Register drift (verified, not assumed):**

- *"a `Disabled` user with a valid code flips status to `Enabled` via the TOTP-gated update"* — **not reproducible in the current tree.** The recovery update writes `user_authorities.params` only (SQL and struct above); `user_authorities` has no status column, `insert_user.sql` has no upsert, and no migration defines a trigger. Nothing reachable from `update_password.rs` writes `users.status`. Confirmed by reading the whole file plus its wiring (`services.rs:103-115`) and grepping every non-test `UserStatus` reference in `oxidauth-services`, `oxidauth-api`, `oxidauth-kernel`, `oxidauth-postgres` — the only non-test occurrences are the `Enabled`/`Disabled` string-mapping impls, the CRUD backfills, and the user-minting defaults listed above.
- Probable origin of the audit's wording: `UserStatus`'s `#[default] = Enabled` (`oxidauth-kernel/src/users/mod.rs:82-87`) leaking into every user-minting path (`insert_user/mod.rs:23`, `registrar.rs:128`, `oauth2/registrar.rs:97`). That is a real "silently Enabled by default" hazard, but it belongs to registration/insert, not to recovery. Git history cannot settle it: the tree predates a squashed layout move (`c9312e0`), so the audited revision is unreachable.
- The underlying security claim **is** true and is *broader* than the register states: because status is never read, a `Disabled` account is not disabled. The register describes a symptom; the defect is the missing gate.

## Problem

`users.status` is write-only. The server stores and displays `disabled` but nothing in any credential path ever checks it, so password recovery — and login, refresh, and TOTP validation — work unchanged for a disabled account:

1. `POST /api/v1/auth/username_password/update_password` is reachable anonymously (no extractor on the route, no global layer). Its only gate is possession of a valid 600-second TOTP code for the target user.
2. That code is obtainable for anyone who knows a `user_id` via `POST /api/v1/auth/username_password/forgot_password`, which returns the live code in the response body (SEC-5). The recovery request itself needs `{ username, client_key, code, password, password_conf }` (`oxidauth-kernel/src/auth/username_password/update_password.rs:10-17`) — `username` and `client_key` are configuration-level values, not secrets.
3. `update_password` never consults the user row, so a `Disabled` user's password hash is replaced just like anyone else's (`:147-156`).
4. The victim (or the attacker holding the new password) then calls `POST /api/v1/auth/authenticate`: `AuthenticateUseCase` checks authority + password + TOTP, **not status** (`authenticate.rs:103-132`), and mints a full JWT with the user's permission tree. Refresh-token exchange (`exchange_refresh_token.rs`) and `/totp/validate` don't load the user row at all, so a session obtained while disabled keeps renewing.
5. Admins cannot express the intent they were sold: `PUT` user update with `status: "disabled"` (`oxidauth-services/src/users/update_user.rs:41-64`) writes the column, indexes it, and has zero behavioural effect. Disabling a compromised or departed employee does nothing.
6. Compounding the write side: `POST /api/v1/invitations/{invitation_id}` accepts `user.status` from the request body (`oxidauth-kernel/src/invitations/accept_invitation.rs:34,48`) and forwards it to `UpdateUser`, so an invitee can write their own status — the closest verifiable thing in the tree to "self re-enable".

## Analysis

**Mechanism.** There is no "status gate" layer to gate in. Authentication is composed of authority lookup → strategy authenticator → JWT mint, and the `User` entity is only fetched on the TOTP branch of `authenticate` (`:158-163`) for cosmetic reasons (the webhook greeting). Strategies receive `(&JsonValue, &Authority, &UserAuthority)` (`oxidauth-kernel/src/auth/mod.rs:37` — `pub trait Authenticator: UserIdentifierFromRequest`), i.e. by construction they *cannot* see `users.status`. The recovery use case is even narrower — it holds no user repository (`:33-44`), so today it is not merely unchecked, it is *un-checkable* without a dependency change.

**Edge cases and adjacent findings (verified):**

- **`Invited` is equally unenforced.** Whatever gate is chosen must state whether `Invited` users may authenticate; they currently can, and `accept_invitation` writes the new status from request data. A gate that only rejects `Disabled` leaves `Invited` accounts with normal login.
- **Recovery only works for TOTP-enrolled users.** `FindTOTPSecretByUserId` uses `?` (`update_password.rs:112-117`) → missing secret propagates → handler swallows it into `200 {success:false}` (`api/.../update_password.rs:25`). A disabled user with no authenticator enrolled simply gets a different failure mode, not a block.
- **Duplicate `totp_secrets` rows (SEC-6)** make `fetch_one` return an arbitrary secret, so a recovery code can be minted from one row and validated against another. Any test added here that enrolls TOTP through the repository must not assume one secret per user.
- **Recovery overwrites the whole `params` jsonb** with `UserAuthorityParams { password_hash }` (`:143-151`), i.e. `{"password_hash": ...}`. That is currently lossless *because* registration writes the identical single-key object (`registrar.rs:52-56`, and `UserAuthorityParams` is declared at `oxidauth-services/src/auth/strategies/username_password/mod.rs:46-48` with no `deny_unknown_fields` anywhere in the workspace). If any future flow stores extra keys per user authority, recovery silently drops them. Not a fix requirement here; a one-line comment/typed-merge note is enough.
- **`user_identifier` is UNIQUE** (`create_user_authorities.sql`, tightened by migration `20240619063635`), so the username lookup resolves exactly one row — there is no ambiguity for a gate to get wrong on "which user".
- **Failure-shape noise:** all recovery failures collapse to `200 {success:false}`, and lookup failures are relabelled "Failed to find user by username" (`BUG(pinned)` at `update_password.rs:533-535`). A new "account disabled" rejection inherits that shape unless the register's error-shape work lands first; pick a distinct error and keep the same envelope to avoid a new oracle.
- **No e2e coverage.** `src/oxidauth/hurl/` (incl. `tests/`) has no request for either `update_password` or `forgot_password` — grep confirms; coverage is service unit tests plus the wiremock route contracts in `oxidauth-rs/src/client/auth/username_password/{update_password,forgot_password}.rs`.

**Who is affected.** Every deployment exposing `/api/v1`, i.e. all of them: there is no feature flag, no config switch, and no middleware slot where an operator could re-impose the check. Operators who disable an account (offboarding, compromise, abuse, non-payment) get no effect at all and no log line saying so.

## Impact

- **Security:** the account-disable control does not exist server-side. A disabled/stolen/departed credential keeps working — full login, JWT, refresh loop, TOTP validation, *and* self-service password recovery. Combined with SEC-5's anonymous code oracle, a disabled account is still fully takeover-able. Register rates this P2 as written; the missing-gate reading is closer to P1 (the register's own P1 definition is "auth/security"), and it should be re-rated when the product decision below is taken.
- **Trust/audit:** administrators see `status: disabled` echoed back by `/users` endpoints and reasonably believe it is enforced. Nothing warns them otherwise, and `users_status_idx` makes the schema look deliberate.
- **Data:** none — no status value is written or lost by this path (that is precisely the drift finding).
- **Compatibility:** adding a gate changes login outcomes for accounts currently sitting at `disabled` — in a live deployment those users are silently logging in today. Any fix needs a release note and a check of existing `disabled` rows before rollout.

## Proposed resolution

**Step 0 — decide the policy (product owner, one line).** Two defensible answers:
- *(A) Disable means disable:* no authentication, no token refresh, no TOTP validation, **no password recovery**. Re-enabling is an admin-only action (`PUT` user update). This is the standard IAM behaviour and the one this ticket assumes.
- *(B) Disable blocks login but recovery may proceed:* lets a locked-out user reset and then discover they're still disabled — an information leak and a support loop. Only worth it if "disabled" is meant as a soft "must reset password" state, which the string does not suggest.
Recommend (A). Under (A) the recovery path must fail closed.

**Step 1 — enforce status at the credential boundary (the real fix), not just in recovery.** Add a single check wherever a `User` row is (or can cheaply be) loaded:

1. `oxidauth-services/src/auth/authenticate.rs` — move the `user_by_id` fetch out of the `TotpSettings::Enabled` arm (`:145-163`) so it always runs, then reject before `Jwt::builder()`:
   ```rust
   let user = self.user_by_id.call(&FindUserById { user_id: user_authority.user_id }).await?;

   if matches!(user.status, UserStatus::Disabled) {
       return Err(AuthenticationError::AccountDisabled.into()); // new domain error
   }
   ```
   Place it **after** the authenticator call (`:130-132`) so a wrong password still reports the password error (no status oracle for non-credential holders), and log at `warn!` with `user_id` so disabled-login attempts are auditable. Decide the `Invited` case per Step 0 and put it in the same `match`, not a later patch.
2. `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs` — the refresh path currently never loads the user (bound is `SelectRefreshTokenByIdQuery`, `:27,34`; the SQL at `oxidauth-postgres/src/refresh_tokens/select_refresh_token_by_id/select_refresh_token_by_id.sql` is a bare `SELECT * FROM refresh_tokens WHERE id = $1`), so a disabled user's sessions renew forever. Cheapest fix: extend that SQL to `SELECT rt.*, u.status FROM refresh_tokens rt JOIN users u ON u.id = rt.user_id WHERE rt.id = $1` and reject `disabled` in the use case — one added column, one extra field on the row type, no extra round trip per refresh. If the join is judged too invasive for the first cut, an explicit `SelectUserByIdQuery` dependency is correct and costs one read per refresh; do **not** skip this leg, otherwise Step 1 is cosmetic.
3. `oxidauth-services/src/totp/validate.rs` — same shape; it mints the privileged full-entitlement JWT after 2FA, so it must not honour a disabled user. It holds no user query today (grep: zero `*UserById` hits; its bounds at `:35-39` are totp-secret / private-key / permission-tree / authority / refresh-insert), so this needs a `SelectUserByIdQuery` bound added to `ValidateTOTPUseCase` plus `PgUserRepository::new(db.clone())` at its wiring block, `oxidauth-api/src/provider/services.rs:131-143`.

**Step 2 — gate recovery in `update_password.rs` (the item as filed).** Under policy (A), recovery is a credential path and must fail closed. The use case cannot see the user, so:

- Add a fifth dependency to `UpdatePasswordUseCase` (`oxidauth-services/.../update_password.rs:33-44` struct, `:46-66` `new()`, plus the `where` clauses at `:46-51` and `:69-74`): `W: SelectUserByIdQuery` (mirror `authenticate.rs:34,49,100`, which uses `oxidauth_repository::users::select_user_by_id_query::SelectUserByIdQuery` with `oxidauth_kernel::users::find_user_by_id::FindUserById`).
- After the user-authority lookup succeeds and **before** the TOTP check (`update_password.rs:110-112`), fetch and gate:
  ```rust
  let user = self.user_by_id.call(&FindUserById { user_id: user_authority.user_id }).await?;

  if matches!(user.status, UserStatus::Disabled) {
      return Err("account is disabled".into()); // surfaces as 200 {success:false}, same as every other failure
  }
  ```
  Placing it before `is_valid` means a disabled user cannot even spend/confirm a code, and the code never reaches the password write. `UserStatus` is already reachable via `oxidauth_kernel::users`.
- Wire it: `oxidauth-api/src/provider/services.rs:107-112` — add `PgUserRepository::new(db.clone())` (already imported at `:21`) as the new argument. No other caller constructs `UpdatePasswordUseCase` (grep: only `services.rs` and the service's own test harness).
- Keep ` oxidauth-services/src/user_authorities/update_user_authority.rs` untouched — it is a pure passthrough (`:27-31`) and the admin route; the recovery bug is not there.

**Step 3 — close the write-side hole: status must not be client-supplied.** Drop `status` from `AcceptInvitationUserParams` (`oxidauth-kernel/src/invitations/accept_invitation.rs:34`) and from the `From<(Uuid, &AcceptInvitationUserParams)> for UpdateUser` mapping (`:48`), letting `UpdateUserUseCase` backfill the stored value (`oxidauth-services/src/users/update_user.rs:54-56`); status transitions then belong to the admin `update_user` route only. Serde on that struct is permissive (no `deny_unknown_fields` in the workspace), so existing clients that still send `"status"` keep deserialising — the field just stops being honoured. Note this in the changelog: **behaviour change** for anyone who relied on inviting with a preset status.

**Step 4 — defaults audit (the drift's kernel).** `#[default] Enabled` (`oxidauth-kernel/src/users/mod.rs:82-84`) means every `status: None` insert is a live account, and both registrars pass `Some(UserStatus::default())` explicitly (`username_password/registrar.rs:128`, `oauth2/registrar.rs:97`). Do **not** change the `Default` impl blindly — `oxidauth-postgres/src/users/insert_user/mod.rs:23` and the `create_user`/bootstrap paths depend on it, and `oxidauth-kernel/src/users/mod.rs:248` pins `UserStatus::default() == Enabled`. The minimal honest change is to make the registrars' intent explicit (`UserStatus::Enabled`, with a comment that registration implies enabled) and record the remaining question — *should self-service registration ever produce `Invited` instead?* — as a separate product item, since that is what would make `Invited` meaningful.

**Pinned-test handling (grep-verified).** The register's "NOT pinned" is confirmed for the recovery status question: no `BUG(pinned)` marker in `update_password.rs` or its wiring asserts anything about status. Adjacent markers that this ticket must not disturb, and the flips it does cause:

- `oxidauth-services/src/auth/strategies/username_password/update_password.rs:533-535` (`BUG(pinned)` — lookup error relabelled "Failed to find user by username"), `:555` (comment pin: no old-password gate, TOTP is the sole gate), `:642` (`BUG(pinned)` — update failure swallowed into `Ok(success:false)`). All three survive Steps 1-3. Only touch `:555`'s comment text if Step 2's status gate lands next to it, so the comment still reads "the only *credential* gate is the TOTP code; the status gate lives above it".
- `oxidauth-services/src/users/update_user.rs:235` and `oxidauth-postgres/src/users/update_user/mod.rs:105` (`BUG(pinned)` — unconditional overwrite + backfill list). Unaffected by this ticket; Step 3 changes only what the *invitation* mapping sends, and `users/update_user.rs:241`'s `status: Some("disabled")` assertion stays valid.
- `oxidauth-services/src/auth/strategies/username_password/registrar.rs:284` (`BUG(pinned)` — duplicate username not rejected by the registrar). Independent; leave for the duplicate-registration item.
- `oxidauth-postgres/src/user_authorities/update_user_authority/mod.rs:46-94` `it_should_overwrite_params` — pins that recovery's write replaces `params`. Still correct after Step 2 (the gate is in the service, not the query); add the "status untouched" assertion at the *service* level instead, since `user_authorities` has no status to preserve.
- New assertions that must land with the fix (no existing test to flip): `oxidauth-services/src/auth/strategies/username_password/update_password.rs` test module — extend the `use_case(...)` harness (`:390-421`) with a `MockUserById` returning a configurable status, and assert (a) a `Disabled` user with a **valid** code yields `Err` and an **empty** `update_requests` log (write never attempted), (b) `Enabled` still writes (existing `valid_code_replaces_the_stored_password_hash`, `:589`), (c) the status check runs before the TOTP secret lookup (assert call order, mirroring the op-order style used in the `totp::validate` suite) so a disabled user is not distinguished by a different error surface. The same three legs for `authenticate`, `exchange_refresh_token`, and `totp::validate`.
- `oxidauth-postgres/src/user_authorities/select_user_authorities_by_authority_id_and_user_identifier/mod.rs:58` `BUG(pinned)` (plain-VARCHAR-identifier / `fetch_one` semantics) — untouched; note it is why the recovery lookup resolves exactly one user.

**Compat/migration.** No schema change, no data migration. Deployment caveat that must be in the release note: run `SELECT count(*) FROM users WHERE status = 'disabled'` first — those accounts are live today and will start failing to log in the moment Steps 1-2 ship; that is the fix working, but it will look like an outage to whoever disabled them months ago. If `Invited` is decided to be non-authenticating, the same query applies to `'invited'`. SDK surface: `oxidauth-rs/src/client/auth/username_password/update_password.rs` needs no change (same DTO); the invitation wrapper must stop sending `status` after Step 3.

## Verification

No hurl coverage exists for either recovery route (`src/oxidauth/hurl/tests/` has no `update_password`/`forgot_password` request) — prove the fix at service level plus one new e2e leg.

- Reproduce the hole **before** fixing (dev server with a test DB via `src/oxidauth/database_test.sh`): seed a user, enrol a TOTP secret, `UPDATE users SET status='disabled' WHERE username='…'`; then `curl -s -X POST $HOST/api/v1/auth/username_password/forgot_password -d '{"user_id":"<uuid>"}'` → `200` + `payload.code`; `curl -s -X POST $HOST/api/v1/auth/username_password/update_password -d '{"username":"…","client_key":"<uuid>","code":"<code>","password":"…","password_conf":"…"}'` → `{"success":true}`, and `POST /api/v1/auth/authenticate` with the new password → a JWT. Post-fix: recovery → `{"success":false}` (or the new domain error), authenticate → rejected, and `SELECT params FROM user_authorities WHERE user_id=…` **unchanged** (proves the gate runs before the write).
- `cargo test -p oxidauth-services --lib auth::strategies::username_password::update_password` — the existing seven tests (mismatch, unknown client key, masked lookup failure, invalid code, valid code, swallowed update failure, secret-lookup failure) plus the new disabled-user legs; the valid-code test at `:589` is the regression guard that `Enabled` is unaffected.
- `cargo test -p oxidauth-services --lib auth::authenticate` / `--lib refresh_tokens::exchange_refresh_token` / `--lib totp::validate` — the Step 1 gates.
- `cargo test -p oxidauth-services --lib invitations::accept_invitation` — Step 3: the captured `UpdateUser` snapshot (`oxidauth-services/src/invitations/accept_invitation.rs:133-141` `status: Option<String>`) must assert `status: None` regardless of the request body.
- With a test DB: `cargo test -p oxidauth-postgres --lib users::update_user` and `--lib user_authorities::update_user_authority` — confirm no status write regressed and `it_should_overwrite_params` still passes.
- New e2e leg in `src/oxidauth/hurl/tests/` (mirroring `authenticate.hurl`'s setup/vars style): authenticate as admin, disable a seeded user via the user-update route, then assert `401`/rejection on `authenticate` and `{"success":false}` on `update_password`. This is the first durable coverage of the recovery route at all.
- `cargo test -p oxidauth --lib auth::username_password` (package `oxidauth`, `oxidauth-rs/`) — route/verb/payload contracts for the recovery wrappers; must stay green (DTO unchanged), and the invitation wrapper's canned payload must drop `status` after Step 3.
