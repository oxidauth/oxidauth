# OXA-000042 — Update-password swallows repository errors; a failed DB write reports `success: true` to clients

**Original ID:** SRV-6 · **Severity:** P2 (arguably P1 — see Impact) · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; both masks verified intact at HEAD — lookup swallow `:98-110` (outage ⇒ "Failed to find user by username"), `success: res.is_ok()` `:158` + unconditional `success:true` on the TOTP path ⇒ client told password changed when write failed. 3-step typed-error recipe sound (Display-verbatim pins stay green by construction; wire-compatible business arm 200+`{success:false}`, 500+sanitize for infra); first consumer of scheduled 000025's typed not-found (interim sqlx downcast documented), 000021 Step-1 precedence noted, composes w/ deferred 000031 — revisit w/ the username_password family pass)


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

**Register drift (verified, not assumed):** the register cites `oxidauth-services/src/auth/strategies/username_password/update_password.rs:533,642` — both lines are `BUG(pinned)` **test markers**, not the defect: `:533-535` is the comment pinning the masked lookup error, `:642` is the `expect("BUG(pinned): a repository update failure is swallowed into Ok")` message. Same drift pattern as OXA-000031 (CLI-5). The actual defects:

- `oxidauth-services/src/auth/strategies/username_password/update_password.rs:98-110` — user-authority lookup: `let Ok(user_authority) = user_authority_res else { return Err("Failed to find user by username".into()); };`. The `else` arm fires on **every** `Err` from `SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery` and replaces it with a fixed string; the repository's `BoxedError` (sqlx outage, decode failure, typed NotFound once OXA-000025 lands) is dropped on the floor.
- `oxidauth-services/src/auth/strategies/username_password/update_password.rs:153-160` — the write: `let res = self.update_user_authority.call(&user_authority).await; Ok(UpdatePasswordResponse { success: res.is_ok() })`. Any UPDATE failure is converted to `Ok({ success: false })`; the error value is discarded, never returned, never logged (the `#[tracing::instrument]` at `:76` records no fields and the function returns `Ok`).
- `oxidauth-services/src/auth/strategies/username_password/update_password.rs:87-96` — sibling masking one step earlier: `let Ok(Some(authority)) = authority_res else { return Err("Failed to find authority by client key".into()); }` flattens `Err` (DB down) and `Ok(None)` (unknown key) alike. OXA-000021 Step 2 explicitly **disclaims** this site ("Leave `update_password.rs:94-96` alone… tightening is out of scope for this ticket"), so it is owned here.
- **Contrast within the same function:** the TOTP-secret lookup (`:112-117`) propagates its raw repository error with `?`, and `epoch()?` (`:119`), the pepper env read (`:137`), and the serde/argon2 arms (`:133-145`) return `Err`. Three error policies coexist in one 80-line function: propagate raw (`:112-145`), mask to a fixed string (`:94-96`, `:98-110`), fold into `Ok` (`:153-160`).

**The HTTP layer compounds it:**

- `oxidauth-api/src/server/api/v1/auth/username_password/update_password.rs:23-26` — the handler: `Ok(_) => Response::success().payload(UpdatePasswordResponse { success: true })`, `Err(_) => Response::success().payload(UpdatePasswordResponse { success: false })`. Both arms return HTTP **200** (`Response::success()` = `status_code: StatusCode::OK`, `src/xlib/http/src/lib.rs:33-41`; `IntoResponse` honors `status_code`, `:142`). The `Ok(_)` arm **fabricates** `success: true`, discarding the flag the use case computed; the `Err(_)` arm discards the error entirely — not serialized, not logged. No test module exists in this 27-line handler.
- Same-folder convention for contrast: `oxidauth-api/src/server/api/v1/auth/username_password/forgot_password.rs:25-28` maps `Err(err) => Response::bad_request().error(err.into_error())` — HTTP 400 with error detail. update_password is the only auth handler in the folder that 200s its errors.
- The wire envelope: `src/xlib/http/src/lib.rs:10-27` (`Response { success, payload, errors, …, status_code }`); status builders `fail(:44)`, `unauthorized(:55)`, `internal_error(:59)`, `bad_request(:63)`.

**Wiring and consumers:**

- `oxidauth-api/src/provider/services.rs:103-115` — production wiring (`PgTotpSecretRepository`, `PgAuthorityRepository`, `PgUserAuthorityRepository` ×2). No internal server-side caller of `UpdatePasswordService` exists beyond the route (workspace grep).
- `oxidauth-postgres/src/user_authorities/update_user_authority/mod.rs:15-23` — `sqlx::query_as(...).fetch_one(...)` over `UPDATE user_authorities SET params = $3 WHERE user_id = $1 AND authority_id = $2 RETURNING *`: connection death, constraint errors, **and** a vanished row all arrive as `Err(BoxedError)` — exactly the error class `:153-160` swallows.
- `oxidauth-rs/src/client/auth/username_password/update_password.rs:13-26` — SDK raw wrapper (`Ok(result)` pass-through, owned by OXA-000031); its contract test `:37-59` is wiremock-based (`json!({ "success": true })`) and pins only the happy envelope, so it is insensitive to server-side status changes. No hurl test touches this route (grep over `hurl/tests/` returns nothing for `update_password`).

**BUG(pinned) census for this ticket** (`oxidauth-services/src/auth/strategies/username_password/update_password.rs`): `:533-535` (+ assertions `:536-544`) and `:642` (+ `:644-647`) are **this ticket's pins**; `:555-557` is a comment-only pin (no old-password gate; TOTP is the sole credential gate) owned by OXA-000009 — OXA-000009:122 states its Steps 1–3 leave all three markers intact and reserves the `:555` comment text for its own status-gate landing. Related pins elsewhere: `oxidauth-postgres/src/user_authorities/select_user_authorities_by_authority_id_and_user_identifier/mod.rs:58` (OXA-000025), the `contract_raw` leg-2 envelope pin (OXA-000031), turbofish pins (OXA-000035).

## Problem

The update-password flow cannot tell the caller *anything* about why it failed, and — worst case — tells them the opposite of the truth:

1. **Masked lookups.** Any user-lookup `Err` — Postgres down, connection pool exhausted, row-not-found — surfaces as the single string `"Failed to find user by username"` (`:98-110`); the same flattening exists for the authority lookup (`:87-96`). "Wrong username" and "database outage" are one indistinguishable message.
2. **Swallowed write.** A repository UPDATE failure becomes `Ok(UpdatePasswordResponse { success: false })` (`:153-160`). The write failed; the service reports success-with-a-flag, and the error text is gone forever.
3. **Inverted end-to-end signal.** The handler ignores the payload the use case built (`Ok(_)` arm): flow reaches the UPDATE call at all → 200 `{"success":true,"payload":{"success":true}}`; flow bails early → 200 `{"success":true,"payload":{"success":false}}`. Since the *only* path where the UPDATE actually fails is the `Ok` channel, a **failed DB write is reported to the SDK caller as a successful password change**. Conversely, every legitimate refusal (wrong code, unknown user, mismatched confirmation, config missing) is also 200, indistinguishable from any other, with zero diagnostic text.
4. **No operator visibility.** Both swallowing sites drop the error value without logging; the tracing instrument records no error. A sustained DB outage on this route looks like traffic, not failure.

## Analysis

**Mechanism.** `let Ok(x) = res else { … }` and `success: res.is_ok()` are the two textual idioms that turn `Result`'s error channel into a lossy bool/string. Each is locally plausible ("don't leak internals on an unauthenticated route"), but nothing downstream can reconstruct what was lost, and the handler's own `Ok(_) => success:true` fabrication adds a *second* swallow on top of the service's, converting the loss into an inversion.

**What the caller cannot distinguish today (all four are one 200 body):**

| Condition | Service returns | Wire body |
|---|---|---|
| Password write succeeded | `Ok({success:true})` | `{"success":true,"payload":{"success":true}}` |
| **DB error during write** | `Ok({success:false})` | `{"success":true,"payload":{"success":true}}` |
| Wrong/expired TOTP code | `Err("invalid code")` | `{"success":true,"payload":{"success":false}}` |
| Unknown username / **DB error during lookup** / unknown client key / confirmation mismatch / pepper env missing | `Err(…)` | `{"success":true,"payload":{"success":false}}` |

Row-two is the headline: the password was **not** changed, yet the envelope and payload both say it was. The user's next login fails; the reset UI has already shown "done".

**Why the handler's fabrication is load-bearing for the bug:** even if the service today returned `Err` for the update failure (the honest channel), the handler's `Err(_) => {success:false}` on HTTP 200 would *still* lose the distinction between outage and business refusal. Both layers must change; fixing either alone leaves an ambiguity. This is the same envelope-collapse family as OXA-000037 (SRV-1, oauth2 callback 200-on-error); the update_password route is the username_password instance of it.

**Edge cases:**
- **Row vanishes between lookup and write** (user deleted mid-flow): `fetch_one` over `UPDATE … RETURNING` yields `Err(RowNotFound)` → today, success:true (row two). After the fix this is an infra-shaped `Err`; the resolution below should classify it (a vanished row is arguably a business "not found", but it is indistinguishable from an outage at the sqlx level until the repository types it — same taxonomy conversation as OXA-000025 Step 3).
- **Config errors** (`OXIDAUTH_USERNAME_PASSWORD_PEPPER` unset, `:137`): already `Err`, but today rendered as 200 `{success:false}` — a misconfigured server looks like a user error. Post-fix: 500.
- **TOTP-secret lookup already propagates raw** (`:112-117`): today invisible (handler drops all `Err` text); once the handler renders errors, un-sanitized sqlx `Display` would start leaking — the exact oracle OXA-000005 flagged on forgot_password's 400 arm. The fix must wrap infra errors, not echo raw text.
- **Authority-lookup dependency:** until OXA-000021 Step 1 (`fetch_one`→`fetch_optional`) lands, a *missing* client key arrives as `Err(RowNotFound)` too (the dead-None defect), so an infra→500 rule would mislabel unknown keys. Sequence Step 4 after OXA-000021, or gate on `RowNotFound` in the interim.

## Impact

**Who is affected:** external SDK users of `Client::username_password_update_password` (OXA-000031's wrapper hands them the envelope verbatim — `payload.success` is their only signal, and it lies on row two) and any password-reset UI built on the documented 200 body; operators, who see 200s during outages; support, who inherits the "I changed my password and can't log in" tickets. There is no in-repo consumer to update (grep) — all blast radius is the wire contract.

## Proposed resolution

Direction (per assignment and the error-taxonomy line OXA-000021/OXA-000025/OXA-000010 established): **typed errors, propagate lookup failures, DB failure ⇒ `Err` ⇒ 5xx; `success: false` remains only for genuinely business outcomes.**

**Step 1 — typed use-case error (`oxidauth-kernel/src/auth/username_password/update_password.rs`).** Add, following the `AuthorityNotFoundError` pattern (`oxidauth-kernel/src/authorities/mod.rs:159-191`):

```rust
#[derive(Debug)]
pub enum UpdatePasswordError {
    ConfirmationMismatch,
    AuthorityNotFound,
    UserNotFound,
    InvalidCode,
    Repository(BoxedError), // infra: DB outage, decode, config
}
```

`Display` for the four business variants **must reproduce today's strings verbatim** — `"password and password confirmation do not match"`, `"Failed to find authority by client key"`, `"Failed to find user by username"`, `"invalid code"` — and the `Repository` variant must forward its source's `Display` (`write!(f, "{source}")` or `{source}` suffix), because service tests `:478-481`, `:507-510`, `:536-539`, `:575`, `:661-664` assert on exactly these texts. The non-leak property the endpoint has today is preserved: business errors are generic by construction; infra text surfaces only in logs/500 body (Step 3).

**Step 2 — the use case (`oxidauth-services/.../update_password.rs`).**
- `:98-110`: stop masking. `match user_authority_res { Ok(ua) => ua, Err(err) if err.downcast_ref::<UserAuthorityNotFoundError>().is_some() => return Err(UpdatePasswordError::UserNotFound.into()), Err(err) => return Err(UpdatePasswordError::Repository(err).into()) }`. The typed `UserAuthorityNotFoundError` is OXA-000025 Step 3's deliverable (that ticket already claims the mock flip at `:290-301` → typed error); if this ticket lands first, interim-classify via `sqlx::Error::RowNotFound` downcast and leave a `// TODO(OXA-000025)` to swap. Ordering choice belongs to whoever integrates both.
- `:153-160`: replace `Ok(UpdatePasswordResponse { success: res.is_ok() })` with `res.map(|_| UpdatePasswordResponse { success: true }).map_err(|err| UpdatePasswordError::Repository(err).into())`. Delete `res.is_ok()` outright — grep gate below.
- `:87-96` and `:112-117`: same taxonomy — authority `Err` ⇒ `Repository` (see sequencing note), `Ok(None)` ⇒ `AuthorityNotFound` (message preserved); wrap the propagated TOTP error in `Repository` so raw sqlx text stays server-side. `epoch()?`/pepper/serde arms ⇒ `Repository(...)` mapping.
- The `Ok` channel now means exactly one thing: the write happened. `success: false` never appears on `Ok` — the field's remaining wire purpose is Step 3's business-failure body.

**Step 3 — the handler (`oxidauth-api/.../update_password.rs:23-26`).** Render the real distinction:

```rust
match result {
    Ok(res) => Response::success().payload(res),                    // write confirmed
    Err(err) if matches!(err.downcast_ref(), Some(UpdatePasswordError::Repository(_))) => {
        tracing::error!(%err, "update_password repository failure");  // infra text: logs only
        Response::internal_error().error("failed to update password")
    },
    Err(err) => Response::success().payload(UpdatePasswordResponse { success: false }), // business
}
```

The business arm keeps the current 200 + `{"success":false}` payload, so existing SDK/UI flows that check `payload.success` (the contract OXA-000031 documents and deliberately does not interpret, `OXA-000031` Analysis "Trap to keep separate") keep working byte-for-byte; outage paths move to 500 with a sanitized string. If the team prefers 400 for business refusals (the forgot_password/folder convention), that is a stricter but wire-breaking change — flag it for product sign-off rather than bundling; this ticket recommends wire-compat. Note the SDK's `handle_response`-less wrapper (OXA-000031) still passes the 500 envelope back as `Ok`; OXA-000031's `check_envelope` then turns it into a `ClientError` — the two fixes compose, and neither waits on the other.

**Step 4 — coordinate, don't re-ticket.**
- **OXA-000009** adds the users-status gate to this same use case (its Step 2) and owns the `:555-557` comment; its gate returns a business error (user disabled ⇒ business refusal, likely a fifth variant `AccountDisabled` or reuse of `UserNotFound`) through Step 1's enum. Whoever integrates second rebases onto the enum; the two changes touch disjoint lines (`:98-110`/`:153-160` vs. new user-lookup + gate between them).
- **OXA-000021** owns `FindAuthorityByClientKey`'s `Ok(None)` semantics (Step 2 disclaims `:94-96` — that ownership transfers here); land its Step 1 before this ticket's authority arm claims `AuthorityNotFound` for zero rows.
- **OXA-000025** owns the typed `UserAuthorityNotFoundError` at the lookup query (Step 3) — this ticket is its first behavior-different consumer; that is the payoff case for its taxonomy.

**Pinned-test flips (this ticket owns):**
- `user_lookup_failure_is_masked_as_a_username_error` (`:524-551`): the mock's `Err("simulated database failure")` (`:307`) is infra-shaped, so post-fix it must **propagate**. Rename to `user_lookup_infrastructure_failure_propagates`; delete the `BUG(pinned)` comment `:533-535`; invert `:536-544` to `assert!(err.to_string().contains("simulated database failure"))` (or downcast `UpdatePasswordError::Repository`); keep the lookup-keying assertion `:545-550`. Add a second test leg: typed `UserAuthorityNotFoundError` (per OXA-000025) → generic `"Failed to find user by username"` with **no** repo detail — the negative half of the fix (business message stays non-leaky).
- `repository_failure_reports_success_false_instead_of_an_error` (`:630-649`): `expect("BUG(pinned): …")` `:639-642` → `expect_err("a repository update failure must propagate")`; `:644-647` (`!res.success`) → assert on the error (downcast `Repository`, text contains `"simulated update failure"`); keep `:648`. Delete the `:642` marker. Rename `repository_update_failure_propagates_as_an_error`.
- `:555-557` comment pin: **not flipped here** — TOTP-gate behavior is untouched; text changes belong to OXA-000009.
- Stay-green by construction (Display invariants in Step 1): `:464-495`, `:497-521`, `:553-586`, `:589-628`, `:651-669` (the TOTP test asserts the *source* text appears — satisfied by source-forwarding Display).

## Verification

- `cargo test -p oxidauth-services auth::strategies::username_password::update_password` — two renamed/propagating tests green, five stay-green tests unchanged, new NotFound-leg test green.
- `cargo test -p oxidauth-services auth::` — no regressions across `authenticate`/`authenticate_or_register`/`forgot_password` (the enum touches only this use case's error channel; shared mock `oxidauth-services/.../username_password/mod.rs` fixtures untouched).
- `cargo test -p oxidauth-rs username_password_update_password_route_contract` — wiremock contract insensitive to the change (pins the success envelope only).
- Wire proof against a live dev server (`src/oxidauth/database_test.sh`, per OXA-000009's reproduction convention): (a) happy path — `curl -s -o /dev/null -w '%{http_code}' -X POST $HOST/api/v1/auth/username_password/update_password -d '{valid payload}'` → `200 {"success":true,"payload":{"success":true}}` and the hash actually changed (`SELECT params FROM user_authorities …`); (b) wrong code → `200` + `{"success":false}` (contract preserved); (c) outage leg — kill the write pool (e.g. `pg_terminate_backend` / stop postgres after obtaining a code) → **`500`** with `{"success":false,"errors":["failed to update password"]}` and the raw failure visible in server logs — previously `200` + payload `{"success":true}`.
- Grep gates after cutover: `grep -rn "success: res.is_ok()" src/oxidauth/oxidauth-services/src` → zero; `grep -n "BUG(pinned)" oxidauth-services/src/auth/strategies/username_password/update_password.rs` → only the `:555` OXA-000009 comment remains.
- Register hygiene once green: drop `BUGS_AND_NOTES.md:71` (SRV-6).
