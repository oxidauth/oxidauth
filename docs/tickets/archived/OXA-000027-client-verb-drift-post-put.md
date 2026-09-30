# OXA-000027 — SDK `update_user`/`update_role` send POST; the api mounts PUT (live 405s)

**Original ID:** CLI-1 · **Severity:** P1 · **Type:** bug · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #11 of 27 · reviewed 2026-09-29 · SCHEDULED (front of batch — live-broken P1)


## Locations

Register row: `BUGS_AND_NOTES.md:51` (CLI-1).

- `src/oxidauth/oxidauth-rs/src/client/users/update_user.rs:32` — `self.post(&format!("/users/{}", user_id), user)` inside `impl UpdateUserTrait for Client`. **The register's `:69` is the `BUG(pinned)` comment in the test module (`:69–71`), not the defect.**
- `src/oxidauth/oxidauth-rs/src/client/roles/update_role.rs:32` — `self.post(&format!("/roles/{}", role_id), role)` inside `impl UpdateRoleTrait for Client`. Same drift: register's `:69` is the pin comment.
- Server mount (this is where the verb is chosen, not in the handler files the register names):
  - `src/oxidauth/oxidauth-api/src/server/api/v1/users/mod.rs:28` — `.route("/{user_id}", put(update_user::handle))`; no POST mount exists on that path.
  - `src/oxidauth/oxidauth-api/src/server/api/v1/roles/mod.rs:25` — `.route("/{role_id}", put(update_role::handle))`; no POST mount exists.
- PUT handlers (DTO-compatible, register claim verified):
  - `src/oxidauth/oxidauth-api/src/server/api/v1/users/update_user.rs:25–26` — `Path<UpdateUserPathReq>` + `Json(body): Json<UpdateUserBodyReq>` — the *same* `UpdateUserBodyReq` the wrapper serializes as its POST body (`client/users/update_user.rs:3,32`).
  - `src/oxidauth/oxidauth-api/src/server/api/v1/roles/update_role.rs:25–26` — `Path<UpdateRolePathReq>` + `Json<UpdateRoleReq>` — identical to the wrapper's body type (`client/roles/update_role.rs:3,32`).
- Pinned tests (the only pins of this behavior):
  - `src/oxidauth/oxidauth-rs/src/client/users/update_user.rs:69–71` (marker) + `update_user_route_contract` (`:72–100`), passing `"POST"` as the harness verb at `:77`.
  - `src/oxidauth/oxidauth-rs/src/client/roles/update_role.rs:69–71` (marker) + `update_role_route_contract` (`:72–96`), `"POST"` at `:77`.
- Why the harness cannot catch this: `src/oxidauth/oxidauth-rs/src/client/users/contract.rs` — `mount()` wires `Mock::given(method(verb))` with the verb the *test* supplies (`:70–82`), and `assert_single_authed_hit` filters `received_requests()` by that same verb (`:84–107`). The runner verifies wrapper↔mock self-consistency, never wrapper↔api-router agreement; the verb audit was a manual cross-check (see Analysis).
- Correct-verb siblings in the same crate (proof the fix path works): `client/authorities/update_authority.rs:45` and `client/users/authorities/update_user_authority.rs:37` already call `.put(...)` and their contract tests match the api's PUT mounts (`authorities/mod.rs:26`, `users/authorities/mod.rs:23`).

## Problem

Both update wrappers issue **POST**; the server exposes those paths only as GET/PUT/DELETE. Against a live server, every `client.update_user(..)` and `client.update_role(..)` call fails — axum answers **405 Method Not Allowed** (review log states it outright: "axum answers 405", `missing-tests.md:577`).

**Exact current state (register claims checked against the tree, 2026-09-29):**

1. **The register's line numbers have drifted** (the campaign's known pattern): `:69` in both wrapper files lands on the `BUG(pinned)` comment, which reproduces the register text verbatim; the defect is the `.post(` at `:32` in each file.
2. **The register's "pins flipped in `client/users/contract.rs::verb_drift_*` tests" is false twice over.** (a) No test named `verb_drift_*` exists anywhere in the repo — `grep -rn verb_drift src` returns zero matches; the actual pins live in the wrapper files themselves (`update_user_route_contract`, `update_role_route_contract`), not in `contract.rs`. (b) The pins are **not flipped**: both tests still hand `"POST"` to the harness (`update_user.rs:77`, `update_role.rs:77`) and the marker comments still say "Pin asserts the wrapper's actual verb". Read the register line as *intended* resolution ("pins to be flipped in this product slice") with a wrong test name — not as current state.
3. Consequently the tree is **live-broken but green**: wrappers POST, wiremock mounts/asserts POST, `cargo test -p oxidauth` passes — while the real routers (`users/mod.rs:28`, `roles/mod.rs:25`) accept only PUT. The test suite currently *certifies a contract the server rejects* (`missing-tests.md:577`: "Must-fix — the two verb pins certify a contract the server rejects").
4. The live e2e suite proves PUT is the server truth: `src/oxidauth/hurl/tests/users.hurl:115` (`PUT /api/v1/users/{{user_id}}`) and `roles.hurl:72` (`PUT /api/v1/roles/{{role_id}}`) pass today against the running api — the SDK wrapper is the sole outlier; the api is consistent with its own tests, docs convention (`OXA-000014` notes the id-keyed-update-PUT convention at `users/mod.rs:28`, `roles/mod.rs:25`, `authorities/mod.rs:26`), and hurl.

**Failure mechanism at the wire.** `Client::request` (`client/mod.rs:452–496`) never inspects the HTTP status: it sends `.json(&payload)` (`:473`) and unconditionally `res.json().await`s the reply (`:485`). Axum's 405 carries no `Response`-envelope JSON body, so deserialization fails and the caller gets `ClientError::new(ClientErrorKind::Other("failed to deserialize response"), …)` (`:488–491`) — the 405 status and its `Allow` header are completely swallowed. SDK users see a *parse* error for what is a routing error, which is maximally confusing and makes the bug look like a server malfunction.

## Analysis

**How it survived.** Three layers each validated the wrong thing:
- The mock client (`--features mock`, `client/mock.rs`) bypasses HTTP entirely, so mock-based consumer tests never see the verb.
- The E5 contract harness pins wrapper verb against a wiremock mounted with that same verb (see Locations) — structurally incapable of drift detection; its doc comment (`contract.rs:6`) only promises "the wrapper hits exactly `<base_url><route>` with the expected HTTP verb", where "expected" = the verb the test passes in.
- The api's own e2e (hurl) issues raw PUTs and never exercises the SDK. The only wrapper↔api cross-check was the manual route/verb audit recorded in `missing-tests.md:575`: **54/56 contract pins match the api routers exactly; the two exceptions are exactly `update_user` and `update_role`.** That row also records the fix is "drop-in" DTO-wise and that the bug pre-dates the test migration (`missing-tests.md:577`: verb is "pre-existing at HEAD", the migration diff on those files was reflow-only).

**Full sweep (this ticket re-derived all verb+route pairs from the tree; agrees with the E5 audit).** Wrapper verb (client file: call-site line) vs api mount:

| Endpoint | Wrapper | Api mount | Match |
|---|---|---|---|
| POST /auth/authenticate (state machine `client/mod.rs:244`) + `auth/register.rs:29` | POST | `auth/mod.rs:14–15` POST | ✓ |
| /auth/oauth2/redirect (`auth/oauth2/redirect.rs:26`) | POST | `oauth2/mod.rs:13` POST | ✓ |
| /auth/username_password/{forgot_password,update_password} (`:21` each) | POST | `username_password/mod.rs:10–11` POST | ✓ |
| GET /public_keys (jwks machinery `client/mod.rs:184`) | GET | `public_keys/mod.rs:15` GET | ✓ |
| POST /refresh_tokens (`refresh_tokens/exchange_refresh_token.rs:37`; machinery `client/mod.rs:344`) | POST | `refresh_tokens/mod.rs:8` POST | ✓ |
| authorities list/create/by_strategy/find/update/delete (`authorities/*.rs`, e.g. `update_authority.rs:45`) | GET/POST/GET/GET/**PUT**/DELETE | `authorities/mod.rs:19–27` (PUT on `/{authority_id}`) | ✓ |
| can (`can/mod.rs:28`) | GET | `can/mod.rs:13` GET | ✓ |
| invitations create/find/accept (`invitations/*.rs:33/:30/:44`) | POST/GET/POST | `invitations/mod.rs:14–16` | ✓ |
| permissions list/find/create/delete (`permissions/*.rs`) | GET/GET/POST/DELETE | `permissions/mod.rs:17–20` | ✓ |
| public_keys list/create/find/delete (`public_keys/*.rs`) | GET/POST/GET/DELETE | `public_keys/mod.rs:15–18` | ✓ |
| roles list/create/find_by_id/find_by_name/delete (`roles/*.rs`) | GET/POST/GET/GET/DELETE | `roles/mod.rs:22–27` | ✓ |
| **roles update (`roles/update_role.rs:32`)** | **POST** /roles/{id} | `roles/mod.rs:25` **PUT** | **✗ (this ticket)** |
| role→permission grants list/create/delete (`roles/permissions/*.rs`) | GET/POST/DELETE | `roles/permissions/mod.rs:16–18` | ✓ |
| role→role grants list/create/delete (`roles/roles/*.rs`) | GET/POST/DELETE | `roles/roles/mod.rs:16–18` | ✓ |
| settings save/fetch (`settings/save_setting.rs:28`, `fetch_setting.rs:28`) | POST / , GET /{key} | `settings/mod.rs:13–15` mounts the same handler on **both POST and PUT** + GET — double mount is legal axum 0.8 MethodRouter merge (verified in `missing-tests.md:575`), so the POST pin is sound | ✓ |
| users list/by_ids/create/find/by_username/delete (`users/*.rs`) | GET/POST/POST/GET/GET/DELETE | `users/mod.rs:24–33` | ✓ |
| **users update (`users/update_user.rs:32`)** | **POST** /users/{id} | `users/mod.rs:28` **PUT** | **✗ (this ticket)** |
| user→authority list/create/find/update/delete (`users/authorities/*.rs`, incl. `update_user_authority.rs:37` PUT) | GET/POST/GET/**PUT**/DELETE | `users/authorities/mod.rs:17–24` | ✓ |
| user→permission grants list/create/delete (`users/permissions/*.rs`) | GET/POST/DELETE | `users/permissions/mod.rs:15–17` | ✓ |
| user→role grants list/create/delete (`users/roles/*.rs`) | GET/POST/DELETE | `users/roles/mod.rs:15–17` | ✓ |

**Sweep conclusion: no other client wrapper has verb drift — the defect is exactly these two call sites**, matching the independent E5 audit. Server routes with *no SDK wrapper at all* (coverage gaps, deliberately not drift, do not fix here): `DELETE /invitations/{id}` (`invitations/mod.rs:17`), `POST /totp/validate` (`totp/mod.rs:8`), `GET /__meta/{healthcheck,livecheck}` + snake_case aliases (`meta/mod.rs:9–14`; the SDK's `get_version` wart is CLI-10, register line 60 — [CLI-10/OXA-000036 SUPERSEDED 2026-09-30 — phantoms: get_version/build_authorization never existed in SDK history (verified by full git-history grep); OXA-000036 closed as note.]).

**Edge cases.**
- The fix is a pure verb swap: `Client::put` (`client/mod.rs:507–514`) feeds the identical `request(Method::PUT, …)` path with the same `.json(&payload)` body — byte-identical payload, URL, bearer header; only the method token changes. The two working PUT siblings above are the existence proof.
- `handle_response`/error mapping, `RESOURCE`/`METHOD` labels, canned payloads, and `ClientMock` are untouched.
- Consumers using `--features mock` or the wiremock contract tests see zero compile-time change; consumers who hand-rolled workarounds (raw `PUT` via reqwest, or a server-side POST alias in a fork) are the only behavioral-compat question — see Resolution.
- Once the verb is fixed, SDK callers newly reach the handler — and with it `OXA-000015` (`update_user` null-overwrite: omitted fields written as NULL). That bug is *already* live via hurl/raw HTTP today; fixing the verb does not create it, but it converts it from "raw-HTTP only" to "SDK-reachable" and should be sequenced/communicated with OXA-000015.

## Impact

- **Who:** every external SDK consumer of `oxidauth` (oxidauth-rs) that calls `update_user` or `update_role` against a real server — the two basic admin rename/mutate primitives. Success rate today: **0%**. Server-side flows are unaffected (`accept_invitation` invokes the `UpdateUserService` use case in-process, not over HTTP — `oxidauth-services/src/invitations/accept_invitation.rs:78–83`), and `oxidauth-cli`/`bin/` contain no `.update_user(`/`.update_role(` callers (grep verified), so blast radius is the published SDK.
- **What:** guaranteed hard failure, surfaced as `ClientErrorKind::Other("failed to deserialize response")` with the 405 hidden (`client/mod.rs:485–491`) — consumers reasonably conclude the *server* is broken. The two methods are also the natural targets for permission-gated admin tooling built on the SDK.
- **Test-surface damage:** the two green contract tests actively document POST as the api contract, so the next reader/agent trusting `src/client/**` tests will re-introduce or route around the bug instead of fixing it (`missing-tests.md:577` flagged precisely this: "this slice's new tests are the first document that claims these verbs ARE the api contract").
- **Severity:** register says P1. The failure mode is loud, not silent data loss, but it is a 100% broken core capability of the product's client crate plus a falsified contract test — P1 stands.

## Proposed resolution

**Client-side two-line fix (recommended; the register's option):**

1. `client/users/update_user.rs:32` — `.post(` → `.put(`; `client/roles/update_role.rs:32` — `.post(` → `.put(`. Nothing else in the product code changes.
2. Flip the pins, per the campaign rule "fix a bug = flip the pinned assertion + delete the marker":
   - `update_user.rs`: verb arg `"POST"` → `"PUT"` (`:77`), delete marker comment `:69–71`.
   - `update_role.rs`: verb arg `"POST"` → `"PUT"` (`:77`), delete marker comment `:69–71`.
   The harness needs no change — verb is a parameter (`contract.rs:70–82,112–114`), so wiremock now mounts/asserts PUT and all three legs (route+verb+bearer, payload decode, error/empty-payload) re-run unchanged against the corrected verb.
3. **Do NOT touch** the neighboring client markers/pins: `contract.rs:230` (CLI-5 raw-wrapper pass-through), `contract.rs:319` (CLI-7 TotpSettings casing), `create_invitation.rs:73`/`find_invitation.rs:75`/`list_all_authorities.rs:70` (CLI-6 METHOD typos), `mod.rs:966/:1074/:1290` (CLI-2/4/3), `auth/authenticate.rs:60` (CLI-8), `forgot_password.rs:45` (CLI-9). No other pin in the repo touches these two verbs, so this fix is the *only* marker to delete.

   *(Update 2026-09-30, superseded: the `contract.rs:319` CLI-7 marker in this census was **deleted by OXA-000033** — the `TotpSettings` snake_case rename flipped the canned `"totp"` payload and retired the pin. OXA-000027's own fix still must not touch `contract.rs:319` — there is nothing there to touch, and its fix must NOT re-add the marker. All other listed markers stand.)*
4. **Reject the server-side alternative** (mount POST, `missing-tests.md:587` mentions it): hurl e2e (`users.hurl:115`, `roles.hurl:72`) and three api resources already treat PUT as the update verb; adding POST on `/{id}` would collide with the codebase's create-semantics-on-collection convention, double the public surface of two endpoints forever, and break the 54-honest-test story by making those two wrappers permanent exceptions. Fix the two wrappers.

**Compat concerns:**
- **Public SDK Rust API: zero change** — `UpdateUserTrait::update_user` / `UpdateRoleTrait::update_role` signatures, DTOs (`oxidauth-http` `UpdateUserBodyReq`/`UpdateRoleReq`), `ClientMock`, and re-exports are untouched; consumers recompile unchanged.
- Wire behavior changes from "405 for everyone" to "works": strictly a repair; nobody can be relying on the current behavior against the published server. The one visible-to-consumers case is *their* wiremock tests that copied our `"POST"` contract string — call it out in the changelog (`changelogs/`) as a test-only break for SDK consumers who mirrored our contract tests.
- No server change, no schema change, no `oxidauth-http`/`oxidauth-kernel` change; docs checked: neither `README.md` nor `docs/*.md` references these wrappers' verbs (grep verified), so no doc updates beyond the changelog.
- Sequence with **OXA-000015** (update_user null-overwrite, DATA-2): after this fix, SDK callers hit it. Not a blocker — raw HTTP already hits it today — but worth one sentence in the release note.

## Verification

1. **Contract tests (the flipped pins):** `cargo test -p oxidauth --lib client::users::update_user` and `--lib client::roles::update_role` (package `oxidauth` lives in `oxidauth-rs/`). Both must pass with `"PUT"`; temporarily restoring `.post(` must make each fail with the "wrapper must issue exactly one PUT …" assert (`contract.rs:93–97`) — a failing-before/passing-after regression for the verb.
2. **Whole E5 suite:** `cargo test -p oxidauth --lib client::` (and `--features mock`) — all wrapper contracts stay green; expect the same counts as `missing-tests.md:569` (88 passed baseline).
3. **Live repro before, live pass after** (compose stack + `cargo run -p oxidauth-api`, repo-root `.env`):
   - Server truth: `curl -X POST $HOST/api/v1/users/{id} -H "authorization: Bearer <jwt>" -d '{"user":{…}}'` → **405** (no JSON envelope); `curl -X PUT …` same body → 200/`success:true`. Same pair for `/api/v1/roles/{id}`.
   - SDK: throwaway binary against the live server — `client.update_user(id, body).await` and `client.update_role(id, body).await`: pre-fix error kind `Other("failed to deserialize response")`; post-fix `Ok(UpdateUserRes/UpdateRoleRes)` with the mutated entity echoed.
4. **Server-side e2e unaffected:** `./src/oxidauth/hurl.sh` — `users.hurl:115` and `roles.hurl:72` PUTs were already passing; they must keep passing (this ticket changes nothing server-side).
5. **Sweep re-audit:** re-extract verb+route from all wrapper tests and diff against the api router mount table (the `missing-tests.md:575` procedure) — result must become "N/N match, zero exceptions"; then update `missing-tests.md` E5 row (`:271`, currently "54 pins match the api … 405-drift") and its open must-fix items (`:577`, `:587`), plus drop register line `BUGS_AND_NOTES.md:51`.
6. **Marker hygiene:** `grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-rs --include='*.rs'` → the two `update_*.rs:69` markers are gone; the CLI-2…CLI-10 markers listed in Resolution step 3 all remain.

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- Reviewed jointly (independent 56-pair verb re-sweep + owner ruling 2026-09-29). P1 stands: `client.update_user`/`update_role` are 0% functional against a live server today, and the failure presents as `Other("failed to deserialize response")` because `Client::request` swallows the 405 (`client/mod.rs:485-491`). Re-sweep result: exactly the two drifts (`update_user.rs:32`, `update_role.rs:32`); all other 54 wrapper↔mount pairs match.
- Register corrections ratified: CLI-1's "pins flipped in `client/users/contract.rs::verb_drift_*`" is false twice — no `verb_drift_*` test exists, and the pins (`update_user.rs:77`, `update_role.rs:77`) still hand the harness `"POST"`. Read the register line as intended resolution with a wrong test name.
- Steps 1-2 approved: `.post(` → `.put(` at `:32` in each file (byte-identical payload/URL/bearer — `update_authority.rs:45` / `update_user_authority.rs:37` are the working existence proofs); pins flip verb to `"PUT"`, markers `:69-71` deleted; harness untouched (verb is a parameter). Step 3's do-not-touch marker census (CLI-2…CLI-10, `contract.rs:230/:319`) approved — exactly these two markers die.
- Step 4 ratified: server-side POST mount rejected — hurl (`users.hurl:115`, `roles.hurl:72`), the router, and the collection-create/id-update convention all fix PUT as the truth.
- Explicitly out of scope: the 405-swallowing mechanism is OXA-000031's (Tier 4, unreviewed) problem; this ticket changes verbs only.
- Sequencing: after this lands, SDK callers newly reach OXA-000015 (null-overwrite) — already live via raw HTTP, not a blocker; release note gets a sentence. Changelog must flag test-only break for consumers who mirrored our `"POST"` contract string.
- Acceptance: `cargo test -p oxidauth --lib client::users::update_user` / `client::roles::update_role` green with `"PUT"` and temporarily restoring `.post(` reddens them via the harness's single-hit assert; `hurl.sh` unchanged-green; sweep re-audit zero exceptions; then strike `BUGS_AND_NOTES.md:51`, update `missing-tests.md:271/:575/:577`.
