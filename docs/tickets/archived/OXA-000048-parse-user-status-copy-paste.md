# OXA-000048 — `ParseUserStatusErr` Display is a byte-for-byte copy of the user_kind message: every bad-status parse blames the wrong column

**Original ID:** SRV-12 · **Severity:** P3 · **Type:** bug · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #10 of 27 · reviewed 2026-09-29 · SCHEDULED


## Locations

Register cites `oxidauth-kernel/src/users/mod.rs:264` — **marker drift**: `:264` is the `BUG(pinned)` comment header inside the test module, not the defect. The claim itself is accurate; `missing-tests.md:301` already records the real coordinates ("`ParseUserStatusErr` Display emits `"user_kind"` (`users/mod.rs:146` vs the correct `:73`)"). Verified sites:

- **Defect:** `src/oxidauth/oxidauth-kernel/src/users/mod.rs:144-148` — `impl fmt::Display for ParseUserStatusErr`; the offending line is `:146`: `write!(f, "failed to parse user_kind, unknown: {}", self.unknown)`.
- **Correct sibling:** `:71-75` (`:73`), `impl fmt::Display for ParseUserKindErr` — same format string, and here it is true.
- **Producers (who decides which label is right):** `UserKind::FromStr` `:48-64` returns `ParseUserKindErr` only at `:56`, vocabulary `human|api` via consts `HUMAN`/`API` (`:36-37`, matched at `:53-54`). `UserStatus::FromStr` `:103-120` returns `ParseUserStatusErr` only at `:112`, vocabulary `enabled|invited|disabled` (consts `:89-91`, matched as **raw literals** at `:108-110`). The two error structs are structurally identical (`:66-69`, `:139-142`, both `{ unknown: String }`) with empty `Error` impls (`:77-78`, `:150-151`) — i.e. nothing about either type carries the enum name, so Display is the *only* thing that says which type failed.
- **Pinned test:** `user_status_rejects_unknown_strings` `src/oxidauth/oxidauth-kernel/src/users/mod.rs:260-273` — marker `:264-265` (the register cite), pinning assertion `:266-269` (`"failed to parse user_kind, unknown: paused"`). The symmetric *correct* pin is `user_kind_rejects_unknown_strings` `:213-225` (`:216-219`). No other `BUG(pinned)` marker exists in this file.
- **Only production consumer:** `src/oxidauth/oxidauth-postgres/src/users/mod.rs` — re-export at `:4`; `TryFrom<UserRow> for User` `:46-59` parses both text columns (`kind` at `:49`, `status` at `:50`); `TryFromUserRowError` `:62-66` (derives `Debug`), `Display` `:68-76` interpolates the inner Display verbatim (`:71-72`), `Error::source` `:79-86` returns the inner error (`:82-83`), `From<ParseUserStatusErr>` `:94-98`.
- **Every user read-back path funnels through it:** `select_user_by_id_query/mod.rs:19`, `select_user_by_username_query/mod.rs:20`, `select_all_users_query/mod.rs:19-20` (all rows collected into one `Result<Vec<User>, TryFromUserRowError>`), `select_users_by_ids_query/mod.rs:33-34` (`Box::new(err).into()` → `BoxedError`), `insert_user/mod.rs:44`, `update_user/mod.rs:29`, `delete_user_by_id_query/mod.rs:18`.
- **Where the text becomes a customer-visible string:** `oxidauth-kernel/src/error.rs:32-45` — `display: format!("{}", self)` (`:36`) and `source: self.source().map(|s| s.to_string())` (`:40-42`), both always serialized; `debug: format!("{:?}", self)` (`:37`). API handlers map service errors with `Response::bad_request().error(err.into_error())`, e.g. `oxidauth-api/src/server/api/v1/users/list_all_users.rs:59`, `find_user_by_username.rs:64`, `auth/authenticate.rs:42`.
- **Why a bad `status` is storable at all:** `oxidauth-postgres/migrations/20221019185709_create_users.sql:4` — `status VARCHAR(32) NOT NULL`, no CHECK constraint and no Postgres enum type (indexed at `:17`).

## Problem

`ParseUserStatusErr` is the error type for parsing a **user status** string (`enabled|invited|disabled`), yet its `Display` prints `failed to parse user_kind, unknown: <input>` — text that names a different field, a different type, and a different vocabulary (`human|api`). Line `:146` is byte-identical to line `:73`; the only difference between the two impls is that one of them lies.

So when the repository converts a `users` row whose `status` column holds an out-of-vocabulary value, the operator-facing message is:

```
failed to convert UserRow to User: failed to parse user_kind, unknown: paused
```

`status: paused` is not a `kind` problem, and `paused` is not something `UserKind::FromStr` was ever asked about. The label is wrong for **every** input, always — there is no reading in which the message is merely imprecise.

## Analysis

**Copy-paste origin is unambiguous.** The two error types are clones of each other down to the field name (`unknown: String`), the two `impl fmt::Display` blocks sit ~70 lines apart with identical bodies, and both `std::error::Error` impls are empty. Changing one word (`user_kind` → `user_status`) is the entire difference between correct and incorrect, which is exactly the edit that got lost when the status impl was cloned from the kind impl.

**Which one is wrong is provable, not a judgment call.** `ParseUserStatusErr` is constructed at exactly one site, `:112`, inside `UserStatus::FromStr`; `ParseUserKindErr` at exactly one site, `:56`, inside `UserKind::FromStr`. Neither type is constructible from outside the module (`unknown` is private; the only public construction is `str::parse::<UserStatus>()`). Therefore any string printed by `:146` is, by construction, a rejected *status* token.

**The wrong label survives all the way into the wire body, while the field that tells the truth is buried.** Chain for a poisoned row:

1. `row.status.parse()?` (`oxidauth-postgres/src/users/mod.rs:50`) → `ParseUserStatusErr { unknown }`.
2. `From` (`:94-98`) → `TryFromUserRowError::ParseUserStatusErr(..)` — the enum variant is the *only* component that names the truth.
3. `Display` (`:71-72`) inlines the inner Display → top-level string names `user_kind`.
4. `source()` (`:83`) hands back the same `ParseUserStatusErr`, whose Display is the same lie → envelope `source` repeats "user_kind".
5. `into_error()` (`error.rs:32-45`) puts step 3 in `display`, step 4 in `source`, and the derived `Debug` (`ParseUserStatusErr(ParseUserStatusErr { unknown: "paused" })`) only in `debug`.
6. Handler emits `400 Bad Request` with that body (`list_all_users.rs:59` et al.).

Net effect: the two human-readable fields an operator actually reads both misdirect, and the one field that is correct (`debug`) is the one people grep last. (That `display`/`source` are both Display while `debug` is Debug is register line SRV-11's subject — un-ticketed as of writing; this ticket does not depend on its outcome and does not re-ticket it.)

**Blast radius when triggered.** `select_all_users_query/mod.rs:19-20` collects *all* rows into a single `Result`, so one poisoned row fails the entire list endpoint, not just that record — and the diagnostic sent to whoever is paging is an instruction to look at the `kind` column of a table where `kind` is fine. `find_user_by_username` failure additionally breaks login for that user (`auth/authenticate.rs:42` surfaces the same body). Time-to-diagnose is the whole cost here; nothing computes on the string.

**How a bad status actually gets into the column (why P3 is right).** In-application writes cannot produce one: the `&UserStatus → &'static str` conversion (`:93-101`) can only emit `enabled|invited|disabled`, and the JSON write path never reaches `FromStr` — the update DTO carries `Option<UserStatus>` deserialized by serde with `rename_all = "snake_case"` (`oxidauth-kernel/src/users/update_user.rs:26`, applied at `oxidauth-api/src/server/api/v1/users/update_user.rs:48`), so a client-supplied garbage status produces serde's own error text, not this Display. That leaves out-of-band writers: manual `UPDATE`/`INSERT` during ops fixes, bulk imports, and raw-SQL seeders — `seedz/src/fixtures.rs:342-343` inserts `'enabled'` literally, in-vocabulary today, but the pattern is unchecked (same for `oxidauth-postgres/src/users/insert_user/insert_user.sql`, which takes a bound value). With `status VARCHAR(32) NOT NULL` and no CHECK constraint, anything ≤32 chars is storable. A rolling deploy that adds a fourth `UserStatus` variant is a third producer `[INFERENCE — no such variant planned in-repo]`: old binaries reading rows written by the new binary would hit this exact message. No realistic input makes the *code* wrong-lier more often than "rarely", so P3 (diagnostic-only, exotic trigger, no security/data exposure) is proportionate.

**Adjacent drift in the same file, same family, deliberately small.** `UserStatus::FromStr` matches **raw literals** (`:108-110`) even though `ENABLED`/`INVITED`/`DISABLED` exist two lines above the type (`:89-91`), while `UserKind::FromStr` matches the consts (`:53-54`). The duplication is what makes "the two impls differ by one word" hard to spot on review. Fixing it is a no-op behaviorally (the consts hold the identical strings), so it can ride along. By contrast, unifying this module's typed-error convention with `AuthorityStatus`'s convention (`oxidauth-kernel/src/authorities/mod.rs:85-95`: `type Err = BoxedError`, message `invalid authority status: {}`) is a real refactor across three conventions and is **out of scope for a P3**.

**No contract depends on the wrong string.** Repo-wide search for `failed to parse user_kind` / `failed to convert UserRow` finds hits only at the two definitions (`:73`, `:146`), the two kernel test assertions (`:218`, `:268`), the composite Display definition (`oxidauth-postgres/src/users/mod.rs:75`), and the register/audit notes (`BUGS_AND_NOTES.md:77`, `missing-tests.md:301`). No `oxidauth-rs` client code or test, no `.hurl`/`.http` fixture, no JSON fixture, and no postgres-crate test asserts this text — `TryFromUserRowError`'s composite message is entirely unpinned. Both types are `pub` and re-exported (`oxidauth-postgres/src/users/mod.rs:4`), but the only consumers in this repo are the kernel's own tests and the postgres conversion above.

## Impact

Affected: whoever debugs a poisoned `users.status` row — on-call/SRE reading a `400` envelope body or a service log, and support engineers relaying it. The message sends them to the wrong column, wrong type, and wrong value set (`human|api`), so the wasted time is a full detour rather than a typo. Secondary: `GET`-style list endpoints fail wholesale for one bad row (`select_all_users_query/mod.rs:19-20`), and the affected user's login path (`find_user_by_username`) reports the same misdirection.

Not affected: no security exposure, no data corruption, no behavior change for valid data, no client-visible API change (only an error string's wording, asserted nowhere). Severity stays **P3** — the fix is one word plus its pin.

## Proposed resolution

**1. The fix (one word), `oxidauth-kernel/src/users/mod.rs:146`:**

```rust
impl fmt::Display for ParseUserStatusErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse user_status, unknown: {}", self.unknown)
    }
}
```

Keep the sibling's shape verbatim — `failed to parse <field>, unknown: <input>` — and use the token `user_status`, not `status`: it is greppable next to its sibling, matches the `users.status` column and the `User::status` field name, and avoids colliding with `AuthorityStatus`'s unrelated `"invalid authority status: …"` text (`authorities/mod.rs:92`). Do **not** touch `:73` — its pin (`:216-219`) stays green and churn there buys nothing.

**2. Pin flip (the only test change), `oxidauth-kernel/src/users/mod.rs:260-273`:**

- Delete the `BUG(pinned)` marker `:264-265`.
- Assertion `:266-269` → `assert_eq!(err.to_string(), "failed to parse user_status, unknown: paused");`.
- The rest of the test (`:271-272` case/empty rejection) stays as-is; its name already describes correct behavior. No new test needed: the exact-string assertion plus `user_kind_rejects_unknown_strings` (`:216-219`) already makes a future re-paste fail in both directions.

**3. Optional companion, same commit (recommended, zero behavior change):** make `UserStatus::FromStr` match the consts — `"enabled" => …` → `ENABLED => …` etc. (`:108-110` vs consts `:89-91`) — mirroring `UserKind::FromStr` (`:53-54`). Skip if the reviewer wants the diff to be strictly one word.

**4. Explicitly out of scope (do not bundle):** rejecting out-of-vocabulary `status` at *write* time (CHECK constraint / domain-error mapping for raw DB constraint text). That is a schema-migration and error-taxonomy change; it shares motivation with register line SRV-14 (raw DB error text surfacing unmapped) and belongs in its own ticket. Likewise: collapsing the three stringly-typed status/strategy parse-error conventions across the kernel into one shared helper — a real refactor, not a P3 diagnostic fix.

**Compat:** the message is not asserted by any client, fixture, or runbook in this repo (see Analysis), so no caller updates are required. If an external alert rule greps `failed to parse user_kind` for status failures, it must be repointed at `failed to parse user_status` — nothing in-repo does. `unknown` stays private; no signature or `From`/`Error` impl changes; `oxidauth-postgres` needs no edit.

## Verification

Commands for the implementer (this ticket was written from static reading — per assignment, no builds or tests were executed here).

- **Failing-before / passing-after:** `cargo test -p oxidauth-kernel users::tests::user_status_rejects_unknown_strings` — before the fix, after step 2 is applied, this fails with the two strings differing only in the `kind`/`status` token; after step 1 it passes.
- **Module suite:** `cargo test -p oxidauth-kernel users::tests` — 5 tests (`user_kind_round_trips_through_str_and_serde`, `user_kind_rejects_unknown_strings`, `user_status_round_trips_through_str_and_serde`, `user_status_rejects_unknown_strings`, `username_display_and_from_str_round_trip`); only the flipped assertion changes, the other four stay green (they never assert this text — see Locations).
- **Composite/envelope path (throwaway, no DB):** a scratch test asserting the operator-facing string; both types are publicly reachable (`pub mod users;` at `oxidauth-postgres/src/lib.rs:34`, re-export at `oxidauth-postgres/src/users/mod.rs:4`) and the `From` (`:94-98`) is infallible:
  `let err = "paused".parse::<oxidauth_kernel::users::UserStatus>().unwrap_err();`
  `assert_eq!(oxidauth_postgres::users::TryFromUserRowError::from(err).to_string(), "failed to convert UserRow to User: failed to parse user_status, unknown: paused");`
  and the same value through `BoxedError::into_error()` (`oxidauth-kernel/src/error.rs:32-45`) to confirm `display` **and** `source` both say `user_status` while `debug` still contains `ParseUserStatusErr`. Delete the scratch test afterward; the permanent coverage is the flipped kernel assertion.
- **Text-hygiene grep:** `grep -rn "failed to parse user_kind" src` must return exactly two hits after the change — `oxidauth-kernel/src/users/mod.rs:73` (definition) and `:218` (the user_kind pin) — and zero hits at `:146`/`:268`.
- **End-to-end, needs Postgres `[recipe not executed here]`:** start the database, `UPDATE users SET status = 'paused' WHERE username = '<seed user>'`, then `GET /api/v1/users` (or `find_user_by_username`) — expect `400` with `$.errors[0].display` = `failed to convert UserRow to User: failed to parse user_status, unknown: paused` and the same text in `$.errors[0].source`; restore the row to `'enabled'` afterward.

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- Reviewed jointly (scout re-verification + owner ruling 2026-09-29). All claims CONFIRMED against the current tree: defect at `oxidauth-kernel/src/users/mod.rs:146` byte-identical to the truthful sibling at `:73`; each error type has exactly one construction site (`:112`/`:56`), so the label is wrong by construction for every input; pin at `:268` (`"failed to parse user_kind, unknown: paused"`), marker `:264-265`; `users.status VARCHAR(32)` has no CHECK (`20221019185709_create_users.sql:4`); register `BUGS_AND_NOTES.md:77` accurate apart from the known marker-drift cite.
- Step 1 approved: `:146` → `failed to parse user_status, …`. Token `user_status` (greppable sibling, matches column + field, avoids colliding with `AuthorityStatus`'s `"invalid authority status"` text). `:73` stays untouched.
- Step 2 approved: delete marker `:264-265`, flip assertion `:268`. No new test — the symmetric `user_kind` pin (`:216-219`) makes a future re-paste fail in both directions.
- Step 3 companion ACCEPTED (not skipped): `UserStatus::FromStr` matches consts `ENABLED`/`INVITED`/`DISABLED` (`:108-110`), mirroring `UserKind::FromStr` — behavior-identical; the raw-literal duplication is exactly what hid the copy-paste.
- Step 4 boundaries held: no write-time CHECK (SRV-14 territory), no kernel error-taxonomy unification.
- Envelope nuance recorded: `into_error()` (`error.rs:32-45`) puts the wrong text in BOTH `display` and `source`; only `debug` is truthful. The composite `TryFromUserRowError` message is unpinned repo-wide, so exactly one assertion flips.
- Compat: zero in-repo consumers of the wrong string; external alert rules grepping `failed to parse user_kind` for status failures must repoint — changelog one-liner.
- Acceptance: `cargo test -p oxidauth-kernel users::tests` — flipped assertion fails-before/passes-after, other four module tests untouched-green; `grep -rn "failed to parse user_kind" src` → exactly two hits (`:73` definition, `:218` pin); throwaway envelope check per Verification, then delete.
