# OXA-000058 — jsonwebtoken's default 60s leeway is load-bearing for four boundary tests

**Original ID:** N-8 · **Severity:** n/a · **Type:** note · **Status:** scheduled (via OXA-000011)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #16 of 27 · reviewed 2026-09-29 · FOLDED INTO OXA-000011 — no standalone patch


## Locations

Register line: `BUGS_AND_NOTES.md:90` (§5 N-8). Verified against code; register is accurate but imprecise about the flip threshold (see Analysis).

- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:59-66` — `Jwt::decode`. Line 63 builds `Validation::new(Algorithm::RS256)` and is the **only** `Validation` construction in the workspace (verified: `Validation::new|leeway` matches nothing outside kernel `jwt/mod.rs`); `jsonwebtoken` is a kernel-only dependency (`oxidauth-kernel/Cargo.toml:20`, `Cargo.lock` pins 9.3.1). Leeway is unset anywhere → library default applies. **Not configurable anywhere** — no config field, no builder parameter.
- `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs:68-81` — `Jwt::decode_with_public_keys` loops `Jwt::decode` per key, so it inherits the same leeway. Every verification path funnels through line 63: `oxidauth-api/src/middleware/permission_extractor.rs:43`, `oxidauth-rs/src/axum/extract/mod.rs:63`, client `authenticate` at `oxidauth-rs/src/client/mod.rs:270`, client `refresh` at `oxidauth-rs/src/client/mod.rs:377`, and the services test decodes.
- Upstream default: `~/.cargo/registry/.../jsonwebtoken-9.3.1/src/validation.rs:120` — `leeway: 60` in `Validation::default` (which `Validation::new` uses); applied to `exp` at `:275` and to `nbf` at `:280`.
- Kernel boundary pins (`oxidauth-kernel/src/jwt/mod.rs`, `mint`-relative offsets via `epoch_from_now`, `:362-376`, `SystemTime`-based):
  - `:509-525` `decode_rejects_expired_token` — `exp = now − 300s`, asserts `JwtError: ExpiredSignature` ("well beyond jsonwebtoken's 60s default leeway", `:513`).
  - `:527-548` `jsonwebtoken_default_leeway_accepts_recently_expired` — `exp = now − 30s`, asserts decode succeeds; comment `:531-535` states the test exists SOLELY to depend on the 60s default and MUST fail loudly if the library default changes or oxidauth sets `leeway(0)`.
- Client boundary pins (`oxidauth-rs/src/client/mod.rs`, `mint` at `:726-727` uses `Utc::now() + exp_offset`):
  - `:929-976` `expired_jwt_triggers_refresh_via_get_jwt` — mints `exp = now − 30`; relies on decode accepting the token (leeway) while `check_auth_state` (`:426-440`, exact `now > jwt.exp`, no leeway) classifies it `Refresh`.
  - `:1308-1354` `concurrent_get_jwt_after_expiry_refreshes_exactly_once` — same `−30` mint (`:1314`), same dual dependency.
  - All other client/axum-extract test tokens use `+3600`/`+7200` — leeway-insensitive.

## Problem

N-8 recorded: the crate verifies JWTs with jsonwebtoken's *implicit* 60s leeway, and four tests (two kernel, two client) place claims at ±30s around `exp` — i.e., 30s inside the leeway window — so their pass/fail is coupled to a value the repo never states. The register's "a >60s clock jump flips those tests" is directionally right but imprecise: the `−30s` pins flip on a **>30s** clock step (see below). There is no defect in production behavior today — this is a hidden cross-crate contract: kernel decode semantics ↔ client test offsets ↔ upstream default constant, none of it written down in repo code.

## Analysis

**Register verified true**, with refinements:

1. **Clock-jump sensitivity (only mid-test steps matter — mint and verify run milliseconds apart):**
   - Kernel `:527-548` (exp = now−30): fails on a forward step of >30s (decode then sees `elapsed > 60s`… precisely, rejection at `now₁ > exp + 60 = now₀ + 30`).
   - Kernel `:509-525` (exp = now−300): robust — only a forward step of ≥241s would wrongly accept.
   - Client `:929-976` / `:1308-1354` (mint −30): fail on a **forward** step >30s (decode rejects → `"failed to validate jwt"` instead of the refresh path) *or* a **backward** step >30s (`check_auth_state` sees `now ≤ exp` → `Valid` → zero refresh HTTP).
   - `nbf` also gets leeway (`validation.rs:280`); builder default `nbf = now − 10` (`jwt/mod.rs:132`), so a backward step of ~70s+ mid-test would surface `ImmatureSignature` instead. (Related nbf note: OXA-000049.)
2. **Determinism today:** under NTP slew (sub-second corrections) all four tests are deterministic; only clock *steps* (VM suspend/resume, manual `date`, aggressive NTP step on CI runners) threaten them — rare, matching the register's "accepted".
3. **The kernel pin is an intentional tripwire,** per its own comment (`:531-535`): it exists to detect an upstream default change. That is a real virtue — it converts a silent production tolerance shift into a loud CI failure — but it also means an upstream default change produces confusing *client*-side failures first (`:929-976` dies in `authenticate`, far from the cause).
4. **Does acting require flipping test pins?** Setting an explicit `validation.leeway = 60` keeps **all four pins passing unchanged** — it pins the number the tests already assume. Only *comments* change: kernel `:513`, `:531-535` (the test's premise shifts from "library default" to "oxidauth-stated contract"), client `:725`, `:934-935`. Zero test-result flips.
5. **Why not `leeway(0)`?** That would be a behavior change: the client refreshes only when `now > exp` and servers rely on the 60s window to tolerate client/server clock skew; `leeway(0)` would 401 borderline tokens. Explicit-60 preserves behavior exactly.
6. **KEEP-vs-pin assessment:** KEEP is defensible (deterministic under normal clocks; tripwire test documents the contract). Explicit-60 is better: jsonwebtoken's default is a convenience value, not a stability guarantee (upstream has changed defaults across majors; current 9.x keeping 60 is not contract), and the kernel↔client −30 coupling stays implicit under KEEP. The counterweight is churn: line 63 is exactly what OXA-000011 must rewrite.

## Impact

No user-visible defect today; a latent-contract note:

- Upstream default change silently alters server-side TTL tolerance on every verify path (api middleware, axum extract, client auth/refresh) while the guard test lives in a different crate from where failures first appear.
- CI flake surface: four tests fail (with misleading errors — foreign-key-style "failed to validate jwt" on the client) if a clock steps ≥31s during their run window.
- Any future "tighten the verifier" change touching `Validation` must remember both sides; today only test comments carry that knowledge.

## Proposed resolution

**FIX LATER — fold into OXA-000011, do not stand up a separate patch.** Same construction site (`jwt/mod.rs:63`): OXA-000011 already rewrites that `Validation::new` (expected-audience config), so both changes are `let mut validation = Validation::new(Algorithm::RS256);` followed by named settings. Concrete plan, sequenced **with or immediately after OXA-000011 lands**:

1. Add `validation.leeway = 60;` at `jwt/mod.rs:63`'s rewritten block (a `pub const JWT_LEEWAY_SECS: u64 = 60` is optional; value alone suffices — the client tests' −30 offset needs no compile-time link since the client never reads kernel internals for it).
2. Update kernel pin comments: `:513` ("our explicit 60s leeway"), `:531-535` (premise becomes "oxidauth pins leeway to 60 — the test now guards against *us* changing it, which must stay loud").
3. Update client comments `oxidauth-rs/src/client/mod.rs:725`, `:934-935` to cite "kernel's explicit 60s leeway" instead of "jsonwebtoken default".
4. No test flips, no behavior change; N-8 is then retired from `BUGS_AND_NOTES.md` §5.
5. If OXA-000011 is deferred past a jsonwebtoken major bump, promote this one-liner to standalone first — the tripwire's failure message (`:545-546`) will say so if the default moves.

## Verification

Static verification only (no builds/tests run, per assignment):

- `BUGS_AND_NOTES.md:90` read; register text confirmed against code.
- Grep `Validation::new|leeway` across `src/` → single construction at `oxidauth-kernel/src/jwt/mod.rs:63`; no configuration surface anywhere. Grep `jsonwebtoken` in crate manifests → kernel-only dependency; `Cargo.lock` pins 9.3.1.
- Read `jsonwebtoken-9.3.1/src/validation.rs` in the local registry: `leeway: 60` default (`:120`), exp check `exp < now − leeway` (`:275`), nbf check `nbf > now + leeway` (`:280`).
- Read all four boundary tests and their offsets: kernel `:509-525` (−300), `:527-548` (−30); client `:929-976` (−30), `:1308-1354` (−30 at `:1314`); confirmed every other client/extract token uses +3600/+7200.
- Confirmed `check_auth_state` (`oxidauth-rs/src/client/mod.rs:426-440`) compares `now > jwt.exp` with no leeway — the dual dependency the −30 tests exploit; confirmed `mint` (`:726-727`) is wall-clock relative.
- Confirmed every verify path routes through `Jwt::decode`/`decode_with_public_keys` (api `permission_extractor.rs:43`, axum extract `:63`, client `:270`/`:377`).
- Clock-jump flip thresholds (forward >30s, client backward >30s, backward ≥~70s nbf) are arithmetic from the cited comparisons — not executed; labeled inference by construction from the read code.

## Decision (2026-09-29) — FOLDED INTO OXA-000011; no standalone patch
- Reviewed jointly (scout re-verification + owner ruling 2026-09-29). All claims CONFIRMED: sole `Validation::new` at `oxidauth-kernel/src/jwt/mod.rs:63`, leeway unset/unconfigurable anywhere, jsonwebtoken 9.3.1 kernel-only; the four boundary tests verified at their offsets — kernel `:509-525` (−300s, robust) + `:527-548` (−30s, intentional tripwire), client `:929-976` + `:1308-1354` (−30s, exploiting the decode-vs-`check_auth_state` asymmetry at `:426-440`, bare `now > exp`); every other token +3600/+7200.
- Disposition: the one-line `validation.leeway = 60;` **rides OXA-000011's rewrite of the same `:63` block** — additive, zero test-result flips (all four boundary tests pass before and after; it pins the value they already assume), comment-only rewording kernel `:513`/`:531-535`/`:545-546` + client `:725`/`:934-935` to cite "oxidauth's stated 60s" instead of "the library default" — the tripwire then guards against *us* changing it, which must stay loud.
- Coordination note for OXA-000011's implementer: 000011's Analysis says the fix "MUST leave leeway untouched" — no contradiction: the invariant is *effective* leeway = 60; stating it explicitly is how both tickets land at one site. Double-scheduling risk assessed LOW (separate line, no merge contention).
- Promotion trigger: if a jsonwebtoken dependency bump lands *before* OXA-000011 and the tripwire (`:545-546` failure message) fires, promote the explicit assignment to a standalone patch immediately — do not wait for 000011.
- Rejected and recorded: `leeway(0)` (real behavior change — 401s borderline tokens under client/server clock skew, breaks the refresh window clients rely on); any config surface; landing a standalone one-liner now (double-touches 000011's block).
- Orthogonality confirmed (scout): OXA-000049 touches `JwtBuilder::build()` `:132` (nbf claim *presence*); this touches decode validation — zero shared code. OXA-000011's own `aud` pin (`:465-468`) and OXA-000028's stale-`raw_jwt` pin (`client/mod.rs:956`) are unaffected by the leeway line.
- Register: N-8 (`BUGS_AND_NOTES.md:90`) is retired in 000011's landing commit, not here. Acceptance inherits 000011's, plus: the four boundary tests green before AND after the explicit-60 commit.
