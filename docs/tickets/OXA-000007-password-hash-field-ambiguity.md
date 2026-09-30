# OXA-000007 — Password-hash raw material joins fields with an unescaped `:` separator

**Original ID:** SEC-7 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 5 (hard — cross-crate redesign / format migration / IdP flow) · unreviewed


## Locations

Paths relative to `src/oxidauth/`. Verified against the working tree 2026-09-29.

- `oxidauth-services/src/auth/strategies/username_password/helpers.rs:17-19` — `raw_password_hash(password, password_salt, password_pepper)` returns `format!("{}:{}:{}", …)` (the join itself is line 18). No escaping, no lengths, no version tag.
- `oxidauth-services/src/auth/strategies/username_password/helpers.rs:43-54` — test `raw_password_hash_does_not_escape_colons_in_inputs` with the `BUG(pinned)` comment at `:45-48` (the register's cited line 45 is this marker, not the code), asserting the collision equality as pinned behavior.
- `oxidauth-services/src/auth/strategies/username_password/helpers.rs:9-15` — `verify_password` (argon2 `PasswordVerifier` over the raw string's bytes); `:21-30` — `hash_password` (argon2 with a fresh random `SaltString` per hash).
- Writers of stored hashes (all compose raw material via `raw_password_hash`, then `hash_password`):
  - `oxidauth-services/src/auth/strategies/username_password/registrar.rs:48-51` (login registration), constructed by `auth/register.rs:170` (`registrar::new`).
  - `oxidauth-services/src/auth/strategies/username_password/user_authority_from_request.rs:25-33` (AOR path — builds a peppered hash keyed by username).
  - `oxidauth-services/src/auth/strategies/username_password/update_password.rs:135-141` (password reset/rotation; persists via `UpdateUserAuthority` at `:147-156`).
- Reader/verifier: `oxidauth-services/src/auth/strategies/username_password/authenticator.rs:46-60` — recomputes raw material from the submitted password + **current** authority params + env pepper and verifies against the stored hash; built by `auth/authenticate.rs:282-292` (`build_authenticator`).
- Field provenance:
  - `password` — user-submitted (`Password` newtype, `authenticator.rs:19-23`).
  - `password_salt` — `AuthorityParams.password_salt`, the authority's JSONB params (`username_password/mod.rs:20-23`), **mutable through the API**: `oxidauth-api/src/server/api/v1/authorities/update_authority.rs:25-50` (`PUT /api/v1/authorities/{authority_id}`, permission-gated).
  - `password_pepper` — process env `OXIDAUTH_USERNAME_PASSWORD_PEPPER`, read independently at `authenticator.rs:73`, `registrar.rs:73`, `update_password.rs:137`.
- Storage: `UserAuthorityParams { password_hash }` JSON in `user_authorities.params` (`username_password/mod.rs:45-48`); whole-JSON rewrite path exists at `oxidauth-postgres/src/user_authorities/update_user_authority/` (wired in `oxidauth-api/src/provider/services.rs:313-321`).
- Same-class sibling (second implementation): `oxidauth-services/src/authorities/strategies/username_password/authenticate.rs:22-28` and `register.rs:31-37` — raw material `username:password:pepper:addtl_pepper` (four `:`-joined fields; username and password both user-controlled; format duplicated inline in the test helper `authenticate.rs:79-88`). A grep across `oxidauth-services` and `oxidauth-kernel` found **no username charset validation**, so usernames containing `:` are accepted.
- `oxidauth-services/src/auth/authenticate.rs:41-88` — `AuthenticateUseCase` has **no** `UpdateUserAuthorityQuery` dependency; the kernel `Authenticator` trait (`oxidauth-kernel/src/auth/mod.rs:36-44`) returns `Result<(), BoxedError>`, so today the login path cannot persist a rehash.

**Register drift (verified):** line `helpers.rs:45` points at the `BUG(pinned)` comment inside the test; the actual ambiguous join is `helpers.rs:17-19`. The register's technical claim — `("a:b","c","d")` and `("a","b:c","d")` hash identically — is exactly what the pinned test asserts (`helpers.rs:49-53`), and "fix requires a hash-format migration (invalidates all stored hashes)" holds (see Analysis).

## Problem

The secret fed to argon2 is `password + ":" + salt + ":" + pepper` with no delimiter discipline. Because `:` is legal inside every field, distinct logical credentials can produce **byte-identical raw material** — e.g. `("a:b", "c", "d")` and `("a", "b:c", "d")` both yield `"a:b:c:d"` — so the hash is not bound to the credential decomposition, only to the concatenation. The raw material also carries no version tag, so the format can never be changed or extended in place.

Consequences, in order of concreteness:

1. **Every stored hash is format-locked.** argon2 binds the exact input bytes; the plaintext is unrecoverable; therefore *no offline re-hash is possible* and any change to `raw_password_hash` silently turns every `user_authorities.params.password_hash` row in the database into a non-verifying artifact. The register's "invalidates all stored hashes" is verified.
2. **Credential-equivalence classes exist whenever the joining parameters ever move.** `password_salt` is live authority params editable via `PUT /api/v1/authorities/{id}`, and the pepper is a rotatable env var. If an authority's salt ever changes from, say, `"b:c"` to `"c"`, then a user registered before the change with password `P` and a user registered after it with password `P:b` have stored hashes bound to the *same* raw bytes — whoever knows either decomposition authenticates against both hashes.
3. **The format cannot evolve.** Adding a field (realm, tenant, username) or moving fields between the two existing implementations is unsound until boundaries are unambiguous; the sibling four-field strategy already prepends a fully user-controlled `username` with no `:` validation.

## Analysis

**Mechanism.** `raw_password_hash` is a lossy map from `(password, salt, pepper)` to bytes: the concatenation forgets where each field ends. argon2 then commits to those bytes with its own random per-hash salt (`helpers.rs:22`), so two collisions produce different stored strings but **both verify against the same raw material**. Verification (`authenticator.rs:46-60`) recomputes the raw material from the submitted password plus the *current* authority params, then argon2-verifies — so the ambiguity is re-materialized on every login.

**How exploitable is it today (stated honestly).** With a *fixed* salt and pepper, the map `password ↦ password:S:P` is injective (fixed suffix), so two passwords on one authority cannot collide while params are static — and because the server always rebuilds raw material from the submitted password, the classic field-shift is not a "log in with anything" bypass. The collision bites only when decompositions are held by *different parties across a params change* (scenario 2 above): user B registers after a colon-bearing salt rotation, deliberately choosing `P_A + ":" + removed-segment`; if user A's stored hash happens to bind the same bytes, B's *own, known* password authenticates A's account. That requires (a) an authority salt/pepper rotation whose values contain `:`, and (b) a password choice that falls on the matching side of the boundary — coincidental, but realistic in provisioning flows that hand out templated initial passwords while ops rotates salts. I found **no** currently-reachable collision that does not also require knowing the old/new salt strings or the counterpart password; treat the exploitability as narrow-but-real, not theoretical-only, and note the pin exists precisely because nobody wanted to reason about it at fix time.
- The sibling four-field strategy (`authorities/strategies/username_password/`) widens the blast radius the moment any code path verifies one row's hash against raw material built from another row's fields (e.g. a future "copy credential between authorities" feature): username and password are both user-chosen and unvalidated, so decompositions are attacker-plannable.
- Related-but-separate hazard in the same bytes: the static `password_salt` is folded **into the secret** (argon2 already has its own random per-hash salt). Rotating it — a routine-looking `update_authority` call — invalidates every hash in the same way a pepper rotation does. This is why the fix must treat the raw-material format as versioned and why ops rotation policy matters (see missing-tests.md §B1 note (d), line 444, which pins this same collision).

**Who is affected.** Every deployment with a `username_password` authority: all `user_authorities` rows storing `password_hash` under either implementation are the blast radius of any format change. The fix itself breaks nobody who logs in post-migration; the *migration design* determines whether users who never log in again get stranded.

**Why the naive fix is wrong.** "Try v2, fall back to v1" doubles argon2 work on every *wrong-password* attempt (both verifies fail) — a CPU-amplification DoS on the unauthenticated login endpoint. Routing must be decided by a stored marker, never by trial-and-error.

## Impact

- **Security:** credential-equivalence across salt/pepper rotations (impersonation under the scenario above); a hash that commits to concatenation rather than structure blocks every future change to the raw material, which is itself a security liability (the next "just add realm to the string" change is unsound).
- **Data/operational:** ~100% of stored password hashes are re-hashable only at login time (plaintext never available); any rollout must keep old hashes working or accept mass lockout.
- **UX:** forced global reset (the lazy migration) logs every user out of password auth at once; a login-time rehash is invisible.

## Proposed resolution

**Step 1 — unambiguous v2 raw material (length-prefixed, domain-separated).** In `helpers.rs`, keep one builder and make it injective by construction:

```rust
/// v2 raw material: version tag + `<byte_len>:<field>` per field, no separators.
pub fn raw_password_hash(password: &str, password_salt: &str, password_pepper: &str) -> String {
    let enc = |s: &str| format!("{}:{}", s.as_bytes().len(), s);
    format!("v2|{}{}{}", enc(password), enc(password_salt), enc(password_pepper))
}
```

Lengths use `as_bytes().len()` (argon2 takes bytes; `len_chars`/`len` would be wrong for UTF-8). The `v2|` prefix domain-separates: no v2 byte string can ever equal a v1 one. Apply the same encoding to the sibling `authorities/strategies/username_password/{register,authenticate}.rs` (four fields, username first) in the same PR — same bug, same migration moment, one migration budget.

**Step 2 — store a format marker instead of trial-and-error verify.** In `username_password/mod.rs:45-48`:

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAuthorityParams {
    pub password_hash: String,
    #[serde(default)]           // absent on existing rows == v1
    pub hash_version: u8,       // 1 = `:`-join, 2 = length-prefixed
}
```

Absent field means v1 (existing rows deserialize unchanged). Every writer (`registrar.rs`, `user_authority_from_request.rs`, `update_password.rs`, and the AOR register path) stamps `hash_version: 2` and uses the v2 builder. The authenticator dispatches on `hash_version`: v2 rows verify with v2 raw material only; v1 rows verify with v1 raw material only — exactly one argon2 verify per attempt, no cost amplification.

**Step 3 — verify-then-rehash at login (preferred migration).**
- Kernel: change `Authenticator::authenticate` (`oxidauth-kernel/src/auth/mod.rs:37-44`) to return `Result<AuthenticationOutcome, BoxedError>` with `pub struct AuthenticationOutcome { pub rehash_params: Option<JsonValue>, .. }` (or simply `Option<JsonValue>`). `oauth2::authenticator` and the `SingleUseToken` arm return `None`; they are in-tree and mechanical to update.
- `username_password/authenticator.rs`: on a successful v1 verification, compute `hash_password(raw_v2(...))` and return it as new `UserAuthorityParams { password_hash, hash_version: 2 }` JSON.
- `AuthenticateUseCase` (`auth/authenticate.rs:41-88`) gains a `U2: UpdateUserAuthorityQuery` constructor dependency and, on `rehash_params: Some`, calls the existing `update_user_authority` query (`oxidauth-postgres/src/user_authorities/update_user_authority/` — writes the whole `params` JSONB, which is safe here because this strategy owns the entire object). Provider wiring pattern already exists at `oxidauth-api/src/provider/services.rs:313-321`. Persist failure must **not** fail the login: log, proceed, retry next login.
- No SQL migration can rewrite hashes (plaintext unavailable); an *optional* observability migration backfilling `"hash_version": 0→`-style markers is possible (`UPDATE user_authorities SET params = params || '{"hash_version":1}' WHERE …` joined to `authorities.strategy = 'username_password'`) but adds nothing functionally — skip unless metrics need it. Instead, increment a `tracing` counter on every v1 success to gauge migration progress.

**Step 4 — deadline + kill switch.** Introduce an env/config gate (e.g. `OXIDAUTH_ALLOW_LEGACY_PASSWORD_HASH=1`, shipped default-on, default-off in a later release). On or after the deadline, v1 rows fail verification with a distinct internal error; admins force-reset stragglers through the existing TOTP-gated `update_password` path (note: that path requires the user's TOTP secret — users without one intersect OXA-000005's broken `forgot_password`; do not silently expand scope there). Users who log in before the deadline are migrated invisibly.

**Rejected alternatives.** (a) Forced global reset: one line of config, but logs every user out of password auth and needs a working self-service reset — which OXA-000005 says doesn't exist safely. (b) Fallback "try v2 then v1": rejected above (verify-cost doubling on failed logins). (c) Escaping `:` (`\:`) instead of length prefixes: smaller diff but adds an escaping function to reason about and still needs the whole versioning story; length-prefix is self-describing and cheaper than parser code.

**Compat/migration summary.** No API surface changes; `UserAuthorityParams` deserialization stays backward-compatible via `#[serde(default)]`. Kernel `Authenticator` trait return type is a breaking change for out-of-tree strategy implementers — call it out in the changelog (`changelogs/`). Behavior change for admins: rotating `password_salt`/pepper remains destructive to all hashes — unchanged by this fix, but the migration should ship with a runbook note saying so.

**Pinned-test handling (grep-verified).** The only `BUG(pinned)` marker for this item is `oxidauth-services/src/auth/strategies/username_password/helpers.rs:45-48`; the other markers in these files (`registrar.rs:284`, `update_password.rs:533,642`) belong to different items. Flip list:
1. `helpers.rs:43-54` `raw_password_hash_does_not_escape_colons_in_inputs` — delete the marker and flip the equality to its negation: `assert_ne!(raw_password_hash("a:b","c","d"), raw_password_hash("a","b:c","d"))` plus a positive assert of the v2 encoding.
2. `helpers.rs:36-41` `raw_password_hash_joins_password_salt_pepper_with_colons` — *unmarked* but pins the exact v1 output `"hunter2:s-a-l-t:p3pp3r"`; rewrite for v2 (rename accordingly, e.g. `…_encodes_fields_with_byte_lengths`). Per repo convention this test's wording must not be re-pinned to the old format.
3. Round-trip call-site tests pass `raw_password_hash` output symmetrically into both hash and verify and therefore **survive automatically**: `registrar.rs:221-226,322-327`, `user_authority_from_request.rs:81-86`, `update_password.rs:620-625`, `auth/register.rs:592-597`, fixture `mod.rs:140-143`, `authenticator.rs` tests at `:118,139,274,327,342`. Tests that assert the *legacy* path (new v1-verify tests added in Step 2) get deleted together with the Step 4 deadline flip.
4. Sibling strategy tests hardcode the format inline — `authorities/strategies/username_password/authenticate.rs:79-88` (`/// The documented raw material: username:password:pepper:addtl_pepper` + `stored_hash` helper) and the matching helper in `register.rs` tests — update in the same PR; the semantic assertions (`rejects_a_hash_made_without_the_username_in_the_raw_material` `:130`, pepper/env-pepper rejects `:151,:171`) stay valid and should gain a username-colon collision case (`("a","b:c")` vs `("a:b","c")` must differ).
5. `missing-tests.md:444` (audit note (d)) records the pinned collision as intentional — update that note when the pin flips.

## Verification

- Unit-level hazard proof (before/after): `cargo test -p oxidauth-services --lib auth::strategies::username_password::helpers` — post-fix the collision test must fail against old code and pass against new (classic failing-before/passing-after regression).
- Whole strategy suites: `cargo test -p oxidauth-services --lib auth::strategies::username_password` and `cargo test -p oxidauth-services --lib auth::authenticate_or_register` (AOR write path stamps v2).
- Sibling strategy: `cargo test -p oxidauth-services --lib authorities::strategies::username_password`.
- Migration behavior (new permanent tests): in `auth/authenticate.rs` tests — seed a fixture user_authority with a **v1** hash (built via the old `format!` inline, deliberately not via the helper so the test doesn't auto-follow), assert (a) login succeeds, (b) the mocked `UpdateUserAuthority` service is called exactly once with `hash_version: 2` and a hash that verifies against the v2 raw material, (c) wrong password on a v1 row triggers exactly one argon2 verify path (no v2 attempt — observable via the mock/counter), (d) a v2 row never triggers `update_user_authority`, (e) rehash persistence failure still returns a successful login. Existing mocks exist in `auth/authenticate.rs` tests; add `UpdateUserAuthorityQuery` to the use case's mock set.
- Persistence leg (needs test DB, `src/oxidauth/devops`/`database_test.sh` convention): `cargo test -p oxidauth-postgres --lib user_authorities::update_user_authority`.
- End-to-end rotation repro (manual, dev server): create authority with `password_salt: "b:c"` → register user `victim` / password `secret` → `PUT /api/v1/authorities/{id}` with `password_salt: "c"` → pre-fix: verify the raw-material collision exists (`raw_password_hash("secret","b:c",PEPPER) == raw_password_hash("secret:b","c",PEPPER)`); post-fix: v1 stored hash still logs `victim` in once, and the row in `user_authorities.params` now shows `hash_version: 2` with the v2 raw material rejecting the shifted password (`secret:b` fails).
- Deadline flip: set `OXIDAUTH_ALLOW_LEGACY_PASSWORD_HASH` off in a test env and assert a v1 row is rejected with the distinct internal error and a login-trace log line.
