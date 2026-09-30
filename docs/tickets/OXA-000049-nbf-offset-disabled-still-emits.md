# OXA-000049 — `NbfOffset::Disabled` only skips the custom offset; `JwtBuilder::build` always emits `nbf` (now−10s default)

**Original ID:** SRV-13 · **Severity:** P3 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; defect verified intact at HEAD — `Disabled` skips only `with_not_before_from`, builder's unconditional `Some(now-10)` re-stamps it; all 4 mint paths same shape; pin asserts `nbf is emitted regardless`; `validate_nbf:false` default in jsonwebtoken 9.3.1 means claim is emitted-but-unenforced, so wire stable today. Option A recipe sound (builder `omit_nbf` + `.without_not_before()`, 4 match-sites, key vanishes via existing skip_serializing_if); hazard is future-consumer class (external validators) — revisit before any nbf-validating integration ships)


## Locations

Register cites `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:777` — **marker drift**: line 777 is the first line of the `BUG(pinned)` comment (777–779) inside the test `jwt_claims_carry_subject_issuer_authority_ttl_and_tree_entitlements` (`:748-781`), not the defect. The register's *content* is accurate; the defect lives in the kernel builder. Real sites:

- `oxidauth-kernel/src/jwt/mod.rs:132` — the defect. In `JwtBuilder::build` (`:117-155`): `let nbf = Some(nbf.unwrap_or(Ok(now - 10))?);` — an unset offset falls back to `now − 10s`, and the result is wrapped in `Some(…)` unconditionally. `Jwt.nbf` is **structurally always `Some` after `build()`**; the builder exposes no API at all to omit it. (`Jwt` is `#[derive(Default)]`-constructible by hand — all fields `pub`, `:28-44` — but no code path through `Jwt::builder()` can produce `nbf: None`.)
- `oxidauth-kernel/src/jwt/mod.rs:104-114` — `JwtBuilder` state: `nbf: Option<Result<usize, JwtError>>` — two states (explicit / unset), no "explicitly absent" state. `with_not_before_from` (`:175-181`) is the only setter; it computes `epoch_from_now(Sub, duration)` eagerly at call time (`:176`), not inside `build`.
- `oxidauth-kernel/src/authorities/mod.rs:40-44` — `enum NbfOffset { Enabled(Duration), Disabled }` (wire: `{"enabled":{"secs":…}}` / `"disabled"`). There is no `Custom` variant — the register's "custom offset" is the `Enabled(Duration)` arm. `DEFAULT_JWT_NBF = 10s` (`:46`), `Default` = `Enabled(DEFAULT_JWT_NBF)` (`:48-52`).
- The four production mint paths, each the same `if let` with no else arm: `oxidauth-services/src/auth/authenticate.rs:145-150`, `oxidauth-services/src/auth/register.rs:125-130`, `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:165-170`, `oxidauth-services/src/totp/validate.rs:150-155`. `Disabled` ⇒ skip `with_not_before_from` ⇒ `build()` default applies anyway.
- No other production `Jwt::builder()` callers exist — every other site (api `permission_extractor.rs:155,315`, `oxidauth-rs` client/axum test helpers) is test/mock code that builds claims directly and never sets an offset.

Pinned markers and assertions:

- `exchange_refresh_token.rs:777-780` — this register item's marker; assertion `:780` `assert_eq!(claims.nbf.expect("nbf is emitted regardless"), iat - 10);` under a `Disabled` fixture (`nbf_enabled=false` ⇒ `NbfOffset::Disabled`, `:539-543`).
- `oxidauth-kernel/src/jwt/mod.rs:591-609` `builder_applies_nbf_iat_and_ttl_defaults` — `:604` pins `jwt.nbf == Some(iat - 10)` for a bare builder (no `BUG(pinned)` marker there — it pins the kernel default as designed).
- `oxidauth-kernel/src/jwt/mod.rs:611-635` `builder_honors_explicit_claims` — tolerant range for explicit offsets (`:634`).
- Services side, `Enabled` arm (unaffected): `authenticate.rs:812-814` (`iat - 30`; fixture deliberately uses 30s at `:421-424`), `exchange_refresh_token.rs:783-809` (`iat - NBF_OFFSET_SECS`, 30s, `:249-251`).
- `BUG(pinned)` markers elsewhere in the kernel jwt module (`:465-468` audience rejection, `:495-498` Gz asymmetry) are separate issues — OXA-000011 and OXA-000040; out of scope here.

## Problem

`NbfOffset::Disabled` on an authority does not disable the `nbf` claim. It only suppresses the `with_not_before_from(value)` call, and `JwtBuilder::build` then fills in its own hardcoded `now − 10s` default and force-wraps `Some(…)`. Consequences, all code-observable today:

1. Every token minted by any of the four paths carries `nbf`, regardless of the authority setting. `Disabled` and `Enabled(10s)` produce byte-equivalent `nbf` values; the setting is behaviorally a no-op except when `Enabled(d)` with `d ≠ 10s` is configured.
2. The builder cannot express "no `nbf`" at all — `build()` line 132 ignores even a hypothetical "set to None" API, since the `unwrap_or` default *and* the outer `Some(…)` both re-materialize the claim.
3. The pinned test at `:780` documents this as intended-pinned behavior ("nbf is emitted regardless").

## Analysis

**What each setting value actually changes today** (mint paths verified at the line ranges above):

| Authority setting | Mint path action | Wire `nbf` |
|---|---|---|
| `Enabled(d)` | calls `with_not_before_from(d)` | `now(call-site) − d` |
| `Disabled` | skips the call | `now(build-site) − 10` — the builder default |
| unset (`Default` = `Enabled(10s)`, used by bootstrap `oxidauth-services/src/bootstrap/mod.rs:409,782`) | calls `with_not_before_from(10s)` | `≈ now − 10`, identical to `Disabled` |

**Nobody in the repo consumes the claim.** `Jwt::decode` (`oxidauth-kernel/src/jwt/mod.rs:59-66`) validates with `Validation::new(Algorithm::RS256)` (`:63`); in jsonwebtoken 9.3.1 (pinned in `oxidauth-kernel/Cargo.toml:20`) `Validation::new` defaults `validate_nbf: false` (vendored `validation.rs:124`) and `leeway: 60` (`:120`) — the nbf branch (`:280-283`) only fires when `validate_nbf` is on *and* the claim is present. A repo-wide grep for `.nbf` finds reads only in the kernel builder internals and tests; `oxidauth-rs`'s `check_auth_state` keys off `exp` alone (`oxidauth-rs/src/client/mod.rs:934-935`). So today's emitted `nbf` is decorative for oxidauth's own verifiers — which is exactly why honoring `Disabled` carries near-zero validation risk (see resolution).

**External verifiers:** RFC 7519 §4.1.5 makes `nbf` optional; a validator only rejects tokens whose `nbf` is in the future. The emitted `now − 10` is always in the past, so honest verifiers pass today and would equally pass tokens without the claim.

**Fixture flood shows the expectation gap.** `NbfOffset::Disabled` is the routine "don't care" value across the suite — `oxidauth-postgres/src/test_fixtures.rs:35`, `insert_authority/mod.rs:60`, `select_all_authorities/mod.rs:75`, `select_authority_by_id/mod.rs:53`, `update_authority/mod.rs:59`, `oxidauth-rs/src/client/authorities/create_authority.rs:92`, `update_authority.rs:113`, plus services fixtures (`register.rs:308`, `totp/validate.rs:408`, `authenticate_or_register.rs:523`, the exchange fixture above). An operator reading `Disabled` in these settings is being told the claim is absent when it is present.

**Pin fragility (adjacent observation).** The exchange test comment at `:801-802` ("both iat and nbf derive from the same clock reading inside build()") is wrong for the `Enabled` arm: `with_not_before_from` samples the clock at its call site (`jwt/mod.rs:176`), while `iat` samples inside `build` (`:130,134`); the exact-equality pins at `:804-808` and `authenticate.rs:814` hold only when both samples land in the same wall-clock second — a latent per-run race (the kernel's own explicit-claims test uses a tolerant range at `:634` for exactly this reason). [mechanism code-verified; flakiness inferred] Only the exact-default pin `:604`/`:780` (same `now`, `:132`) is arithmetically exact. Any rewritten assertions should use a tolerant range.

**Cosmetic:** each mint path ends its `if let` with a stray `};` (e.g. `exchange_refresh_token.rs:170`); a `match` rewrite in the fix removes these naturally.

## Impact

- **Affected:** operators who set `jwt_nbf_offset: "disabled"` on an authority (a first-class persisted setting — the postgres repository round-trips it verbatim, e.g. `select_all_authorities/mod.rs:139-141`) and expect tokens without `nbf`. Tokens they mint still carry it; every JWT the server ever issues leaks a mint-time hint via `nbf` (weakly — `iat` already does).
- **Not affected:** token acceptance. oxidauth's own decoder never checks nbf (`validate_nbf: false`), and spec-conformant third-party verifiers accept past `nbf` unconditionally. No auth success/availability impact — hence P3.
- **Risk type:** contract/expectation drift pinned as intended behavior. The latent bite is the inverse operation: the knob cannot do the one thing its name promises, and no API exists to mint an `nbf`-free token even for future features (e.g. interop with verifiers that treat `nbf` presence as a policy signal, per RFC 7519's "MUST NOT be required … process only if understood").

## Proposed resolution

**Option A (recommended): honor `Disabled` by omitting `nbf`.** The name promises suppression and nothing in the system depends on presence; the rename alternative institutionalizes the lie.

1. Kernel builder (`oxidauth-kernel/src/jwt/mod.rs`): add `omit_nbf: bool` to `JwtBuilder` and a `without_not_before()` setter; in `build`, `:132` becomes `let nbf = if omit_nbf { None } else { Some(nbf.unwrap_or(Ok(now - 10))?) };`. Document that `without_not_before` wins over an earlier `with_not_before_from` (no caller uses both). Keep the `now − 10` default for unset offsets so the bare builder (used by every test helper in api/oxidauth-rs) is unchanged. The `Jwt` struct needs no change — `nbf` is already `Option<usize>` with `skip_serializing_if = "Option::is_none"` (`:29-36`), so `None` drops the key from the claim JSON, and `Jwt::decode` deserializes an absent `nbf` to `None` via serde.
2. The four mint paths become `match authority.settings.jwt_nbf_offset { Enabled(v) => .with_not_before_from(v), Disabled => .without_not_before() }` (`authenticate.rs:145-150`, `register.rs:125-130`, `exchange_refresh_token.rs:165-170`, `totp/validate.rs:150-155`).
3. Compat: **wire** claim JSON changes only for `Disabled` authorities — `nbf` disappears; tokens minted before/after coexist under the same verifiers (jsonwebtoken skips absent nbf at `validation.rs:280`; `validate_nbf` is off anyway). Authority settings JSON and DB rows are untouched — the enum and its `"disabled"` serde tag stay as-is. The realistic breakage requires a downstream that *requires* `nbf` presence on a deliberately-disabled authority, which contradicts the setting's own contract.
4. Optional hardening while touching the area: re-express the exact-equality `Enabled` pins as tolerant windows (fragility above).

**Option B (rejected): rename `Disabled` → `Default` / document as "use default offset."** Honest but strictly worse: same fixture churn (`match` sites), plus a public-enum rename touching kernel + `oxidauth-rs` re-export (`client/authorities/mod.rs:8`) + every fixture listed above, needing `#[serde(rename = "disabled")]` to keep DB/API wire stable — and the knob still does nothing. Choose behavior, not nomenclature.

**Pin flips (Option A):**

- `exchange_refresh_token.rs:777-780` — the flip: delete the marker comment; `:780` becomes `assert!(claims.nbf.is_none(), "…")` plus a wire assertion that the decoded claim JSON carries no `"nbf"` key. This test is the fail-before/pass-after regression: today the claim is `Some(iat - 10)`.
- Kernel `builder_applies_nbf_iat_and_ttl_defaults` `:604` — **stays green unchanged** (bare builder keeps the 10s default); add a new kernel test for `without_not_before()` → `jwt.nbf.is_none()` and `serde_json::to_value(&jwt)` lacking `"nbf"`, and encode/decode round-trip of an nbf-free token.
- `authenticate.rs:812-814` and `exchange_refresh_token.rs:783-809` (`Enabled` arm) — unchanged, proving the custom-offset path is untouched.
- `register.rs` and `totp/validate.rs` tests — fixtures already `Disabled` but contain no `nbf` assertions; stay green. If permanent coverage of all four paths is wanted, add one `is_none()` assertion each to the register and validate success tests rather than new fixtures.

## Verification

- Failing-before/passing-after: the flipped `jwt_claims_carry_subject_issuer_authority_ttl_and_tree_entitlements` — pre-fix it observes `nbf == Some(iat - 10)` on a `Disabled` fixture; post-fix `is_none()` with no `"nbf"` key in `serde_json::to_value(&claims)`.
- `cargo test -p oxidauth-kernel jwt` — new `without_not_before` tests; `builder_applies_nbf_iat_and_ttl_defaults`, `builder_honors_explicit_claims`, `jsonwebtoken_default_leeway_accepts_recently_expired` (`:527-548`, exp-leeway contract — unchanged) all stay green.
- `cargo test -p oxidauth-services refresh_tokens::exchange_refresh_token` (flipped Disabled test + `Enabled(30s)` counterpart), `cargo test -p oxidauth-services auth::` (authenticate keeps its `iat - 30` pin; register unaffected), `cargo test -p oxidauth-services totp::` (Disabled fixture, no nbf pins).
- `cargo test -p oxidauth-api permission_extractor` and `cargo test -p oxidauth-rs` — test-helper mints use the bare builder (default unchanged), must stay green; this is the no-wider-blast-radius proof.
- Wire spot check (throwaway): decode a post-fix `Disabled`-authority token and dump `serde_json::to_value(&claims)`; confirm key set is the pre-fix set minus `nbf`, and `Jwt::decode` still accepts the token.
