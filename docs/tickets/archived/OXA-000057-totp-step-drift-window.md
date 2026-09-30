# OXA-000057 — TOTP validation window: register says ±1 step, code is zero tolerance

**Original ID:** N-7 · **Severity:** n/a · **Type:** note · **Status:** done
**Review tier:** Tier 1 (docs-only), ranked item 5/9 (pair with OXA-000055) · IMPLEMENTED — N-7 register line 2026-09-29; update_password.rs:121 mirror landed in 2026-09-30 review pass (was missing at close)


## Locations
- `src/oxidauth/oxidauth-services/src/totp/validate.rs:112-118` — production validation (`ValidateTOTPUseCase::validate_totp`); builder sets `ascii_key`, `period(totp_ttl)`, `timestamp(now)` and never sets a tolerance
- `src/oxidauth/oxidauth-services/src/totp/validate.rs:94-99` — period taken per-authority from `TotpSettings::Enabled { totp_ttl, .. }`
- `src/oxidauth/oxidauth-services/src/auth/strategies/username_password/update_password.rs:121-127` — second validation site; hardcoded `.period(600)`, no tolerance
- boringauth 0.9.0 vendored source `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/boringauth-0.9.0/src/oath/totp.rs`:
  - `:125-131` — `is_valid` loops `(base_counter - negative_tolerance)..=(base_counter + positive_tolerance)`
  - `:199-212` — `TOTPBuilder::new()` defaults `positive_tolerance: 0`, `negative_tolerance: 0`
  - `:225-243` — `.tolerance()`, `.positive_tolerance()`, `.negative_tolerance()` setters exist ("should not set a value higher than 2", default 0)
- `src/oxidauth/oxidauth-kernel/src/authorities/mod.rs:55-60` — `TotpSettings::Enabled { totp_ttl, webhook, webhook_key }`; no tolerance field
- Pins: `totp/validate.rs:486-497` (`stay_inside_totp_window`, comment "`is_valid` runs with zero tolerance"), `:499-530` (`wrong_code_rejects_before_fetching_signing_material`), `:532-583` (`valid_code_issues_jwt_claims_and_an_inserted_refresh_token`); `update_password.rs:450-457`; `forgot_password.rs:223-231` ("zero-tolerance `is_valid` semantics") and `:235-252` ("the returned code must be the current window TOTP")

## Problem
Register N-7 records: "TOTP validation accepts ±1 step (60s drift window) — pinned in `totp::validate` tests." Verification against current code shows both halves of the claim do not hold:

1. **Acceptance is the current step only (zero tolerance).** Neither validation site calls any boringauth tolerance setter, and boringauth 0.9.0's builder defaults are `0/0` (`oath/totp.rs:203-204`), so `is_valid` checks only the base counter. A repo-wide grep finds no `.tolerance(`/window call anywhere under `src/`.
2. **No 60s window exists.** The period is the authority's `totp_ttl` (`totp/validate.rs:94-99,114`); tests pin 600s (`totp/validate.rs:226`) and `update_password.rs:123` hardcodes 600.
3. **No test pins ±1 acceptance.** The `totp::validate` tests pin the opposite: `stay_inside_totp_window()` exists precisely because zero tolerance makes boundary-straddling codes fail, and its comment states the zero-tolerance semantics explicitly.

`git log -S tolerance -- src/oxidauth/oxidauth-services/src/totp/validate.rs` returns nothing — the file's history never contained a tolerance call, so the note likely predates boringauth 0.9.0 usage or assumed the RFC's recommended window without checking the default.

## Analysis
- **Where ±1 would come from:** RFC 6238 §5.2 recommends receivers allow at least one time-step in either direction — but that guidance exists to absorb skew between an independent client clock (authenticator app) and the verifier. That premise does not apply here: the code is generated **server-side** (`authenticate.rs:186-192` generates and delivers via the authority webhook; `forgot_password.rs:63` generates) and validated **server-side** on the same `epoch()` clock. The failure mode zero tolerance actually exposes is not clock drift but delivery+user latency crossing a bucket boundary — a code delivered near the end of step *N* is rejected when resubmitted in step *N+1* (same UX tension N-5 records for failed exchanges; see OXA-000055).
- **Security math:** with zero tolerance a code's lifetime is the remainder of its step (avg `totp_ttl/2`, worst `totp_ttl`; 600s in tests, per-authority in production). There is **no per-code consumption state** — no table, flag, or burn mechanism exists on this path — so a captured code is replayable until the step flips. A ±1 tolerance would double the worst-case replay ceiling to two steps. Note this also means N-5's "codes are burned by exchange" phrasing deserves the same skeptical re-check in OXA-000055: `validate()` keeps no used-code state.
- **Configurability today:** none. `TotpSettings::Enabled` exposes only `totp_ttl`, `webhook`, `webhook_key`; there is no tolerance knob in the kernel type and no call site sets one. The upstream knobs (`.tolerance(u64)` etc.) exist in boringauth if a tolerance is ever wanted; adding one would be a new `TotpSettings` field plus plumbing in **both** verify sites (`totp/validate.rs:112`, `update_password.rs:121` — the latter also carries a hardcoded 600s period drift separate to other tickets).
- **Test-flip exposure:** enabling any tolerance would flip the pins that document zero tolerance — `stay_inside_totp_window` guards (`totp/validate.rs:486-497`, `update_password.rs:450-457`, `forgot_password.rs:223-231`) become unnecessary, and the "current window TOTP" assertion (`forgot_password.rs:247-252`) plus the old-step rejection in `wrong_code_rejects_before_fetching_signing_material` change contract. Treat as an intentional contract change, not a silent tweak.

## Impact
No live security defect — current behavior is *stricter* than the note's premise and stricter than RFC 6238's recommendation, which is defensible for a server-generated, webhook-delivered code. The real harm is register rot: a future reader trusting N-7 would (a) believe stale-step codes are accepted (they are not) when reasoning about replay ceilings or UX timeouts, or (b) "restore" a ±1 window that never existed, weakening the pins and the property above.

## Proposed resolution
**KEEP** — zero tolerance is intended, test-defended behavior; retire the note with documentation, no code change:

1. Rewrite the N-7 line in `BUGS_AND_NOTES.md` §5 to record the verified state: boringauth 0.9.0 default tolerance 0/0, per-authority `totp_ttl` period, no per-code burn state, no config surface; cross-ref OXA-000055.
2. Add a doc comment above the builder at `totp/validate.rs:112` stating: zero tolerance by design — codes are generated and validated on the server clock (webhook delivery model), so client clock skew is not a factor; a code is accepted only within the current `totp_ttl` step, and nothing consumes it, so replay is possible until the step flips. Mirror it at `update_password.rs:121`.
3. FIX LATER only if delivery-latency complaints appear in the field: add an optional `tolerance: u64` (capped at 1–2 per boringauth docs) to `TotpSettings::Enabled`, plumb into both verify sites, and update the boundary pins listed in Analysis as a deliberate contract change with a changelog entry.

## Verification
- Read the cited ranges before writing: `totp/validate.rs:81-190` (impl) and `:194-669` (tests), `update_password.rs:112-132,438-462`, `forgot_password.rs:210-258`, kernel `authorities/mod.rs:28-75`.
- Read boringauth 0.9.0 source at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/boringauth-0.9.0/src/oath/totp.rs` — confirmed `is_valid` tolerance loop (`:125-131`), zero defaults (`:199-212`), and unused setters (`:225-243`); `Cargo.toml` pins `boringauth = "0.9.0"`.
- `grep -rn "tolerance\|window"` over `src/oxidauth/**/*.rs` — every hit is a test helper/comment asserting zero tolerance; no production setter call exists.
- `git log --oneline -S "tolerance" -- src/oxidauth/oxidauth-services/src/totp/validate.rs` — empty; no tolerance ever existed in this file's history.
- Not verified (marked as such): which boringauth version N-7 was written against (no history evidence in-file); the exact production `totp_ttl` values in deployments (per-authority DB data, not in repo — tests use 600s).

## Decision (2026-09-29) — ACCEPTED as proposed (KEEP + document), scheduled; no implementation started
- **Review tier: Tier 1 (docs-only), item 5/9 (pair with OXA-000055).**
- Zero tolerance confirmed as intended behavior — server-generated/webhook-delivered codes share the verifier clock; RFC 6238's skew rationale does not apply, and ±1 would double the worst-case replay ceiling under 55's no-consumption regime.
- Ownership split with OXA-000055 honored: 57 owns the N-7 rewrite in `BUGS_AND_NOTES.md` §5 and the `update_password.rs:121` mirror comment; 55 owns N-5 + the `totp/validate.rs:112` comment (55's drafted text retires both claims — one comment serves both passes). Combined doc pass.
- Accepted UX cost: correct code submitted in the final seconds of a bucket rejects ("invalid totp code"); treated as theoretical until field complaints (the `stay_inside_totp_window` helper proves the boundary is near, not that users hit it).
- FIX-LATER branch stays gated, NOT scheduled: optional `tolerance: u64` (cap 1–2) on `TotpSettings::Enabled` + plumbing at both verify sites ONLY if delivery-latency complaints appear; recorded as a deliberate contract change — flips the zero-tolerance pins (`stay_inside_totp_window` ×3, current-window assertion, old-step rejection) + changelog. Never a silent tweak.
- Hardcoded `600` period at `update_password.rs:123` noted as separate drift, owned by its own ticket — not touched here.
- Acceptance: doc-comment + register text only; no tests flip; `cargo check -p oxidauth-services`.
