# OXA-000040 — Entitlements::Gz carries different payload types on the mint side vs the decoded side

**Original ID:** SRV-4 · **Severity:** P2 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved, zero findings; Option A live — Gz holds base64 wire payload both sides, as_vec inflates on demand/None-on-corrupt, 2 pins flipped to identity contract, re-sign round-trip regression red-before Invalid-symbol-58 → green, Serialize/Deserialize provably untouched, token bytes byte-identical; size caveat doc landed)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved Option A: `Gz(String)` always holds wire payload (base64-of-gzip) both sides — decode keeps gunzip-as-validation, `as_vec()` decompresses on demand, Serialize/Deserialize unchanged, wire byte-identical, no migration; flip 2 pins to round-trip-identity (stronger), add decode→re-encode→decode regression (fails today); end-to-end extractor tests must stay green; merge-order note only vs deferred 000011, same file)


## Locations

Register cites `oxidauth-kernel/src/jwt/mod.rs:495` — **marker drift**: line 495 is the `BUG(pinned)` comment (495-498) inside the test `gz_entitlements_survive_jwt_round_trip` (`:475-507`), not the defect. Real sites, all in `oxidauth-kernel/src/jwt/mod.rs`:

- `:221-225` — `enum Entitlements { Txt(String), Gz(String) }` with `#[derive(PartialEq, Eq)]`; the single `String` payload of `Gz` has **two incompatible meanings** depending on provenance.
- `:231-257` — `Entitlements::encode`: the `Gz` arm (`:239-253`) gzip-compresses (`flate2::GzEncoder`, `Compression::best()`, import `:10`) and base64-wraps (`BASE64_STANDARD`) the joined plaintext, then stores **base64(gzip(plaintext))** in `Gz(..)`.
- `:259-288` — `Entitlements::decode`: the `gz` arm (`:268-281`) base64-decodes, gunzips via `GzDecoder`, and stores the **decompressed plaintext** in `Gz(..)`.
- `:290-302` — `as_vec`: splits the payload on `' '` verbatim for both variants — correct only when the payload is plaintext, i.e. only after `decode`. Returns `Option` but never `None`; no signal for a wrong-side `Gz`.
- `:313-325` — hand-written `Serialize`: unconditional `format!("{} {}", GZ_PREFIX, s)` — assumes the payload is wire-ready (base64-of-gzip), true only on the encode side.
- `:327-355` + `:305-311` — `Deserialize`/`FromStr` → `decode`, producing the plaintext side.
- `:203-211` — `JwtBuilder::with_entitlements` stores the **encode side**; `:117-157` `build()` transposes encode errors; `:28-44` the `Jwt` claims struct exposes `pub entitlements: Option<Entitlements>`; `:51-58` `Jwt::encode(&self)` and `:60..` `Jwt::decode` run these same `Serialize`/`Deserialize` impls on **the same public struct** — decode-then-modify-then-re-encode is a one-liner away.

Producers (mint, all pass the per-authority `EntitlementsEncoding`, kernel `authorities/mod.rs:36`, with a fresh plaintext `Vec<String>` from `PermissionSearch` each time — no variant reuse): `oxidauth-services/src/auth/authenticate.rs:167-174,232-239`, `oxidauth-services/src/auth/register.rs:118-123`, `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:154-162`, `oxidauth-services/src/totp/validate.rs:140-146`.

Consumers (decode side only): `oxidauth-api/src/middleware/permission_extractor.rs:67-73` and `oxidauth-rs/src/axum/extract/mod.rs:90-96` — both `jwt.entitlements.and_then(|e| e.as_vec()).unwrap_or_default()` on claims returned by `Jwt::decode`.

Tests pinning current behavior:
- `oxidauth-kernel/src/jwt/mod.rs:475-507` `gz_entitlements_survive_jwt_round_trip` — marker `:495-498`; assertion `:499-505` pins decoded `Gz("oxidauth:**:** realm:resource:action")` (plaintext inside `Gz`); `:485-489` pins mint-side `Gz` as base64 (`starts_with("H4sI")`).
- `oxidauth-kernel/src/jwt/mod.rs:682-698` `gz_entitlements_wire_format_round_trips_via_from_str` — `:696` pins `parsed == Gz("a:b:c d:e:f")` (plaintext side).
- `oxidauth-api/src/middleware/permission_extractor.rs:308-330` `extract_entitlements_yields_gz_permissions` — unpinned end-to-end contract test (Gz token through `ExtractJwt` → `ExtractEntitlements`); helper `signed_token` `:154-172` mints either encoding.
- Txt-only (symmetric, unaffected): `oxidauth-rs/src/axum/extract/mod.rs:323-341` (constructs `Entitlements::Txt` directly), `oxidauth-services/src/auth/register.rs:604-612`, `authenticate.rs:816-823,1041-1047`, `exchange_refresh_token.rs:768-776`, `totp/validate.rs:555-562` — all `as_vec()` on decoded claims.

## Problem

`Entitlements::Gz(s)` means "base64(gzip(plaintext))" in `s` when built by `encode()` (the JWT-minting side) but "plaintext" when built by `decode()`/`Deserialize`/`FromStr` (the JWT-verifying side). No field, marker, or type distinguishes the two, so every operation on the variant is correct for exactly one provenance:

- `as_vec()` (`:290`) is only correct on the decoded side; on the mint side it returns one garbage entry (`["H4sI…"]`).
- `Serialize` (`:313`) is only correct on the mint side; serializing a decoded-side `Gz` writes `gz <plaintext>` — the `gz` tag over uncompressed, un-base64'd bytes.
- `PartialEq`/`Eq` compares raw strings, so mint-side `Gz(b64)` ≠ decoded-side `Gz(plaintext)` despite being semantically the same entitlements — the assertion at `:499-505` has to spell out the plaintext to match.

`Txt` is fully symmetric (`Txt` holds plaintext on both sides); the asymmetry is exclusive to `Gz`.

## Analysis

Round-trip matrix (verified against the code paths above):

| Start | Operation | Result |
|---|---|---|
| mint-side `Gz(b64)` | `Serialize` → wire → `Deserialize` | decoded-side `Gz(plaintext)` — **works**; pinned by `:475-507` |
| decoded-side `Gz(plaintext)` | `as_vec()` | correct — pinned by `:506` |
| mint-side `Gz(b64)` | `as_vec()` | `["H4sI…"]` — silent garbage, no error |
| decoded-side `Gz(plaintext)` | `Serialize` | `gz <plaintext>` — corrupted wire form |
| corrupted form | `Jwt::decode` / `FromStr` | **hard error**: `BASE64_STANDARD.decode` rejects `:` (permissions always contain `:` — the `realm:resource:action` grammar), and ` ` separators are also non-base64 → every re-serialized decoded claim becomes a token that carries a **valid RS256 signature but fails deserialization of its own claims** — at oxidauth's own extractors |
| decoded-side `Gz(plaintext)` | `Deserialize`→`Serialize` idempotence | `Deserialize ∘ Serialize` is a one-way mint→decode projection, never identity |

**Why it doesn't bite today.** The end-to-end pipeline is mint (`with_entitlements` from fresh plaintext) → `Serialize` → wire → `Deserialize` → `as_vec` — each step uses the variant in the one orientation where it is correct, and every operation between mint and consume is a one-way flow: no code in the repo re-serializes a decoded `Jwt` (all eight non-base64 `.encode(&..)` sites operate on builder-constructed claims — production `register.rs:135`, `authenticate.rs:266`, `exchange_refresh_token.rs:175`, `totp/validate.rs:180`; test helpers `permission_extractor.rs:171,322`, `oxidauth-rs/src/axum/extract/mod.rs:208,292`), no code calls `as_vec` on a builder-side claim, and nothing copies a decoded `entitlements` value into a new `Jwt`. So `Gz` round-trips correctly **by convention only** — a convention the type system cannot enforce and two pinned tests actively document as expected.

**Gz is reachable in production, not dead code.** `AuthoritySettings.entitlements_encoding` (`oxidauth-kernel/src/authorities/mod.rs:36`) is per-authority, serialized as `"txt"`/`"gz"` in the settings JSON (e.g. `oxidauth-rs/src/client/users/contract.rs:323`), and persisted — the postgres repository round-trips `Gz` settings through insert/update/select (`oxidauth-postgres/src/authorities/insert_authority/mod.rs:84,104`, `update_authority/mod.rs:91,108`, `select_all_authorities/mod.rs:102,130`). Any operator can flip an authority to Gz through the create/update-authority API; all four mint paths then compress. The default is Txt (`oxidauth-services/src/bootstrap/mod.rs:412`, and every test fixture uses Txt), which is why the hot suites are all green. The 2FA-pending token (`authenticate.rs:167-174` → `TOTP_VALIDATE_PERMISSION`) carries a Gz claim too on a Gz authority, and `oxidauth-api` TOTP endpoints read it via the extractor path.

**The `as_vec` trap compounds in the fail-closed direction.** Both extractors do `.and_then(as_vec).unwrap_or_default()`; a wrong-side `Gz` yields `Some(["H4sI…"])`, whose entry matches no permission grammar token, so requests are denied with no error anywhere — silent 403 lockout, not escalation. `unwrap_or_default` also means a hypothetical `None` (missing claim) is indistinguishable from wrong-side corruption.

**Double-compression specifically does not occur today**, because `encode` only ever receives `&[String]` plaintext from `PermissionSearch`. The realistic corruption vector is the adjacent one: the first future feature that re-issues or re-serializes a decoded `Jwt` (token introspection, re-signing after key rotation, refresh copying `entitlements` from the access token instead of re-querying the permission tree — all obvious uses of `decode` + `encode` sitting on the same struct, `:51-60`) mints tokens every consumer rejects.

**Size caveat on the feature itself:** gzip-then-base64 expands by ~4/3 before compression is netted; for typical short permission lists `Gz` claims are *larger* than `Txt`. `Gz` only pays off for very large entitlement sets — worth stating in the setting's docs when the fix lands, since the wire format itself is fine and stays unchanged.

Adjacent, unrelated `BUG(pinned)` markers in the same module — audience rejection at `:465-468` (covered by OXA-000011) — are out of scope here.

## Impact

- **Today:** none observable on the supported mint→verify flow (three independent tests cover it end-to-end: `jwt/mod.rs:475-507`, `jwt/mod.rs:682-698`, `permission_extractor.rs:308-330`).
- **Exposed population:** operators who set `entitlements_encoding = gz` on an authority, and resource servers using `oxidauth-rs` `ExtractEntitlements` against such tokens — the code works but sits on an unenforced convention.
- **Latent breakage:** any re-serialization of decoded claims produces tokens with valid signatures and undecodable claims (complete client lockout per token, with a base64/gzip error buried behind a trusted signature); any builder-side `as_vec` produces silent permission denial. Neither is caught by the compiler, `Option`, or `Eq`. `Eq` across provenances (`Gz(b64) ≠ Gz(plaintext)`) also misleads anyone caching or comparing claims.
- P2: no current security or availability impact; a correctness landmine documented-as-intended by pinned tests, on the auth-critical path.

## Proposed resolution

Make the in-memory representation symmetric by keeping the **wire bytes as the payload** — the wire format (`"txt …"` / `"gz <base64-of-gzip>"`) is not at fault and MUST stay byte-identical; no migration or version gating is needed since tokens issued before/after are interchangeable.

**Option A (recommended):** `Gz(String)` always holds the wire payload (base64-of-gzip).
1. `decode()` (`:268-281`) stores the base64 payload verbatim instead of decompressing — but keep the `BASE64_STANDARD.decode` + `GzDecoder::read_to_string` as *validation* (discard the decompressed string) so `entitlements_decode_rejects_malformed_input` (`:701-712`) keeps its exact error surface at parse time.
2. `as_vec()` decompresses on demand for `Gz` (same code, moved): base64 → gunzip → split `' '`; return `None` on failure, finally giving the `Option` real meaning. Txt branch unchanged.
3. `Serialize`/`Deserialize` need no change — `serialize` becomes provably correct on every value constructible through the API, `Deserialize ∘ Serialize` becomes identity, and mint-side ≡ decoded-side under the derived `Eq`.
4. `as_vec` on a mint-side claim (which no consumer has today) now returns the real permissions instead of garbage.

**Option B (heavier):** split types — mint-side `Entitlements` vs a `DecodedEntitlements`/view with `as_vec` — maximal type safety, but churns the public surface (`oxidauth-rs` constructs `Entitlements::Txt` directly in tests `:326`; `Jwt.entitlements` is `pub`) for the same runtime guarantee Option A already achieves. Prefer A.

**Pinned-test flips (Option A):**
- `jwt/mod.rs:499-505` — replace the plaintext-inside-`Gz` equality with `assert_eq!(decoded.entitlements, claims.entitlements)` (round-trip becomes identity — strictly stronger than the current pin); delete the marker comment `:495-498`. `:485-489` (mint-side base64) and `:506` (`as_vec` → original vec) stay.
- `jwt/mod.rs:696` — `assert_eq!(parsed, gz)` (the encode-side value) instead of `assert_eq!(parsed, Entitlements::Gz("a:b:c d:e:f".to_string()))`; `:697` (`as_vec` output) unchanged.
- Keep `permission_extractor.rs:308-330` and all Txt assertions untouched — they are unpinned end-to-end contract tests that MUST stay green (they are the regression proof).
- No wire-format or DB changes; `oxidauth-rs` mock-feature paths (`axum/extract/mod.rs:30-37`) construct whole `Jwt`s in code and are unaffected.

## Verification

- **Failing-before / passing-after regression:** new kernel test — build a `Gz` token, `Jwt::decode`, `redecoded.encode(&private)` **without rebuilding entitlements**, `Jwt::decode` again, `as_vec() == original vec`, and `serde_json::to_value(&first_decode.entitlements)` equals the original claim string. This fails today at the second decode (`gz oxidauth:**:** …` → base64 error on `:`) and passes after the fix.
- `cargo test -p oxidauth-kernel jwt` — `gz_entitlements_survive_jwt_round_trip` (flipped), `gz_entitlements_wire_format_round_trips_via_from_str` (flipped), `txt_entitlements_wire_format_is_prefix_plus_joined_list` (`:659-680`, unchanged), `entitlements_decode_rejects_malformed_input` (`:701-712`, unchanged — proves validation still rejects `gz !!!not-base64!!!`).
- `cargo test -p oxidauth-api permission_extractor` — Gz end-to-end through the real extractor stack (mint → sign → `ExtractJwt` → `ExtractEntitlements`).
- `cargo test -p oxidauth-rs` — Txt extraction tests unchanged.
- Service suites cover the Txt mint paths: `cargo test -p oxidauth-services auth::`, `refresh_tokens`, `totp` — no expected changes.
- Wire-identity spot check (throwaway): `serde_json::to_string` of mint-side and fixed decoded-side claims is identical to the pre-fix claim string, proving zero wire drift.
