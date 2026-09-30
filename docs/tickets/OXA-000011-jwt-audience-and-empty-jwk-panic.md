# OXA-000011 — Jwt::decode can never accept an `aud` claim; empty-key panic note is stale

**Original ID:** SEC-11 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — audience plumbing + typed JwtErrorKind; OXA-000058 folds into this site) · reviewed 2026-09-30 · DEFERRED (owner decision; audience/typed-error claims verified intact at HEAD — decode_with_audience + AUDIENCE const + JwtErrorKind recipe sound, ~6 files; panic-note half confirmed STALE — `decode_with_default_jwk_set` never existed, empty set yields stringly Err; OXA-000058 remains folded here — revisit before building any aud-scoped feature)


## Locations

Register cites `oxidauth-kernel/src/jwt/mod.rs:465 (+ panic note)`.

**Register drift (confirmed).** Line 465 is the `BUG(pinned)` marker inside the test
`tokens_with_an_audience_claim_are_unverifiable_by_decode` (test spans
`jwt/mod.rs:452-472`; marker comment `:465-468`; the failing-decode assertion is `:469-471`).
The product defect is `Jwt::decode` at `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:59-66`,
specifically line 63: `decode(token, &key, &Validation::new(Algorithm::RS256))`.

Verified current locations (workspace root-relative):

- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:59-66` — `Jwt::decode`; `:63` builds `Validation::new(RS256)` with no audience configured.
- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:33-34` — `Jwt.aud: Option<String>`, `skip_serializing_if = "Option::is_none"` (omitted from the payload when unset).
- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:169-173` — `JwtBuilder::with_audience`. **Zero production callers** — only the kernel test uses it.
- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:68-81` — `Jwt::decode_with_public_keys`; `:74` `Err(_) => continue` discards every per-key failure kind; `:78-80` one stringly-typed sentinel `"no valid public key found"` for both "no keys at all" and "no key matched".
- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:84-101` — `JwtError` is a bare `message: String`; `JwtError::new(err)` stringifies jsonwebtoken errors verbatim (the test observes `"JwtError: InvalidAudience"` at `:471`).
- Mint sites (all stamp `iss: "oxidauth"`, **none** stamp `aud`): `oxidauth-services/src/auth/authenticate.rs:141-143`, `oxidauth-services/src/auth/register.rs:114-117`, `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:154-157`, `oxidauth-services/src/totp/validate.rs:139-142`. `oxidauth-services/src/auth/authenticate_or_register.rs` mints no token of its own — it forwards `auth.jwt`/`result.jwt` (`:188`, `:221`).
- Verifier call sites: `oxidauth-api/src/middleware/permission_extractor.rs:38-44` (failure → bare 401, no log), `oxidauth-rs/src/axum/extract/mod.rs:54-67` (401, error-logged), `oxidauth-rs/src/client/mod.rs:269-272` (`auth()`) and `:376-382` (`refresh()`).
- Library mechanism: jsonwebtoken 9.3.1 (`oxidauth-kernel/Cargo.toml:20`), `src/validation.rs:309-317`: with `validate_aud` on (default) and `Validation.aud = None`, any token whose `aud` parses is rejected — `(TryParse::Parsed(_), None) => Err(InvalidAudience)`; the arm comment quotes RFC 7519 §4.1.3. `Validation::new` (`validation.rs:112-120`) requires only `exp`, leeway 60. An **absent** `aud` claim passes even when an expected audience is configured (`:309/:328`, `_ => {}` arm).

**The panic half of the register entry does not exist in this codebase.** `decode_with_default_jwk_set` appears nowhere in the working tree (case-insensitive grep for `jwk` finds only "jwks" comments/tests) and never existed in git history (540 commits; `git log --all -S "jwk"` → 0 hits, pickaxe verified against a known-present string). `Jwt::decode` takes a single PEM byte slice — there is no key set to be empty — and contains no unwrap. `decode_with_public_keys` on `&[]` runs the loop zero times and returns the `Err` sentinel at `:78-80`; this is *pinned as correct behavior* by `decode_with_public_keys_tries_each_key` at `jwt/mod.rs:587-588`. No wired path (api middleware, axum extract, client) unwraps an empty key list. Treat "+ panic note" as register drift — likely inherited from the upstream oxidauth design, which had a JWK-set decoder; the closest real defects here are the untyped, indistinguishable error taxonomy described below.

## Problem

1. **Self-rejecting audience.** `Jwt::decode` configures no expected audience. jsonwebtoken 9.3.1 then rejects *any* token that carries an `aud` claim with `InvalidAudience` (validation.rs:315-317 — the RFC 7519 §4.1.3 rule: a verifier that doesn't identify itself with a value in `aud` must reject). Since `JwtBuilder::with_audience` (`:169-173`) is a public API, the kernel's own builder mints tokens its own verifier cannot read. Today this is latent only because no production mint site stamps `aud`; the moment any integrator (or a future feature) uses `with_audience`, every bearer of that token is 401'd everywhere — api middleware (`permission_extractor.rs:43-44`), the `oxidauth-rs` extractors and client — with the true cause erased by `Err(_) => continue` (`:74`), surfacing as the misleading sentinel "no valid public key found".
2. **Asymmetric claim policy.** `iss` is hardcoded `"oxidauth"` at every mint site yet never validated (`iss` mismatch arm at validation.rs:292-303 falls through when unconfigured — accepted), while `aud`, the one claim jsonwebtoken *forces* verifiers to care about, can never be validated. Verification is effectively signature + `exp`/`nbf` with 60 s leeway only (`exp` is the sole default-required claim; leeway behavior pinned at `jwt/mod.rs:543-547` and relied on in `oxidauth-rs/src/client/mod.rs:934-935`).
3. **Error opacity, not panic.** Where the register claims a panic, the actual issues are: `JwtError` erases `jsonwebtoken::ErrorKind` into a string; `decode_with_public_keys` maps "zero public keys configured" and "N keys, none matched" to the identical sentinel, and discards the per-key error kinds. An empty `public_keys` table silently turns every authenticated endpoint into 401 (permission_extractor maps *all* failures, including list-service failure at `:41`, to `UNAUTHORIZED`) — a misconfiguration that is indistinguishable from credential rejection, for clients and for logs.

## Analysis

Mechanism chain: `with_audience` → `Jwt.aud = Some(..)` → serde writes `"aud"` into the claims JSON → `jsonwebtoken::decode` parses it (`TryParse::Parsed`) → `validate` matches `(Parsed(_), None)` → `InvalidAudience`. The `(None, None)` (token silent about aud) and "expected set, token absent" combinations both pass, which is what makes a required-audience cutover backward compatible: tokens minted before the fix (no `aud`) continue to verify after the verifier starts expecting `aud`.

The direction of the fix has two coherent readings, and they compose:

- **Optional aud validation (compat floor).** Give the verifier an explicit audience policy so `None` means "I do not check aud, and do not reject tokens that carry it" (`validation.validate_aud = false`, validation.rs:306-308 short-circuit) instead of jsonwebtoken's fail-closed default. This alone un-breaks `with_audience`-minted tokens everywhere and is a strict superset of currently-accepted tokens — zero rollout risk, but leaves `aud` unchecked (RFC 8725 §3.4 recommends aud validation).
- **Required audience (the boring OAuth2-correct design).** Mirror the hardcoded issuer with a kernel constant (e.g. `pub const AUDIENCE: &str = "oxidauth";` next to the existing `DEFAULT_EXP_IN_SEC`, `:25`), stamp it at all four mint sites via the builder, and have `decode`/`decode_with_public_keys` call `validation.set_audience(AUDIENCE)`. Old aud-less tokens still verify (validation.rs `_ => {}` arm), so deployment can upgrade verifier and minters in one kernel release without token invalidation. Making `aud` a *required spec claim* would break pre-cutover tokens — deliberately out of scope; recommend presence-tolerant validation.

Related-but-separate items this ticket should NOT absorb: the base64-vs-raw-PEM wire-format asymmetry in `oxidauth-rs` client `refresh()` (pinned at `oxidauth-rs/src/client/mod.rs:1290-1293`, tracked under its own item), the Gz entitlements round-trip asymmetry (`jwt/mod.rs:495-498`), and the 60 s leeway pin (`jwt/mod.rs:543-547`) — the fix MUST leave `Validation::new`'s leeway/`exp`/RS256-only-`alg` settings untouched (the `algorithms: vec![RS256]` from `Validation::new` already blocks `alg`-confusion; keep it).

Who is affected today: nothing in the default deployment breaks (aud is never minted). Affected surfaces are (a) downstream `oxidauth-kernel` users of the public `with_audience` API — a loaded gun; (b) operators — empty/misconfigured `public_keys` produces silent global 401 with no typed signal; (c) auditors — the verifier validates less than the code's naming implies.

## Impact

- **Latent correctness/security trap (P2):** any `with_audience` token is permanently unreadable; the failure mode masquerades as "forged token" ("no valid public key found"), so the bug costs debugging time and hides audience-based capability separation that the `Jwt` struct clearly intends to support (`aud` field + builder exist for a reason).
- **No DoS/panic exists:** the register's panic claim is false for this tree; empty key sets degrade to `Err` → 401. Recorded here so nobody "fixes" a phantom panic; the real availability note is that a *missing* key table rows makes the whole API unauthenticatable, which is fail-closed (safe) but unobservable.
- **Data/UX:** none directly; no stored data involved. Client-visible change from the fix: tokens gain `"aud":"oxidauth"` in their payload (public, decoded by clients; `oxidauth-rs` decodes into the shared `Jwt` struct, so it is format-compatible by construction).

## Proposed resolution

1. **Kernel — audience plumbing (`jwt/mod.rs`).** Thread an expected audience through verification:
   ```rust
   pub fn decode(token: &str, key: &[u8]) -> Result<Jwt, JwtError> {
       Self::decode_with_audience(token, key, None)
   }
   pub fn decode_with_audience(token: &str, key: &[u8], aud: Option<&str>) -> Result<Jwt, JwtError> {
       let key = DecodingKey::from_rsa_pem(key).map_err(JwtError::new)?;
       let mut validation = Validation::new(Algorithm::RS256);
       match aud {
           Some(expected) => validation.set_audience(expected),
           None => validation.validate_aud = false, // explicit "no aud policy", replaces fail-closed default
       }
       let result: TokenData<Jwt> = decode(token, &key, &validation).map_err(JwtError::from)?;
       Ok(result.claims)
   }
   ```
   Mirror the signature on `decode_with_public_keys(token, keys, aud: Option<&str>)`. Keep the 2-arg `decode` as the compat shim (≈20 service-test call sites keep compiling; see pin list).
2. **Kernel — required-audience default (product decision).** Add `pub const AUDIENCE: &str = "oxidauth";`; stamp `.with_audience(AUDIENCE.into())` at the four mint sites (`authenticate.rs:141`, `register.rs:114`, `exchange_refresh_token.rs:154`, `totp/validate.rs:139`); verifiers pass `Some(AUDIENCE)` — `permission_extractor.rs:43`, `oxidauth-rs/src/axum/extract/mod.rs:63`, `oxidauth-rs/src/client/mod.rs:270` and `:377`. Compat: aud-less pre-cutover tokens verify unchanged; one atomic kernel release covers mint+verify. (Validating `iss` the same way is a separable decision — all tokens already carry `"iss":"oxidauth"`, but it is not required to close this ticket.)
3. **Kernel — typed errors, kill the opaque sentinel.** Extend `JwtError` with a kind: `JwtErrorKind { InvalidSignature, ExpiredSignature, ImmatureSignature, InvalidAudience, Other }` mapped from `jsonwebtoken::ErrorKind` (replaces the `JwtError::new` stringification for decode paths, `:90-94`), and split `decode_with_public_keys`'s terminal error: empty `keys` ⇒ `JwtErrorKind::NoPublicKeysConfigured`, non-empty-but-all-failed ⇒ `NoMatchingPublicKey` carrying the last inner error kind (stop discarding at `:74` `Err(_) => continue`).
4. **API/clients — surface it.** `permission_extractor.rs` currently maps *everything* to `UNAUTHORIZED` (`:41`, `:44`): map `NoPublicKeysConfigured` to `503` + `tracing::error!` (misconfiguration is not the caller's fault) and log the `JwtErrorKind` on 401s; `oxidauth-rs` extractors keep 401 but stop flattening the kind in the existing log line (`axum/extract/mod.rs:64`).
5. **Register correction.** Mark the "`decode`/`decode_with_default_jwk_set` panics (unwrap on empty JWK set)" clause as not-reproducible/stale when this item is closed — no `decode_with_default_jwk_set` ever existed here (verified via working-tree grep and `git log --all -S "jwk"`, 0/540 commits), and `jwt/mod.rs:587-588` pins Err-on-empty-keys.
6. **Pinned-test handling (flip these in the same commit):**
   - `jwt/mod.rs:452-472` `tokens_with_an_audience_claim_are_unverifiable_by_decode` — delete the `BUG(pinned)` comment (`:465-468`) and rename the test; after fix 1 alone, `Jwt::decode` (no aud policy) must return `Ok` for aud-bearing tokens; after fix 2, add/rewrite so `decode_with_audience(&t, &pub, Some("oxidauth"))` is `Ok`, `Some("other-realm")` is `Err(InvalidAudience)`, and aud-less legacy tokens are `Ok` under both.
   - `jwt/mod.rs:582-588` inside `decode_with_public_keys_tries_each_key` — the two sentinel-string asserts (`"JwtError: no valid public key found"`) change to the split variants (e.g. NoPublicKeysConfigured vs NoMatchingPublicKey); update both asserts (no `BUG(pinned)` marker there; it pins current behavior that this fix intentionally changes).
   - Intentionally UNCHANGED: `works_with_rsa` (`:419-450`, aud-less tokens verify under all variants), `decode_rejects_expired_token` (`:510-525`) and the leeway pin (`:543-547`) — the fix must not touch `exp`/leeway/`alg` settings; service-test decodes of minted tokens (`authenticate.rs:807,1040`, `register.rs:601`, `exchange_refresh_token.rs:758,799`, `totp/validate.rs:551`, `authenticate_or_register.rs:1013,1064`) keep passing via the `decode` compat shim + aud-tolerant validation; `oxidauth-rs` client/extractor tests mint aud-less tokens through the builder and stay green.

## Verification

- New kernel regression tests (permanent, replacing the pin above): aud-token round-trip matrix — `aud` token × {no policy → Ok, expected match → Ok, expected mismatch → `InvalidAudience`, legacy aud-less → Ok}; `decode_with_public_keys` empty-vs-foreign key kinds. `cargo test -p oxidauth-kernel` (whole module: `cargo test -p oxidauth-kernel jwt::`).
- Verifier plumbing: `cargo test -p oxidauth-api middleware::permission_extractor` (assert empty-keys → 503 / bad-token → 401 branches) and `cargo test -p oxidauth-rs` (extractor + client decode paths; native subset per the crate's existing wasm-boundary tests).
- Mint-path regressions run with existing service suites (mock repos, no DB): `cargo test -p oxidauth-services auth::` plus `refresh_tokens::exchange_refresh_token` and `totp::validate` — these already decode every minted token, so they prove aud-stamped tokens verify with the updated verifier.
- End-to-end: `./hurl.sh` (from `src/oxidauth/`, needs Postgres per `docker-compose.yml`) — authenticate/register/exchange flows assert `$.payload.jwt` and reuse it on protected routes (`hurl/tests/authenticate.hurl`, `exchange.hurl`), proving a live signed token with `aud` passes the live verifier.
- Manual spot check: decode a minted token's payload (`jwt.split('.')[1]`, base64url) in a throwaway script and assert the claim set now contains `"aud":"oxidauth"`, and that a payload captured pre-fix (no `aud`) still decodes via `decode_with_public_keys`.
