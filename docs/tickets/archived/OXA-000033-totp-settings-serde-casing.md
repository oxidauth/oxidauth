# OXA-000033 — `TotpSettings` wire tokens are PascalCase ("Disabled") while the whole `AuthoritySettings` family is snake_case

**Original ID:** CLI-7 · **Severity:** P3 · **Type:** note · **Status:** done
**Review tier:** Tier 1 (docs-only) ranked item 9/9 ONLY under Option B · OPTION A chosen → HARD data-format slice · IMPLEMENTED — snake_case + migration + aliases landed (verified 2026-09-30 review pass)


## Locations

Register row: `BUGS_AND_NOTES.md:57` (CLI-7).

**Register line has drifted (the campaign's known pattern) AND mislocates the defect.** `oxidauth-rs/src/client/users/contract.rs:319` is the `BUG(pinned)` comment (`:319–321`, which paraphrases the register text); the pinned payload token is `"totp": "Disabled"` at `:322`. The actual missing attribute is not in the client crate at all — it is in the kernel:

- Defect site: `src/oxidauth/oxidauth-kernel/src/authorities/mod.rs:54–62` — `#[derive(Debug, Serialize, Deserialize)] pub enum TotpSettings { Enabled { totp_ttl: Duration, webhook: Url, webhook_key: String }, Disabled }` with **no** `#[serde(rename_all = …)]`.
- The snake_case family it deviates from (all verified in the same tree):
  - `oxidauth-kernel/src/authorities/mod.rs:39–44` — `NbfOffset`, `#[serde(rename_all = "snake_case")]` at `:40` → wire `"disabled"` / `{"enabled":{"secs":…,"nanos":…}}` (pinned by `nbf_offset_serde_wire_shape_is_pinned`, `:283–321`).
  - `oxidauth-kernel/src/authorities/mod.rs:64–69` — `AuthorityStatus`, `rename_all` at `:65` → `"enabled"` / `"disabled"` (pinned `:241–262`).
  - `oxidauth-kernel/src/authorities/mod.rs:97–103` — `AuthorityStrategy`, `rename_all` at `:98` → `"username_password"` etc. (pinned `:204–225`).
  - `oxidauth-kernel/src/jwt/mod.rs:214–219` — `EntitlementsEncoding` (the fifth `AuthoritySettings` field), `rename_all` at `:215` → `"txt"` / `"gz"` (pinned `:676–678`). The register names "status/strategy/nbf"; `entitlements_encoding` is a fourth conforming member.
- Shared-type chain (why "client vs server" is the wrong frame — see Problem):
  - Server DTOs reuse the kernel type verbatim: `oxidauth-http/src/authorities/create_authority.rs:1,5–11` (`CreateAuthorityReq.authority: CreateAuthority`, `CreateAuthorityRes.authority: Authority` are `oxidauth_kernel::authorities::*` re-imports); handler `oxidauth-api/src/server/api/v1/authorities/create_authority.rs:21` `Json<CreateAuthorityReq>` / `:48` response.
  - SDK re-exports the *same* kernel type: `oxidauth-rs/src/client/authorities/mod.rs:8`, `client/authorities/create_authority.rs:4–5` (`pub use oxidauth_kernel::…TotpSettings…`); `oxidauth-rs/Cargo.toml:18–20` depends on `oxidauth-kernel` (and `oxidauth-services`). One enum definition in the workspace — `grep` finds no second definition.
- Persistence (where the PascalCase token physically lives):
  - Write: `oxidauth-postgres/src/authorities/insert_authority/mod.rs:23` and `update_authority/mod.rs:23` — `.bind(serde_json::to_value(&params.settings)?)` into the `authorities.settings` JSONB column (`oxidauth-postgres/migrations/20221020180349_create_authorities.sql:7`, `DEFAULT '{}'::jsonb`).
  - Read: `oxidauth-postgres/src/authorities/mod.rs:63,71` — `serde_json::from_value(value.settings)` back into `AuthoritySettings`.
- Artifacts that carry the PascalCase token today (the full flip list if the casing is ever changed):
  - Bootstrap default authority: `oxidauth-services/src/bootstrap/mod.rs:407–413` (`totp: TotpSettings::Disabled`, persisted through the serde write path) and its test assertion `:1434` `assert_eq!(authority["settings"]["totp"], json!("Disabled"))`.
  - seedz raw seed string: `src/seedz/src/fixtures.rs:49` — `"totp":"Disabled"` in `AUTHORITY_SETTINGS` (hand-written SQL JSON; **not** produced by serde, so a kernel attribute would NOT fix it automatically).
  - Live e2e: `src/oxidauth/hurl/tests/authorities.hurl:66` (POST create body), `:86` (response jsonpath `== "Disabled"`), `:121` (PUT update body); `src/oxidauth/hurl/tests/user_authorities.hurl:30` (seed-create body).
  - postgres round-trip tests: `oxidauth-postgres/src/authorities/insert_authority/mod.rs:126–130` and `select_all_authorities/mod.rs:83–87` — `serde_json::from_value(json!({"Enabled": {…}}))`.
  - The SDK pin itself: `oxidauth-rs/src/client/users/contract.rs:319–322`, canned `authority()` fed to eight contract tests (`client/authorities/{create_authority.rs:81,find_authority_by_id.rs:84,find_authority_by_strategy.rs:83,list_all_authorities.rs:77,update_authority.rs:99,delete_authority.rs:75}`, `client/users/authorities/{list_user_authorities_by_user_id.rs:89,find_user_authority_by_user_id_and_authority_id.rs:97}`).
  - Repo documentation that blesses the casing: `docs/migration-plan/13-seedz-dev-seeding.md:204–209` — "`totp` — externally-tagged `TotpSettings` with **no** `rename_all`, so `"Disabled"` (capital) is right".

## Problem

`TotpSettings` is the one externally-tagged enum in the `AuthoritySettings` serialization family whose variant tags ride the wire as PascalCase (`"Disabled"`, `{"Enabled":{…}}`) while `NbfOffset`, `AuthorityStatus`, `AuthorityStrategy`, and `EntitlementsEncoding` all emit snake_case (`"disabled"`, `"enabled"`, `"username_password"`, `"txt"`). A hand-authored `settings` blob therefore mixes casings *within one object* — exactly as seedz ships it: `{"totp":"Disabled",…,"jwt_nbf_offset":{"enabled":{…}},…,"entitlements_encoding":"txt"}` (`src/seedz/src/fixtures.rs:49`).

**The register's premise — a client-side fix — is structurally impossible, and the register understates the situation in a way that changes the resolution.** Because `TotpSettings` exists exactly once, in `oxidauth-kernel`, and *both* sides consume that same type (the api via `oxidauth-http` DTOs, the SDK via re-export, the repository via `to_value`/`from_value`), server and client **cannot disagree today**: PascalCase is the correct, only-accepted wire token end to end, and the client is "inconsistent but correct". The register line at `oxidauth-rs/.../contract.rs:319` is a *pin of kernel behavior observed from the client*, not a client defect. Live evidence that the server both accepts and emits PascalCase:
- `authorities.hurl:53–66` POSTs `"totp": "Disabled"` to the running api (2xx, create succeeds) and `:86` asserts the *response* echoes `"Disabled"` — server parse + emit, verified by the passing e2e suite.
- `authorities.hurl:108–121` PUT (update) accepts the same token.
- `bootstrap/mod.rs:1434` asserts the serialized default authority carries `json!("Disabled")`.
- Every `authorities.settings` row written by the server contains PascalCase (`insert_authority/mod.rs:23`, `update_authority/mod.rs:23`), so any existing deployment's DB stores `"totp":"Disabled"` / `{"Enabled":{…}}` — including rows the nbf backfill migration `20250610213132_add_nbf_offset_to_authority_settings.sql:2–8` touched, which wrote *lowercase* `{"enabled": …}` for `jwt_nbf_offset` in the very same JSONB column (precedent both that settings-JSON migrations are the mechanism, and that mixed casing already shipped).

So this is a **wire-format consistency wart in a public API + storage format**, not a malfunction: severity P3 fits, and "fix the client" is off the table; the real decision is *coordinated change vs document-as-is*.

## Analysis

**serde mechanics.** `TotpSettings` is externally tagged, so `#[serde(rename_all = "snake_case")]` on the enum would change only the variant *tags*: `"Disabled"` → `"disabled"`, `{"Enabled":{…}}` → `{"enabled":{…}}`. The `Enabled` struct-variant fields (`totp_ttl`, `webhook`, `webhook_key`) are untouched by an enum-level `rename_all` (serde renames struct-variant fields only with `rename_all_fields`) and are already snake_case — so the flip is tag-only, no field churn. Deserialization is strict-by-default: today `"disabled"` is a hard parse error (`unknown variant`) on every leg that decodes `AuthoritySettings` — api requests, api responses (SDK decode), DB rows.

**Where the token is a "public API" surface.** Three durable surfaces plus tests: (1) REST bodies of `POST /api/v1/authorities` and `PUT /api/v1/authorities/{id}` (`oxidauth-http` DTOs are the request/response shape); (2) the **at-rest** format of `authorities.settings` JSONB — the token survives binary upgrades, so any rename is a storage migration, not just a code change; (3) non-Rust consumers (curl, terraform, JS SDKs, hurl) hand-authoring settings JSON. Rust SDK consumers using the typed `TotpSettings::Disabled` are immune to casing in Rust source but not to casing on the wire. The TOTP *runtime* surfaces are unaffected: `authenticate.rs:152–224` only destructures the enum's fields (webhook delivery posts `WebhookReq`, a separate DTO), and `POST /totp/validate` uses `totp_secrets`, not `TotpSettings`.

**The transition hazard (why the coordinated option is expensive).** With `rename_all` added and no aliases, the new token and old token are mutually un-decodable. Three states must line up: deployed binary (api + in-process services), stored JSONB, and out-of-band SDK consumers pinned to old crate versions. Notably:
- `update_authority` re-serializes the **whole settings blob** (`update_authority/mod.rs:23`), so after a binary-only cutover, any authority that gets updated persists `"totp":"disabled"` while un-updated rows keep `"totp":"Disabled"` — the read path would then reject the *stale* rows, i.e. a binary upgrade silently corrupts reads of un-migrated data. Hence the migration and the binary cannot ship independently in either order unless decode accepts both.
- The `Enabled` variant is a JSON *object*, so a backfill can't just `jsonb_set` a scalar token like the nbf migration did — rewriting `{"Enabled":{…}}` to `{"enabled":{…}}` means renaming a jsonb key (strip + `jsonb_set` of the inner object under the new key), and both token forms must be handled in the `WHERE` clause.

**The cheap compat hatch.** serde variant aliases — `#[serde(alias = "Disabled")]` / `#[serde(alias = "Enabled")]` alongside `rename_all` — make decode dual-accept while serialize emits only the new snake_case tag. That single kernel change (4 lines) neutralizes every ordering hazard: old SDKs keep POSTing PascalCase against new servers, old DB rows keep decoding after the migration, and rollback stays possible. Aliases are invisible to typed Rust users and can be kept permanently (like `NbfOffset` never needs them — only because it has always been lowercase).

**Why nobody has felt it.** Server/SDK share the enum, the repo's own e2e and fixtures copy the PascalCase token verbatim, and `docs/migration-plan/13-seedz-dev-seeding.md:205–206` explicitly documents the casing as "right". The register itself sourced it from a *test payload comment* in the SDK — classic "surprise when hand-writing the JSON", not a breakage report.

**Marker/pin survey.** The only `BUG(pinned)` marker for this behavior is `contract.rs:319–321`. Related pins *without* markers that lock the token and would flip under Option A (documented in Resolution): `bootstrap/mod.rs:1434`, `insert_authority/mod.rs:126–130`, `select_all_authorities/mod.rs:83–87`, `authorities.hurl:66/86/121`, `user_authorities.hurl:30`, `seedz/fixtures.rs:49`. The kernel's `nbf_offset_serde_wire_shape_is_pinned` (`authorities/mod.rs:283`) pins only `NbfOffset` and is unaffected either way. `OXA-000027` (CLI-1) `:92` instructs its fixer NOT to touch `contract.rs:319` — correct for that slice; this ticket is where the marker's fate is actually decided.

## Impact

- **Consumers today: none broken.** Client and server agree by construction (one shared kernel type), so the defect produces zero runtime failures for SDK users, the server, hurl, or seedz — the entire impact is *surprise and inconsistency* for anyone hand-authoring `settings` JSON (the casing looks like a typo, and a snake_case guess is a hard 422/`unknown variant` error), plus repo-level confusion that generated this register item from a test comment.
- **Server/deployment:** the PascalCase token is baked into every existing `authorities.settings` row and into the public REST bodies of the authorities create/update endpoints. That is the whole reason a "quick rename" is not cheap — it is now a data-format decision, not an attribute addition.
- **Trajectory:** the mismatch is self-replicating. New settings fields or sibling enums will copy whichever neighbor the author reads first (seedz already ships objects with both casings); the register, one test pin, one e2e assertion set, one seed string, and one migration-plan note now all certify the PascalCase token. Deciding once (change-with-aliases, or bless-and-document) stops the accretion; P3 stands — nothing is broken *now*, the cost is delayed consistency.

## Proposed resolution

**Reject "add `rename_all` client-side only"** — there is no client-side enum to decorate. The SDK serializes `oxidauth_kernel::authorities::TotpSettings` verbatim (`client/authorities/mod.rs:8`, `create_authority.rs:4–5`); mirroring a private duplicate enum in `oxidauth-rs` would fork the type that `CreateAuthority.settings`/`Authority.settings` are typed with on both sides and break every DTO in the chain. Any casing change is necessarily a kernel (⇒ server + SDK + storage) coordinated change.

**Option A — coordinated rename to snake_case (pick only if the wire break is worth it):**
1. Kernel: add `#[serde(rename_all = "snake_case")]` **plus** `#[serde(alias = "Disabled")]` on `Disabled` and `#[serde(alias = "Enabled")]` on `Enabled` (`oxidauth-kernel/src/authorities/mod.rs:54`). Aliases make rollout/rollback order-free: old tokens still decode, new token is the only emitted form.
2. New SQL migration (name beside `20250610213132_add_nbf_offset_to_authority_settings.sql`): backfill `authorities.settings` — scalar case `jsonb_set(settings,'{totp}','"disabled"') WHERE settings->>'totp' IN ('Disabled')`; object case for `'Enabled'`: delete the `Enabled` key and re-`jsonb_set` its value under `enabled`. Run with the alias-capable binary (either order safe, but migrate before any binary without aliases).
3. Flip every pin/artifact listed in Locations, per the campaign rule (flip assertion + delete marker):
   - `contract.rs:322` `"totp": "Disabled"` → `"disabled"` and **delete marker `:319–321`** — the sole `BUG(pinned)` marker here. The eight SDK contract tests consuming `authority()` stay green automatically: the canned JSON is the mock *response* and the assertions decode through the same (renamed) enum, so the string edit is self-consistent.
   - `bootstrap/mod.rs:1434` `json!("Disabled")` → `json!("disabled")`.
   - `insert_authority/mod.rs:127` and `select_all_authorities/mod.rs:84`: `"Enabled"` key → `"enabled"`.
   - `authorities.hurl:66/121` bodies + `:86` response assertion; `user_authorities.hurl:30`.
   - `seedz/fixtures.rs:49` `"totp":"Disabled"` → `"totp":"disabled"` — this one is raw SQL, it does not follow the attribute; missing it makes seeded rows depend on the alias staying.
   - `docs/migration-plan/13-seedz-dev-seeding.md:204–209` sentence flips from "`"Disabled"` (capital) is right" to the new token.
   - Add a kernel test pinning the new shape, mirroring `nbf_offset_serde_wire_shape_is_pinned`, and (recommended even for Option B) a dual-accept test asserting `from_value(json!("Disabled"))` still decodes via the alias.
4. Compat: public Rust API unchanged (same type, same variants). Public wire breaks for non-Rust/pinned-old-SDK senders — mitigated by decode aliases (accepts both forever), but note *emission* changes, so consumer tests asserting the response token flip (exactly our `authorities.hurl:86`). Changelog entry under `changelogs/`. Cross-reference `OXA-000027:92`, whose "do not touch `contract.rs:319`" bullet becomes stale (its *own* change still must not touch it; only this ticket deletes that marker).

**Option B — document-as-is (legitimate P3 resolution; recommended default):** the casing is correct, tested, and load-bearing in stored data; the cost of Option A (public wire break + JSONB migration + 9 file edits) buys only cosmetics. Concretely: add a doc comment on `TotpSettings` in the kernel (`authorities/mod.rs:54`) — "intentional wire tokens: PascalCase externally-tagged variants; stored DB JSON and published API depend on this — do NOT add `rename_all`" — and rewrite the `contract.rs:319–321` marker from `BUG(pinned)` into an intentional-quirk note so the register item retires without pretending a bug will be fixed (the `BUG(pinned)` convention implies an eventual flip; here the honest resolution is "won't fix — pinned by storage format"). Update `BUGS_AND_NOTES.md:57` disposition. If later evidence of consumer pain appears, Option A's flip list above is already fully enumerated.

**Do not** half-do Option A (attribute without aliases/migration/seedz+hurl flips) in any form — the mixed-casing failure mode in Analysis is data-corrupting for un-updated rows.

## Verification

For Option B: `cargo test -p oxidauth-kernel --lib authorities` (kernel comment change only), plus marker hygiene — `grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-rs/src/client/users/contract.rs` shows the CLI-7 marker converted, while CLI-5's marker at `:230` remains; full `cargo test -p oxidauth --lib client::` unchanged (nothing behaviorally touched).

For Option A:
1. **Failing-before/passing-after kernel pin:** new kernel test asserting `serde_json::to_value(TotpSettings::Disabled) == json!("disabled")` and the `{"enabled":{…}}` object tag, plus alias decode of `"Disabled"`/`{"Enabled":{…}}` — red before the attribute, green after.
2. `cargo test -p oxidauth-kernel --lib authorities` and `cargo test -p oxidauth-services --lib bootstrap` (flipped `:1434` assertion).
3. Repository round-trips against a real DB (`bin/database_test.sh` convention): `cargo test -p oxidauth-postgres authorities::insert_authority authorities::select_all_authorities` — proves the new token round-trips through the JSONB column.
4. Migration proof: run the new migration against a DB seeded with old rows (bootstrap + seedz), then `SELECT settings->>'totp' FROM authorities;` returns `disabled` with zero rows retaining `Disabled`/an `Enabled` key; re-run to prove idempotency; then boot the renamed binary and hit `GET /api/v1/authorities` — decode succeeds on migrated rows *and*, with aliases, on a deliberately re-inserted pre-migration row.
5. SDK contracts: `cargo test -p oxidauth --lib client::authorities client::users::authorities` green with the re-canned `contract.rs:322` payload; temporarily restoring `"Disabled"` in the canned response must fail with aliases removed (proving the pin flipped) and pass with them present (proving the shim).
6. Live e2e: `./src/oxidauth/hurl.sh` (the `authorities.hurl:66/86/121`, `user_authorities.hurl:30` flips pass against the running api) — this is the suite that currently *certifies* PascalCase, so its green run after the flips is the server-side contract proof.
7. seedz: reseed via `src/seedz` and re-run its attach probes (`docs/migration-plan/13-seedz-dev-seeding.md`); confirm the fixed-string fixture decodes.

## Decision (2026-09-29) — OPTION A chosen by owner (coordinated snake_case rename), scheduled
- **Review tier: Tier 1 (docs-only) item 9/9 ONLY under Option B — reclassified: Option A chosen ⇒ now a HARD data-format slice; schedule with the migration-touching work, not the Tier-1 docs batch.**
- Owner overrode the ticket's recommended default (Option B): execute **Option A — coordinated rename to snake_case with compat aliases**. Accepts the wire-format break + JSONB migration + ~9-artifact flip. Implementation NOT started.
- Hard requirements (all load-bearing — "half-A is data-corrupting" per Analysis):
  1. Kernel `authorities/mod.rs:54`: `#[serde(rename_all = "snake_case")]` + `#[serde(alias = "Disabled")]` on `Disabled` + `#[serde(alias = "Enabled")]` on `Enabled`. Aliases make rollout/rollback order-free; they may be kept permanently.
  2. JSONB backfill migration beside `20250610213132_add_nbf_offset_...`: scalar `jsonb_set(settings,'{totp}','"disabled"')` for `'Disabled'` rows; **object case** = key-rename `{"Enabled":{…}}` → `{"enabled":{…}}` (delete old key + `jsonb_set` inner object), both token forms in the `WHERE` clause; idempotent. Ship/run with the alias-capable binary; migrate before any binary WITHOUT aliases (`update_authority` re-serializes the whole blob — binary-only cutover would corrupt reads of un-migrated rows).
  3. Flip every pin/artifact: `contract.rs:322` payload `"Disabled"`→`"disabled"` + DELETE marker `:319–321` (this ticket alone; OXA-000027's own fix still must not touch it — update that stale `:92` note); `bootstrap/mod.rs:1434`; `insert_authority/mod.rs:127` + `select_all_authorities/mod.rs:84` `"Enabled"` key; `authorities.hurl:66/86/121` + `user_authorities.hurl:30`; **`seedz/fixtures.rs:49` raw SQL** (does NOT follow the attribute — missing it makes seeded rows alias-dependent); `docs/migration-plan/13-seedz-dev-seeding.md:204-209` blessing text.
  4. New kernel pin test mirroring `nbf_offset_serde_wire_shape_is_pinned` (failing-before/passing-after): `to_value(Disabled) == json!("disabled")`, `{"enabled":{…}}` tag, plus alias decode of `"Disabled"`/`{"Enabled":{…}}`. Changelog entry under `changelogs/`. Register CLI-7 retired.
- Emission changes even with aliases → external consumers asserting the response token (mirror of `authorities.hurl:86`) flip; changelog must say so.
- Sequencing notes: independent of the Tier-1 docs batch except shared register pass; test authoring follows `docs/TESTING.md` §3 once 66/68 land. Full Option-A verification (Verification §A1–A7: kernel pin, bootstrap, postgres round-trips, migration proof on old-token rows, SDK contract shim check via alias on/off, live hurl.sh, seedz reseed) is the acceptance gate.
- Rejected for the record: client-side-only rename (no client enum exists), half-A (attribute w/o aliases/migration/seedz flip).
