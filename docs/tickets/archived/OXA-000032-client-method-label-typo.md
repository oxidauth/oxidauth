# OXA-000032 — Client wrapper METHOD labels misspell/misname methods in error copy

**Original ID:** CLI-6 · **Severity:** P3 · **Type:** bug · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #12 of 27 · reviewed 2026-09-29 · SCHEDULED


## Locations

Register citations point at `BUG(pinned)` comments in the test modules, not at the defective
constants — classic marker drift. Verified 2026-09-29:

| Register cite (actual content) | Real defect | Defective value |
| --- | --- | --- |
| `oxidauth-rs/src/client/invitations/create_invitation.rs:73` (`BUG(pinned)` comment) | `create_invitation.rs:14` | `const METHOD: &str = "create_invitaion";` |
| `oxidauth-rs/src/client/invitations/find_invitation.rs:75` (`BUG(pinned)` comment) | `find_invitation.rs:11` | `const METHOD: &str = "create_invitaion";` |
| `oxidauth-rs/src/client/authorities/list_all_authorities.rs:70` (`BUG(pinned)` comment) | `list_all_authorities.rs:12` | `const METHOD: &str = "find_authority_by_strategy";` |

Where the label surfaces (all read and verified):

- `client/mod.rs:651-677` — `handle_response(resource, method, response)`; on a `success:true`
  envelope with no `payload` it builds
  `ClientError::new(ClientErrorKind::EmptyPayload(resource, method), None)` (line 674).
- `client/mod.rs:599-639` — `ClientErrorKind::EmptyPayload(Resource, &'static str)` is `pub`, and
  `ClientError`'s `Display` renders the label verbatim (line 626):
  `"received an empty payload when a response payload was expcected for resource {resource} method {method}"`.
- `client/users/contract.rs:171-193` — harness leg 3 asserts both the `EmptyPayload(r, m)` variant
  fields and that `Display` contains `resource {r} method {m}`; the per-wrapper test passes the
  expected `(resource, method)` tuple, which is what pins the bad strings today.

## Problem

Three `oxidauth-rs` endpoint wrappers carry copy-pasted `METHOD` constants. These labels are not
internal trivia — they are the literal text an SDK caller sees when the server returns a payloadless
success envelope:

1. `create_invitation.rs:14` — misspelling of its own method: `"create_invitaion"` (letters
   transposed in `invitation`). Error copy: `… for resource user method create_invitaion`.
2. `find_invitation.rs:11` — wholesale copy-paste of the same broken string from
   `create_invitation.rs`. Doubly wrong: the copy-paste names the *wrong method*, and the string it
   copied is itself the typo. Error copy for `find_invitation` says `create_invitaion`.
3. `list_all_authorities.rs:12` — copy-paste from its sibling
   `authorities/find_authority_by_strategy.rs:9`, which owns that label legitimately.
   `list_all_authorities` errors therefore point triage at a *different, real* endpoint
   (`find_authority_by_strategy` exists and works), actively misdirecting debugging.

## Analysis

**Mechanism.** Every non-raw wrapper's `Client` impl ends with
`handle_response(RESOURCE, METHOD, resp)?`. `METHOD: &'static str` flows straight into
`ClientErrorKind::EmptyPayload` and out through `Display`. Only the `Client` impls use it —
`ClientMock` impls bypass `handle_response`, and `#[tracing::instrument]` on the wrapper methods
records function names, not `METHOD` — so the sole surfaces are the `EmptyPayload` variant fields
(public via the `pub` enum) and the rendered `Display` sentence.

**Full sweep.** All 52 `const METHOD` declarations under `src/client/**` were compared
mechanically against their wrapper file stems (reproduce with the grep in *Verification*):

| Wrapper | Label | Verdict |
| --- | --- | --- |
| `invitations/create_invitation.rs` | `"create_invitaion"` | ❌ typo of own method |
| `invitations/find_invitation.rs` | `"create_invitaion"` | ❌ wrong method + typo |
| `authorities/list_all_authorities.rs` | `"find_authority_by_strategy"` | ❌ wrong method |
| remaining 49 wrappers (`auth/register`, `can`, 5× authorities, `accept_invitation`, 4× permissions, 4× public_keys, `exchange_refresh_token`, 6× roles + 3× role/permission grants + 3× role/role grants, 2× settings, 7× users + 5× user authorities + 3× user permission grants + 3× user roles) | each equals its method name | ✅ |

(The sweep flags `can/mod.rs` label `"can"` vs stem `mod` as a false positive; the label is correct.)

**Adjacent, deliberately out of scope:**

- The same `Display` sentence contains its own typo — `was expcected for` (`mod.rs:626`) — explicitly
  pinned by `client_error_display_is_pinned` (`mod.rs:1358-1384`, comment at 1370: "pin actual copy
  incl. the 'expcected' typo"). CLI-6 names only the METHOD labels; fix the sentence typo as its own
  register line since it has the same flip mechanics (one format string + two pinned assertions,
  `mod.rs:1372` and `mod.rs:1474`).
- The invitation wrappers report `RESOURCE = Resource::User` ("resource user"). The `Resource` enum
  (`mod.rs:557-573`) has no `Invitation` variant, so this reads as intentional (an invitation
  materializes a user); not part of this item.

**Pins.** Exactly three tests pin the bad strings, each via the `contract()` leg-3 tuple, each with
a `BUG(pinned)` comment acknowledging the defect:

| Test | Pin to flip | Marker to remove |
| --- | --- | --- |
| `create_invitation_route_contract` | `("user", "create_invitaion")` → `("user", "create_invitation")` (`create_invitation.rs:78`) | lines 73-74 |
| `find_invitation_route_contract` | `("user", "create_invitaion")` → `("user", "find_invitation")` (`find_invitation.rs:81`) | lines 75-77 |
| `list_all_authorities_route_contract` | `("authority", "find_authority_by_strategy")` → `("authority", "list_all_authorities")` (`list_all_authorities.rs:76`) | lines 70-72 |

Not affected (verified): `find_authority_by_strategy`'s own contract test (`…/find_authority_by_strategy.rs:82`)
keeps its label; `client_error_display_is_pinned` uses `create_user_authority`;
`handle_response_empty_payload_names_resource_and_method` (`mod.rs:1459-1478`) uses the synthetic
`"get_user"`.

## Impact

- **Who:** SDK consumers only. Server, wire format, request routing, and every function signature
  are untouched — the labels are produced and consumed entirely client-side.
- **When:** only on the empty-payload path (server answers `success:true` without `payload`). It is
  a diagnostic-quality defect: it does not break the happy path, it makes the error *lie*.
- **Consequence:** a caller triaging a `create_invitaion` error greps for a method that does not
  exist; a `find_invitation` failure is attributed to `create_invitation`; a
  `list_all_authorities` failure is attributed to the unrelated-but-real `find_authority_by_strategy`
  endpoint (see OXA-000022 for that endpoint's actual server-side bug — this label copy-paste is
  unrelated to it, only confusable with it).
- **Compat:** `ClientErrorKind::EmptyPayload` is public with a `&'static str` field, so a caller that
  string-matches the *typo'd* label (or the rendered sentence) would break on the fix. That is
  matching against text that has always been wrong; the corrected label is the behavior any such
  caller would have wanted. Same class and same tolerance as the OXA-000027 (CLI-1) pin flips in
  this harness — that ticket also changes caller-visible wrapper behavior and flips its
  `contract.rs::verb_drift_*` pins as part of the fix.

## Proposed resolution

One small cutover, no shims:

1. Fix the three constants:
   - `create_invitation.rs:14` → `"create_invitation"`
   - `find_invitation.rs:11` → `"find_invitation"`
   - `list_all_authorities.rs:12` → `"list_all_authorities"`
2. Flip the three leg-3 tuples in the table above and delete the three now-stale `BUG(pinned)`
   comments. No new tests needed: contract leg 3 already pins every wrapper's
   `(resource, method)` tuple and its appearance in `Display`, which is exactly the regression guard.
3. Do **not** touch `find_authority_by_strategy.rs` (correct label) or the `"expcected"` sentence
   typo — file the latter separately as described above.

## Verification

Sweep — after the fix this must print only the `can/mod.rs` false positive and nothing else:

```sh
grep -rn 'const METHOD' src/oxidauth/oxidauth-rs/src/client/ \
  | sed -n 's/^\([^:]*\):.*= "\([^"]*\)";.*/\1 \2/p' \
  | while read -r f lbl; do stem=$(basename "$f" .rs); \
      [ "$lbl" != "$stem" ] && echo "MISMATCH $f label=\"$lbl\" expected=\"$stem\""; done
grep -rn '"create_invitaion"' src/oxidauth/   # expect: no matches
```

Targeted tests — each exercises the wrapper end-to-end against wiremock, including leg 3, which
fails unless the constant and the flipped tuple agree and the corrected label reaches `Display`:

```sh
cargo test -p oxidauth --lib client::invitations
cargo test -p oxidauth --lib client::authorities::list_all_authorities
```

No-change confirmation: `cargo test -p oxidauth --lib client::` should pass with
`client_error_display_is_pinned` and `handle_response_empty_payload_names_resource_and_method`
untouched. (Per ticket workflow, full-suite validation runs once centrally; the commands above are
the scoped proof for this change.)

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- Reviewed jointly (52-wrapper `const METHOD` re-sweep + owner ruling 2026-09-29): exactly the three defective labels (`create_invitaion` typo of own method; `find_invitation` carrying the same broken string wholesale; `list_all_authorities` impersonating the real `find_authority_by_strategy` endpoint), zero other mismatches; `can/mod.rs` confirmed a false positive (label correct vs file stem).
- Steps 1-2 approved: constants → their own method names; flip the three leg-3 tuples (`create_invitation.rs:78`, `find_invitation.rs:81`, `list_all_authorities.rs:76`); delete the three `BUG(pinned)` markers. No new tests — contract leg 3 already pins the `(resource, method)` tuple and its `Display` appearance, which is the permanent regression guard.
- Step 3 boundary held, with the gap closed: the `was expcected` sentence typo at `client/mod.rs:626` (verified present; pinned at `:1372` and `:1474`) is NOT folded in — it has **no register line today** (client section tops out at CLI-10), so the fix commit must add it as register line **CLI-11** with its own flip mechanics (one format string + the two pinned assertions). Separate line, separate flip, keeps the one-ticket-one-marker-census discipline.
- `RESOURCE = Resource::User` on the invitation wrappers stays: the `Resource` enum has no `Invitation` variant; intentional, not part of this item.
- Compat: `ClientErrorKind::EmptyPayload` is `pub` with a `&'static str` — a consumer string-matching the typo'd label changes behavior; that code was matching text that always lied, and the corrected label is what any such matcher wanted. Changelog one-liner.
- Acceptance: the ticket's Verification sweep prints only the `can` false positive; `cargo test -p oxidauth --lib client::invitations` and `client::authorities::list_all_authorities` green; `client_error_display_is_pinned` and `handle_response_empty_payload_names_resource_and_method` untouched-green; `grep '"create_invitaion"'` empty repo-wide. (Command correction at implementation: the package is `oxidauth`, not `oxidauth-rs` — the literal `-p oxidauth-rs` fails to resolve.)
