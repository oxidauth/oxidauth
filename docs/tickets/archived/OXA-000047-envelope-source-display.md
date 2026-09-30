# OXA-000047 — Envelope `source` field has no documented contract; register claims a nonexistent Debug promise

**Original ID:** SRV-11 · **Severity:** P3 · **Type:** note · **Status:** done
**Review tier:** Tier 1 (docs-only), ranked item 8/9 · IMPLEMENTED — byte-identical wire output held (verified 2026-09-30 review pass)


## Locations

- **Register line:** `BUGS_AND_NOTES.md:76` — "Envelope `source` stringifies the source's Display, not Debug (consumers see human text where `from_error` promises Debug)."
- **Actual stringification site:** `src/oxidauth/oxidauth-kernel/src/error.rs:40-42` — inside `IntoOxidAuthError::into_error` for `Box<dyn Error + Send + Sync>` (`error.rs:32-45`): `source: self.source().map(|s| s.to_string())`.
- **Register drift (line number, confirmed):** the cited `error.rs:110` is the `// BUG(pinned): 'source' stringifies the source's *Display*, not its Debug …` comment inside the unit test `into_error_captures_the_error_source_as_display` (test spans `error.rs:103-116`) — the pin, not the defect. Same drift pattern as SRV-10/OXA-000046 and CLI-4/OXA-000030.
- **Register drift (contract claim, refuted):** there is **no `from_error` anywhere in the workspace** — repo-wide grep matches only the register line itself. The method is `into_error`. Moreover `into_error`, the `IntoOxidAuthError` trait (`error.rs:24-30`), and every field of `OxidAuthError` (`error.rs:7-22`) carry **zero doc comments**, and no RFC, changelog, or README text mentions the envelope's `source` rendering (grep of `rfcs/`, `changelogs/`, `README.md`, `docs/` outside tickets). The "promises Debug" premise is unsupported: the only "promise" is an inferred convention from the struct having a separate `debug` field.
- **Envelope carrier:** `src/xlib/http/src/lib.rs:77-95` — `Response::error()` serializes any `Serialize` value into `errors: Option<Vec<serde_json::Value>>` (field at `lib.rs:19`), so the `OxidAuthError` JSON object (including its `source` key) lands verbatim on the wire.
- **Producers:** 56 `err.into_error()` handler call sites under `src/oxidauth/oxidauth-api/src/server/api/v1` (e.g. `auth/authenticate.rs:42`, `authorities/create_authority.rs:56`, `invitations/find_invitation.rs:63`), all on `BoxedError` returned by `oxidauth-services` use cases.
- **Who gets a non-null `source`:** boxed errors with a real cause chain — e.g. `PgError` implements `Error::source()` exposing the wrapped `sqlx::Error` / `VarError` (`src/xlib/postgres/src/lib.rs:260-261`, pinned by `postgres/src/lib.rs:391-421`). Bare `sqlx::Error::RowNotFound` has `source() == None`, so the key is omitted entirely (`#[serde(skip_serializing_if = "Option::is_none")]`, `error.rs:20-21`).
- **Consumers:** none parse `errors[0].source` — see Problem.

## Problem

`into_error` fills the envelope as: `display` = top-level `Display` (`error.rs:36`), `debug` = top-level `Debug` (`error.rs:37`), `source` = **`Display`** of the immediate `std::error::Error::source()` cause (`error.rs:40-42`). The register flags the `source` choice as a defect on the theory that a `from_error` contract promises Debug. Both halves of that theory are wrong:

1. No `from_error` exists (drift), and no doc anywhere states a Debug expectation for `source`.
2. No in-tree consumer reads the `source` field at all, so today's "human text" harms nobody:
   - **hurl:** every envelope assertion targets `$.errors[0].debug` — 24 asserts across `authenticate.hurl`, `authorities.hurl`, `exchange.hurl`, `permissions.hurl`, `public_keys.hurl`, `register.hurl`, `roles.hurl`, `settings.hurl`, `users.hurl` (`contains "RowNotFound"`, `"duplicate key"`, `"PermissionNotFoundError"`, …). **Zero** hurl asserts touch `errors[0].source`. This also resolves the question raised in OXA-000021: the `RowNotFound` text hurl matches lives in the **`debug`** field (top-level derived Debug — pinned in-kernel at `error.rs:93-95`, with the mock's comment at `error.rs:55-57` saying exactly that); `source` either carries different prose or is absent (a bare `RowNotFound` has no source).
   - **oxidauth-rs:** `handle_response` (`src/oxidauth/oxidauth-rs/src/client/mod.rs:651-677`) JSON-stringifies each whole error value (`e.to_string()` at `mod.rs:663-667`) and joins it into `ClientError.source` — so envelope *source text* rides inside the joined dump, but none of the tests pin its content: `handle_response_api_error_joins_errors_into_source` (`mod.rs:1433-1457`) and `authenticate_failure_envelope_surfaces_server_errors_in_source` (`mod.rs:866-900`) mock plain-string errors (`"e1"`, `"invalid credentials"`), as does `client/users/contract.rs:164-168`. Related client-side envelope gaps are already ticketed (OXA-000030, OXA-000031); neither depends on how `source` renders.
   - The only assertions on `source` content are the kernel's own pins: `error.rs:112` (`BUG(pinned)`, Display text `"database row not found"`) and `error.rs:142` (same text through JSON, in `oxid_auth_error_json_includes_present_source`).

The real, defensible complaint that survives verification: the `display` / `debug` / `source` rendering rules are an undocumented convention — hurl, `missing-tests.md` (C5, and the note at `missing-tests.md:302` that the mocks "prove the `debug` field is derived-Debug while `display`/`source` are Display"), and the pinned tests all enforce it by folklore instead of a contract comment.

## Analysis

Making `source` Debug would be actively worse, not just unnecessary:

- **Redundancy:** top-level `Debug` of a boxed error already embeds the nested source's Debug — the test's own comment notes `debug == "Wrapped(RowNotFound)"` "includes the nested Debug" (`error.rs:114-115`). Debug-ing `source` would duplicate the same machine text under two keys with zero new information, while destroying the only distinct signal `source` provides (the human-readable cause, which neither other key carries when the top-level type renders verbosely, e.g. `Database(Error { … })` Debug blobs).
- **Display is the semantically correct rendering** for a field named after `std::error::Error::source()` in a wire envelope aimed at operators and SDK users; the envelope keeps `debug` precisely for machine-shaped text (hurl asserts against it, per `error.rs:55-57`).
- **Wire-compat risk is asymmetric:** a Display→Debug flip changes the JSON body of every failure envelope whose boxed error has a cause chain (i.e. every `PgError`-wrapped DB error, `xlib/postgres/src/lib.rs:260-261`) for any external client already parsing `source` as prose since the field shipped — in exchange for nothing this repo needs.
- The `missing-tests.md` audit trail (`:248` C5, `:302`) already treats the current split as the tested-by-design mechanism, confirming the implementation matches intent; it is only the *documentation* that is missing.

## Impact

No behavioral impact on any current consumer: `source` text is unread by hurl, oxidauth-rs, or `bin/`. The impact is maintenance-only — the register's false premise (a Debug promise by a `from_error` that doesn't exist) invites a future contributor to "fix" the code to match the phantom contract, duplicating text on the wire and flipping the two kernel pins. The `BUG(pinned)` marker at `error.rs:110` actively mislabels correct behavior as a bug.

## Proposed resolution

**Document to behavior (P3 proportionate; zero wire change).**

1. Add doc comments on `IntoOxidAuthError` / `into_error` and the `OxidAuthError` fields (`error.rs:7-30`) pinning the contract: `display` = top-level `Display`; `debug` = top-level `Debug` (transitively includes the cause's Debug); `source` = `Display` of the immediate `std::error::Error::source()` cause, omitted when absent.
2. Rewrite the `BUG(pinned)` comment at `error.rs:110-112` as a plain contract comment (`into_error_captures_the_error_source_as_display` and `oxid_auth_error_json_includes_present_source` keep their assertions unchanged — they are correct pins, not bug pins).
3. Amend/annotate `BUGS_AND_NOTES.md:76` so the phantom `from_error`-promises-Debug claim does not resurface.
4. **Rejected alternative, for the record:** flipping `error.rs:42` to `|s| format!("{:?}", s)` would flip exactly two assertions — `error.rs:112` (`Some("database row not found")` → `Some("RowNotFound")`) and `error.rs:142` (same text via JSON) — plus the test name at `error.rs:103`; **no hurl or oxidauth-rs assertion flips** (verified above), which is itself evidence the change serves no consumer.

No compat concerns: the recommended path emits byte-identical JSON on every route.

## Verification

- `cargo test -p oxidauth-kernel` — the four envelope tests at `error.rs:84-146` must pass unchanged (doc-only change; if the rejected Debug variant were chosen, `error.rs:112` and `error.rs:142` are the assertions that flip).
- `bin/hurl.sh` (two-pass suite, per `README.md:75-79`) — all 24 `$.errors[0].debug` asserts must stay green untouched, demonstrating the `debug` field's independence from the `source` decision.
- `cargo test -p oxidauth-rs` — confirms the joined-envelope tests (`client/mod.rs:866-900`, `1433-1457`, `users/contract.rs:164-168`) are indifferent to the doc change.

## Decision (2026-09-29) — ACCEPTED as proposed (document-to-behavior), scheduled; no implementation started
- **Review tier: Tier 1 (docs-only), item 8/9** (original list position 7).
- All three edits approved, zero wire change (byte-identical JSON on every route):
  1. Doc comments on `IntoOxidAuthError` / `into_error` / `OxidAuthError` fields (`error.rs:7-30`): `display` = top-level Display; `debug` = top-level Debug (transitively includes the cause's Debug); `source` = Display of the immediate `std::error::Error::source()` cause, omitted when absent.
  2. `BUG(pinned)` at `error.rs:110-112` rewritten as a plain contract comment — assertions in `into_error_captures_the_error_source_as_display` and `oxid_auth_error_json_includes_present_source` stay unchanged (correct pins, not bug pins).
  3. `BUGS_AND_NOTES.md:76` amended: no `from_error` exists; the Debug promise was phantom.
- Debug-flip alternative stays REJECTED on the record: duplicates `debug` content, destroys `source`'s only distinct signal, changes wire JSON for every `PgError`-wrapped failure, and flips zero hurl/SDK assertions — no consumer wants it.
- Side effect noted: closes OXA-000021's open question — hurl-matched `RowNotFound` text lives in `debug`; `source` may be absent (bare `RowNotFound` has no cause chain).
- Related tickets OXA-000030/000031 (client envelope handling) unaffected — neither depends on this decision.
- Acceptance: `cargo test -p oxidauth-kernel` four envelope tests unchanged-green; `bin/hurl.sh` 24 `$.errors[0].debug` asserts untouched-green; `cargo test -p oxidauth-rs` indifferent-green.
