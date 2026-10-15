# OXA-000071 — `AuthorityStatus::Disabled` is stored, displayed, and consulted by nothing: disabling an authority changes no auth path

**Original ID:** new (found 2026-10-15 while wiring the multi-authority example) · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** not yet reviewed — new finding, locations verified by grep + read 2026-10-15; the semantics decision (§Semantics) needs an owner before a recipe is worth writing.


## Locations

Paths relative to `src/oxidauth/`. Every auth path that resolves an authority has the row in hand and reads `settings` only; none reads `status`.

- `oxidauth-services/src/auth/authenticate.rs:107` — `select_authority_by_client_key` → the only authority field consumed is `authority.settings.totp` (`:172`), plus the rest of `settings` for the mint at the bottom of the function.
- `oxidauth-services/src/auth/register.rs:91` — same lookup; consumes `authority.settings.totp` (`:112`) and `authority.settings.jwt_ttl` (`:165`).
- `oxidauth-services/src/auth/authenticate_or_register.rs:136` — same lookup, same shape.
- `oxidauth-services/src/auth/strategies/oauth2/redirect.rs:51` — same lookup for the Google-style dance; consumes `params`/`settings`, never `status`.
- `oxidauth-services/src/auth/strategies/oauth2/authenticator.rs` — performs **no** authority lookup at all (`grep -n "select_authority" →` only the test fixture at `:85`), so nothing in it can consult status.
- `oxidauth-services/src/totp/validate.rs:96` — loads the authority to read `settings.totp` (`:102`) and `settings.jwt_ttl` (`:171`); a `Disabled` authority validates codes and converts a `totp_pending` session into a full one.
- `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:167` — loads by **id** (`select_authority_by_id`) and reads `settings.totp` / `settings.jwt_ttl` (`:178-189`); an existing session keeps refreshing against a disabled authority until its refresh token expires.
- Enforcement exists **only** on the CRUD surface: `oxidauth-services/src/authorities/create_authority.rs:36,134,232` and `update_authority.rs:209` write the column; `oxidauth-postgres/src/authorities/*` stores it. Nothing downstream.

## Problem

`authorities.status` is admin-visible state with no effect. `PUT /api/v1/authorities/{id}` with `{"status":"disabled"}` returns 200, the admin console shows the authority switched off, and every auth path for that authority keeps working: logins, registrations, OAuth2 redirect/callback, TOTP code validation, and refresh-token exchange for sessions that already exist. The word "disabled" therefore means nothing to an operator who believes they just turned a login method off — including in an emergency (compromised Google app registration, broken IdP, a strategy under attack).

This was close to theoretical while an instance had one authority per strategy and the only knob was "delete it". It is not theoretical for a multi-authority install: the example stack now ships four authorities behind four login pages (`oxidauth-example`), where the intended way to retire one page is to set its authority `Disabled`. Today the honest instruction would be "delete the authority", which cascades (`user_authorities.authority_id_id ON DELETE CASCADE`) and destroys the users who signed in through it.

The `LIMIT 1`-without-`ORDER BY` fix (OXA-000022) and this are adjacent but distinct: that one is about *which* row answers; this is about the row that answers not honoring its own state.

## Semantics (needs an owner decision before the fix)

1. **New credential presentations only**, or also existing sessions? Option (a): refuse logins/registrations/OAuth2 callbacks and let sessions ride out `refresh_token_ttl` — matches "take the door away", keeps it auditable, and leaves a way to re-enable without a mass logout. Option (b): `exchange_refresh_token` also refuses — a real kill switch, and the only one available today, but it also strands the bootstrap admin if the pinned default authority is ever flipped (see OXA-000022 for how the default authority can be picked wrongly).
2. **Where in each path does the check sit?** In the password path it must run *after* credential validation, or `Disabled` becomes an oracle that distinguishes "this authority exists and is off" from bad credentials without any credential at all. In the OAuth2 path the ordering question is different (the IdP has already authenticated the user by the time the callback lands) and needs its own decision.
3. **Response shape**: a distinct typed error (`AuthorityDisabledError`, say) mapped to 403, or reuse `AuthorityNotFoundError` to avoid disclosing state? The two answers are "helpful to the operator" and "indistinguishable from a wrong client key"; the client SDK surfaces them differently (`oxidauth-rs`), so the choice should be made once, kernel-side.
4. **Does it apply to the system authority** — can an operator lock `oxidauth:admin` out by disabling the authority the admin was registered into? If (b) is chosen, this needs a guard or at least a loud log line.

## Sketch (not a recipe — rewrite after §Semantics)

The check is one `matches!(authority.status, AuthorityStatus::Disabled)` in each of the six loads listed above, all of which already hold the row, plus one typed error next to `AuthorityNotFoundError` in `oxidauth_kernel/src/authorities/mod.rs`. The blast radius is not the code, it is the four files that are concurrently dirty in the working tree (`auth/authenticate.rs`, `auth/register.rs`, `auth/authenticate_or_register.rs`, `refresh_tokens/exchange_refresh_token.rs`) — sequence this after that work lands.

## Verification (for whoever takes it)

- Per-path unit test with the mock authority returning `AuthorityStatus::Disabled`: refused, and refused *after* the credential check (assert no signing material was fetched, matching the existing `wrong_code_rejects_before_fetching_signing_material` style in `totp/validate.rs`).
- One test per semantics choice, not one shared assertion: `login → disable → login refused`, and under (b) `existing refresh token → refused`; under (a) the same exchange must still succeed, which is the assertion that keeps (a) from silently becoming (b).
- End-to-end on the example stack: `/login/totp` works, `PUT /authorities/{admin_totp id} {"status":"disabled"}`, the same credentials are refused while `/login` (a different authority) still logs in. That is the behavior the four-page example is going to advertise, so it should be the thing that gets tested.
