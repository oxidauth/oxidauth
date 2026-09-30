# OXA-000029 — refresh() base64-decodes raw-PEM keys, so client re-auth can never validate its jwt

**Original ID:** CLI-3 · **Severity:** P1 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved; kernel decode_with_flexible_public_keys raw-PEM-first + b64 fallback, refresh() single-call, CLI-3 pin flipped + full_keyset raw-only, b64-fallback + foreign-key-rejection fixtures added — red-before 77/5 green-after 82/82; CLI-2 attribution resolved via red-run (raw_jwt write predates this ticket; OXA-000028 landed in prior WIP); live-server double-renewal smoke deferred to integration pass)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved; scope = kernel `decode_with_flexible_public_keys` (raw PEM first, b64 fallback) + replace refresh()'s hand-rolled b64 loop w/ single call; flip CLI-3 pin as-written (raw-PEM-only keyset passes), simplify `full_keyset` to raw PEM only; DO NOT touch CLI-2/CLI-4 pins or the auth()-anti-pin; independent, ~2 files; fixes dead auto-re-auth + refresh-token burn at token expiry)


## Locations

All paths relative to `src/oxidauth/` unless noted.

- **Defect:** `oxidauth-rs/src/client/mod.rs:369-382` — inside `Client::refresh()` (`:329-423`), the per-key loop calls `BASE64_STANDARD.decode(public_key)` (`:372`) on each key served by `GET /public_keys` and silently `continue`s on decode error (`:374`). If no key produces a valid decode, `jwt` stays `None` and the failure arm at `:411-416` returns `ClientErrorKind::Other("failed to validate jwt")`.
- **Register drift (confirmed, matches the established pattern):** the register cites `client/mod.rs:1290`. That line is the `BUG(pinned)` comment inside the test `refresh_is_broken_against_the_real_raw_pem_wire_format` (`:1256-1301`, marker at `:1290-1296`). The product defect is the decode loop at `:369-382`. The register's *description* of the mechanism is accurate — only the pointer is a test, not the code.
- **The correct path, for contrast:** `auth()` (`:227-326`) validates via `Jwt::decode_with_public_keys(&payload.jwt, &public_keys)` (`:269-272`), which feeds each served string verbatim into `DecodingKey::from_rsa_pem` (`oxidauth-kernel/src/jwt/mod.rs:68-81` → `:59-66`). This is the healthy production path, explicitly pinned by `auth_rejects_base64_storage_format_the_api_never_serves` (`oxidauth-rs/src/client/mod.rs:1219-1254`) with the anti-pin comment at `:1245-1249` ("the format bug belongs to refresh() only").
- **Wire format, settled (this was the open question):** the server stores base64(PEM) — `oxidauth-services/src/public_keys/create_public_key.rs:36-43` generates rows only via `KeyPair::new()?.base64_encode()` (same finding as OXA-000008/OXA-000017) — **but `GET /api/v1/public_keys` serves raw PEM**: `ListAllPublicKeysUseCase` base64-decodes every row back to UTF-8 PEM before returning (`oxidauth-services/src/public_keys/list_all_public_keys.rs:37-48`), and the handler (`oxidauth-api/src/server/api/v1/public_keys/list_all_public_keys.rs:31`) wraps the use-case output straight into `ListAllPublicKeysRes { public_keys }` (`oxidauth-http/src/public_keys/list_all_public_keys.rs`) — a plain serde struct over `PublicKey { public_key: String, .. }` (`oxidauth-kernel/src/public_keys/mod.rs:11-16`) with no custom serializer. There is no re-encoding anywhere on the read path: **the client's `public_key` field is raw armored PEM.** Register claim confirmed.
- **Dead re-auth loop:** `get_jwt`/`get_jwt_decoded`/`request` → `authenticate_if_needed` (`:443-449`) → `check_auth_state` (`:426-440`): `now > jwt.exp` ⇒ `AuthState::Refresh` ⇒ `self.refresh().await` (`:447`) ⇒ guaranteed `Err("failed to validate jwt")` against a live server.
- **Token-burn amplifier (server side):** `ExchangeRefreshTokenUseCase` (`oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs`) does single-use rotation — inserts the new refresh token (`:190-196`) and **deletes the presented one** (`:198-200`) *before* the client ever validates the jwt. On the `jwt = None` arm, `refresh()` returns early at `:412-415` without storing `payload.refresh_token` (`state.refresh_token` is only written on the success arm, `:387`), so the rotated token is discarded while the client's stored copy has already been deleted server-side.
- **Pins/fixtures in the test module:** `BUG(pinned)` at `:1290-1296` (this item); adjacent pins `:966` (CLI-2: `refresh()` never updates `state.raw_jwt`) and `:1074` (CLI-4: failure envelope collapses to `Other("")`) — both separate register items, neither flips here. The `full_keyset` helper (`:708-723`) mounts **both** encodings specifically so happy-path state-machine tests can drive `refresh()` past verification (comment `:711-717`) — scaffolding that exists only because of this bug.

## Problem

`refresh()` assumes the keys fetched from `GET /public_keys` are base64-encoded; they are not. Production rows reach the wire as raw PEM because `ListAllPublicKeysUseCase` decodes the storage format before serving it (Locations). PEM armor (`-----BEGIN PUBLIC KEY-----`) contains `-` and newlines, which are outside the `BASE64_STANDARD` alphabet, so `BASE64_STANDARD.decode` fails on **every** served key, unconditionally — not probabilistically, not for a subset. Each key hits `continue` (`:374`), the loop ends with `jwt = None`, and `refresh()` returns `Other("failed to validate jwt")` (`:411-416`) even though the refresh POST itself succeeded and returned a perfectly good jwt.

Consequences against a live server, in sequence:

1. Login works: `auth()` verifies fine (raw-PEM consumer).
2. At the first expiry, any `get_jwt`/`request` call enters `AuthState::Refresh` → `refresh()` POSTs `/refresh_tokens`, the server rotates (new token inserted, old deleted), the client cannot validate the fresh jwt, errors out, and **throws away the rotated refresh token**.
3. Every subsequent `refresh()` now fails even earlier: the stored refresh token no longer exists server-side, `FindRefreshTokenById` errors, the error envelope falls into the wildcard arm `:419`, and the caller sees `Other("")` (the CLI-4 wart — the server's `RowNotFound`-style message is dropped). The client is permanently unable to re-auth until the application calls `authenticate(user, password)` again — which many long-lived service integrations structurally never do.

The test suite hid this by mounting both encodings (`full_keyset`, `:708-723`): the raw-PEM entry feeds `auth()`, the base64 entry feeds `refresh()`. `refresh_is_broken_against_the_real_raw_pem_wire_format` (`:1256-1301`) then pins the broken behavior with a live-format wiremock fixture.

## Analysis

**Why the mismatch exists (inference, but strongly supported):** the same table is served in *two* different formats depending on endpoint. `POST /public_keys` (create) and `GET /public_keys/{id}` (find) return the storage format, base64(PEM) — the find use case deliberately does not decode (`oxidauth-services/src/public_keys/find_public_key_by_id.rs:27-34`; its test pins the raw b64 string `LS0tLS1…` with the comment "rows store base64(PEM); unlike `list_all_public_keys`, find does not decode", `:54-57`), and the hurl suite asserts find's response byte-equals create's captured `public_key` (`hurl/tests/public_keys.hurl`, by-id GET section). `refresh()`'s pre-base64-decode matches the create/by-id format, not the list format it actually consumes — it reads like it was written against the wrong endpoint's contract.

**Who is affected:** SDK (`oxidauth-rs`) consumers only. The server never takes this path — its `ExtractJwt` middleware verifies bearer tokens with `decode_with_public_keys` on the same raw PEM the use case returns (`oxidauth-api/src/middleware/permission_extractor.rs:36-44`, per OXA-000012). Anyone using the raw wrapper for `POST /refresh_tokens` without jwt validation is also unaffected; the bug is exactly in the managed, expiry-driven path (`get_jwt`, `get_jwt_decoded`, every `request()` call) plus the public `refresh()` method itself. Any integration expected to outlive its `jwt_ttl` is dead on schedule.

**Interaction with OXA-000012 (skip + warn):** OXA-000012 proposes per-row `skip+warn` inside `ListAllPublicKeysUseCase` and explicitly commits to keeping the wire format raw PEM ("The `oxidauth-rs` SDK needs no change (its `refresh()` double-decode mismatch is tracked separately)"; success responses byte-identical). So:

- OXA-000012 **cannot rescue this bug** — the server already serves raw PEM and its resolution preserves that. Conversely, this ticket's client-side fix needs nothing from OXA-000012. The two are orthogonal and compatible in either landing order.
- A **server-side canonicalization** in the other direction (make `GET /public_keys` serve base64 so `refresh()`'s decode succeeds) is dead on arrival: it would (a) sink `auth()`, the only currently-working verify path — the anti-pin at `:1245-1249` exists precisely to prevent that flip; (b) break the server's own `ExtractJwt` middleware, which consumes the same use case through `decode_with_public_keys` → `from_rsa_pem`; (c) contradict OXA-000012's compat commitment. Ruled out; the fix must be client-side.
- After OXA-000012 lands, rows that aren't valid base64(PEM) are *skipped server-side*, so a tolerant client sees an even cleaner raw-PEM set. One residual case becomes client-visible only thanks to OXA-000012: a row that is valid base64 of valid UTF-8 **but not PEM** (e.g. b64-of-b64 from a double-encoded seed) survives the server filter and arrives as base64 text on the wire — the raw-PEM-only client fix fails it in `from_rsa_pem`; the tolerant fix below tries the base64 leg and verifies-or-skips it identically. Either way no bad key is ever trusted (signature verification still gates everything).

**Security note on accepting both encodings:** key material arrives over the same transport regardless of encoding; the JWKS trust model already assumes the client can authenticate the channel to its configured `base_url`. Accepting base64(PEM) in addition to PEM adds no verification capability an attacker could exploit — the jwt is still only accepted if it verifies against the supplied key bytes under RS256.

**Adjacent pins this fix must NOT flip:** `:966` (CLI-2 — `refresh()` still won't write `state.raw_jwt`; even after this fix `get_jwt()` keeps serving the stale token string until CLI-2 lands) and `:1074` (CLI-4 — the wildcard error arm at `:419` stays `Other("")`). Fixing CLI-3 shrinks CLI-4's blast radius (it stops masking the token-burn failure), but touches neither assertion.

## Impact

- **P1 confirmed.** Every `oxidauth-rs` client that depends on automatic session renewal loses all authenticated traffic at the first jwt expiry — deterministic, not a race. Clients that only ever call `authenticate()` with fresh credentials per process (short-lived jobs, tests) never notice, which is how this shipped.
- **Credential/rotational damage beyond the outage:** the first failed renewal *consumes* the refresh token server-side and discards the replacement, so recovery is not automatic even for apps that retry; they must fall back to username/password re-auth, and the burn is invisible (`Other("failed to validate jwt")` looks like a bad server key, not a lost token).
- **Diagnostic cost:** symptoms are split across time and error strings — "failed to validate jwt" at expiry, then empty `Other("")` errors forever (CLI-4) — pointing away from the real one-line-format bug.
- **No server-side or cross-client impact:** token verification by the server, hurl contracts, and non-SDK clients are untouched.

## Proposed resolution

**Client-side, format-robust verification in `refresh()` — try raw PEM first, then base64-decode.** (The server-side alternative was ruled out in Analysis; it would sink `auth()`, the `ExtractJwt` middleware, and OXA-000012's compat contract.)

1. **Add a tolerant helper next to the existing one**, `oxidauth-kernel/src/jwt/mod.rs` (adjacent to `decode_with_public_keys` `:68-81`; note OXA-000011 also proposes changes in this file — coordinate landing order, no functional conflict):

```rust
/// Verify against each key trying raw PEM first (the format
/// `GET /public_keys` serves), then base64-decoded PEM (the storage /
/// create / find-by-id format), so the SDK works against any endpoint
/// encoding or version skew without ever trusting an unverifiable key.
pub fn decode_with_flexible_public_keys(
    token: &str,
    keys: &[PublicKey],
) -> Result<Jwt, JwtError> {
    for key in keys {
        if let Ok(jwt) = Jwt::decode(token, key.public_key.as_ref()) {
            return Ok(jwt);
        }

        if let Ok(decoded) = BASE64_STANDARD.decode(key.public_key.as_bytes())
            && let Ok(jwt) = Jwt::decode(token, &decoded) {
            return Ok(jwt);
        }
    }

    Err(JwtError {
        message: "no valid public key found".to_string(),
    })
}
```

   Raw-first ordering is mandatory: it is the live wire format and keeps `auth()`'s behavior (were it ever pointed at this helper) identical for every well-formed keyset — the `:1245-1249` anti-pin stays satisfied because raw PEM remains the primary leg. The b64 leg is version-skew insurance (SDK deployed against an older/newer server, or double-encoded rows that survive OXA-000012's filter), not a dependency.

2. **Replace `refresh()`'s hand-rolled loop** (`oxidauth-rs/src/client/mod.rs:369-382`) with a single call — `let jwt = Jwt::decode_with_flexible_public_keys(&payload.jwt, &public_keys).map_err(|_| ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None))?;` — then proceed on the existing success arm (`:385-410`). Delete the `Option<Jwt>` staging and the `None` arm (`:411-416`). This mirrors `auth()`'s shape, removes the duplicated per-key loop, and fixes the silent-skip style (a wrong-format key no longer vanishes without a trace — the helper is the sole verify funnel and its error is surfaced). `get_public_keys` (`:182-224`), the wildcard error arm `:419`, and `state.raw_jwt` (`:966` pin / CLI-2) are deliberately left alone.

3. **`auth()` stays as-is.** Swapping it to the flexible helper would be harmless (raw-first) but unnecessary; keeping the diff to the one broken verify path honors the pinned healthy path. Revisit only if a third consumer appears.

**Pin flips (all in `oxidauth-rs/src/client/mod.rs` tests):**
- `refresh_is_broken_against_the_real_raw_pem_wire_format` (`:1256-1301`) → rename `refresh_validates_the_real_raw_pem_wire_format`; delete the `BUG(pinned)` comment (`:1290-1296`); flip the assertion from `matches!(err.kind, Other("failed to validate jwt"))` to `assert!(client.refresh().await.unwrap())` plus `assert_eq!(client.get_jwt_decoded().await.unwrap().exp, fresh_exp)`. Do **not** assert `get_jwt() == new jwt` — that is CLI-2's pinned stale behavior and must stay failing-till-CLI-2.
- `full_keyset` (`:708-723`): keep the helper but rewrite its comment (`:708-717`) — mount a single raw-PEM entry (drop the base64 entry) so every state-machine test exercises the real wire format. The b64-only fixture is promoted to the dedicated fallback test below instead of being smeared across all happy paths.
- `auth_rejects_base64_storage_format_the_api_never_serves` (`:1219-1254`): unchanged — it mocks the wire directly and pins `auth()`, which this fix doesn't touch. Confirm it still passes.
- Pins `:966` (CLI-2) and `:1074` (CLI-4): unchanged.

**No API/wire changes:** server, DTO, hurl suite, and OXA-000012's work all proceed independently. This is an SDK-internal fix; `refresh()`'s public signature (`pub async fn refresh(&self) -> Result<bool, ClientError>`) and `ClientErrorKind` are untouched, so no public SDK API break.

## Verification

- `cargo test -p oxidauth-rs client::tests` — flipped `refresh_validates_the_real_raw_pem_wire_format` passes (raw-PEM-only keyset, `refresh()` returns `Ok(true)`, `get_jwt_decoded` shows the new `exp`); every pre-existing `full_keyset`-driven state-machine test (rotation persistence `:979`, concurrency `:1309`, expiry `:933`) stays green; CLI-2/CLI-4 pins (`:966`, `:1074`) still pin.
- **New wiremock fixture, both formats:** (a) raw-PEM-only keyset → `refresh()` succeeds (primary leg); (b) base64(PEM)-only keyset (storage/by-id format) → `refresh()` succeeds (fallback leg) — mirror the fixture shapes in `:1225-1229` and `:1265`; (c) foreign-key keyset in either encoding → still `Err("failed to validate jwt")` (the fallback must not become rubber-stamping); (d) rotation-chain test: mount two sequential `POST /refresh_tokens` responses and assert the second exchange presents the token rotated by the first (regression for the token-burn loop being *reachable* now).
- `cargo test -p oxidauth-kernel jwt` — direct unit tests for `decode_with_flexible_public_keys`: raw PEM verifies, b64 PEM verifies, garbage-in-both-formats key is skipped, empty key list yields `"no valid public key found"`, and a key whose b64 decodes to non-PEM yields the pass-through error (no panic).
- **Live-server smoke (the proof the mocks couldn't give):** start the server per the repo's hurl setup; via the SDK, `authenticate()`, then force expiry (short `jwt_ttl` authority or wait past `exp`), then `get_jwt()` → must return `Ok` with the *decoded* state carrying a fresh `exp`, `POST /refresh_tokens` hit exactly once server-side, and the old refresh-token row gone while the rotated one is persisted client-side (`get_jwt()` again after a second expiry must renew *again without credentials* — two clean renewals prove the rotation chain, i.e. exactly the loop that was dead before).
- `hurl --test src/oxidauth/hurl/tests/public_keys.hurl` — unchanged server contracts (create/list/by-id formats, including by-id's b64 equality assertion) still pass, confirming the fix introduced zero server-side pressure.
- Confirm marker removal: `grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-rs/src/client/mod.rs` returns only `:966`-CLI-2 and `:1074`-CLI-4-era markers (renumbered), never the CLI-3 one.
