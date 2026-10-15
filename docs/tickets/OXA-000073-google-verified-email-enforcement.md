# OXA-000073 — Google profile path parses `verified_email` and then ignores it: unverified addresses log in and get registered

**Severity:** P3 · **Type:** bug · **Status:** open
**Milestone:** 4 of the Apple-OAuth series (M1 OXA-000004 step 1, M2 OXA-000010, M3 OXA-000072) · register line: SRV-8 (see `docs/tickets/OXA-000044-ms-graph-null-mail.md` for the cross-reference)

## Locations

- `oxidauth-services/src/auth/strategies/oauth2/google/mod.rs:27-36` —
  `GoogleProfile` requires `email: String` + `verified_email: bool`.
- `oxidauth-services/src/auth/strategies/oauth2/google/mod.rs:135-136` —
  the pin: "`verified_email` must be present but is never checked —
  unverified addresses pass this boundary".
- `oxidauth-services/src/auth/strategies/oauth2/google/retrieve_profile.rs:25-29`
  — maps `email`/`given_name`/`family_name` onto `OAuth2Profile` and
  discards `verified_email` on its way out.
- Reached from `authenticate_or_register.rs` (`fetch_profile` →
  `retrieve_google_profile`), i.e. every Google callback before the
  authenticate/register hops.

## Problem

Google flags whether the account's email address is verified. The flag is
deserialized — making the field mandatory so even a scope-incomplete
response aborts — and then dropped. A Google account whose address is
unverified (typo'd at signup, squatted, never confirmed) passes the
profile boundary unchanged: authenticate finds or registers the
`user_authorities` row keyed on that email, and the user receives a full
session. Consequences: an account created for an address nobody controls;
invitations/notifications to dead or hostile mailboxes; registration
itself becomes an effectively anonymous action since email-ownership
proof is skipped for the one identity source that bothers to tell us.

Apple (OXA-000072) rejects `email_verified: false` at its profile
boundary; this ticket brings Google to the same bar. Microsoft is not in
scope: Graph's profile carries no verified flag (identifier robustness is
OXA-000044).

## Proposed resolution

- Refuse the profile when `verified_email == false`, at the profile
  boundary (before the authenticate/register hops), via a typed domain
  error `GoogleProfileError::EmailUnverified` — sanitized display, in the
  `OAuth2StateError`/`AppleAuthError` family.
- The check lives on `GoogleProfile::into_oauth_profile()` (parse stays
  permissive: `TryFrom<JsonValue>` still only enforces the wire shape),
  called by `retrieve_google_profile`.
- Flip the pin: `verified_email=false` is refused at the boundary;
  presence-only pins unchanged.
- AOR-level regression: a fake IdP serving `"verified_email": false`
  errors with the typed error and `find_user_authority` is never called.

## Deliberately NOT in scope

- Making `email`/`verified_email` optional when the `email` scope is
  omitted from authority params (OXA-000044 Step 5).
- Microsoft `mail`/UPN robustness (OXA-000044).
- Google id_token/JWKS verification (deferred register item).

## Impact

- **Who:** Google-authority logins whose address Google itself flags
  unverified — a small, legitimately-broken population.
- **Breaking by design:** their logins fail with the typed error instead
  of succeeding; wire shapes and every verified login are untouched.
