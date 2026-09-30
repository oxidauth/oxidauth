# OXA-000051 — `PgUserAuthorityWithAuthority`'s hand-written `Debug` duplicates `authority_status`; omissions are deliberate redaction (document, don't derive)

**Original ID:** N-1 · **Severity:** n/a · **Type:** note · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #14 of 27 · reviewed 2026-09-29 · SCHEDULED


## Locations

All paths relative to `src/oxidauth/`. Verified against the working tree 2026-09-29.

- `oxidauth-postgres/src/user_authorities/mod.rs:64-80` — `struct PgUserAuthorityWithAuthority` (`#[derive(sqlx::FromRow)]`, module-private, 14 fields including `params: Value` at `:69` and `authority_params: Value` at `:77`).
- `oxidauth-postgres/src/user_authorities/mod.rs:82-100` — the hand-written `impl fmt::Debug`: `authority_status` printed twice (`:92` and `:95`); `params` and `authority_params` never printed; `authority_updated_at` **is** printed (`:97`).
- `oxidauth-postgres/src/user_authorities/mod.rs:29-49` — sibling `PgUserAuthority` row + its hand-written `Debug`, which likewise omits only `params` (`:34`) — clean omission, no duplicated field.
- Redaction doctrine this impl mirrors: `oxidauth-kernel/src/lib.rs:61-66` (`JsonValue`'s `Debug` prints zero fields), `:87-94` (`Password`'s `Debug` masks as `******`, comment: "`Debug` is what tracing spans record"), and `oxidauth-kernel/src/authorities/mod.rs:17-28` (`Authority` derives `Debug` with typed `settings: AuthoritySettings` visible but `params: JsonValue` blank).

Register drift note: the N-1 line says the impl "omits `authority_updated_at`/`authority_params`". Half of that is wrong — `authority_updated_at` is emitted at `:97`. The fields actually omitted are `params` (`:69`) **and** `authority_params` (`:77`), and the duplicate `authority_status` at `:95` sits exactly at the declaration-order position where `authority_params` would belong.

## Problem

The manual `Debug` impl for `PgUserAuthorityWithAuthority` lists 13 `.field` calls against a 14-field struct: it emits `authority_status` twice and never emits `params`/`authority_params`. As written, `format!("{row:?}")` renders a struct that is simultaneously wrong (duplicate key) and incomplete against the struct definition, with nothing in the file explaining which parts of that are intentional.

Provenance: `git blame` attributes the entire impl to `fa5e6f7` (2024-02-01, the file's original commit; `c9312e0` was the `src/` layout move only). The uncommitted working-tree diff touching these lines is pure rustfmt-style reflow (multi-line `.field(...)` calls collapsed to one line); the duplicate survives verbatim. So: pre-existing, as the register says.

## Analysis

**These are the only two hand-written `Debug` impls for row structs in `oxidauth-postgres`.** Every other Pg row struct — `PgAuthority`, `PgInvitation`, `PgPermission`, `PgPrivateKey`, `PgPublicKey`, `PgRefreshToken`, `PgRolePermissionGrant`, `PgRoleRoleGrant(Detail)`, `PgRole`, `PgSetting`, `PgTotpSecret`, `PgUserPermission(Grant)`, `PgUserRole(Grant)`, `UserRow` — uses `#[derive(Debug, sqlx::FromRow)]`. The user_authorities module is the sole exception, and the exception has a coherent rationale:

- The kernel enforces "params never appear in `Debug`" structurally: `JsonValue`'s hand-written `Debug` prints no fields (`oxidauth-kernel/src/lib.rs:61-66`), so derived `Debug` on `UserAuthority`/`Authority`/`UserAuthorityWithAuthority` shows typed `settings` but a blank `params`. Same doctrine as `Password`'s `******` mask (see OXA-000008, which also notes "`Debug` is what tracing spans record").
- The Pg row structs store the same JSON as **raw `serde_json::Value`**, where that structural redaction is lost. The hand-written impls reproduce kernel policy at the row layer: they omit exactly `params` and `authority_params` and print everything else — including `authority_settings`, matching the kernel's "settings visible / params redacted" split (`oxidauth-kernel/src/authorities/mod.rs:24-25`).
- The omission is not theoretical: `user_authorities.params` carries credential material. Tests seed argon2 password hashes into it — `user_authorities/select_user_authorities_by_authority_id_and_user_identifier/mod.rs:67`, `update_user_authority/mod.rs:49`, `insert_user_authority/mod.rs:64`.

The duplicate at `:95` is therefore the only genuine defect: a copy-paste slip in a hand-maintained list (its position is where `authority_params` would sit in declaration order — it must not be "fixed" by swapping in `authority_params`, which would break the redaction).

**Would `derive(Debug)` suffice?** It would compile — all 14 fields (`Uuid`, `String`, `serde_json::Value`, `DateTime<Utc>`) implement `Debug` — but it would be the wrong fix: derive would print `params`/`authority_params` verbatim, leaking argon2 hashes and authority-level params JSON into anything that formats the row (`tracing` spans, panics, `{:?}` in future code). Derive is exactly what the kernel's `Password` got, and OXA-000008 shows how that aged.

**Who consumes this `Debug` today?** No one observable. The type is module-private (imported via `super::` only by `select_user_authorities_by_user_id/` and `select_user_authority_by_user_id_and_authority_id/`); `sqlx::query_as` + `FromRow` don't require `Debug`; both `#[tracing::instrument]` sites `skip(self)` and record only the kernel request (`params`), never the fetched row; and the only `{found:?}` in the tree (`select_user_authorities_by_user_id/mod.rs:60`) formats the kernel `Vec<UserAuthorityWithAuthority>`, not the Pg row. No `BUG(pinned)` marker exists near either impl — the one in this directory (`select_user_authorities_by_authority_id_and_user_identifier/mod.rs:58`) pins the identifier-lookup semantics already ticketed as OXA-000025 and says nothing about `Debug`. So the register's "not test-pinned" is confirmed, and any change here is behavior-free today; it is a guard against the first future `{:?}`.

## Impact

None at runtime: no code path formats these types, so nothing currently logs or asserts on the duplicated/missing fields. The exposure is maintenance-grade — (1) the first developer to `{:?}` a row gets misleading output (two `authority_status` keys, no visible gap suggesting why `params` is absent); (2) the absence of a comment invites a future "simplification" to `derive(Debug, sqlx::FromRow)` to match the other 15 row structs, which would silently defeat the params redaction; (3) the register line itself propagated a wrong claim (`authority_updated_at` "omitted"), which this ticket corrects.

## Proposed resolution

**Disposition: FIX NOW (trivial), and KEEP the hand-written impls as intentional redaction.** Not derive; not leave-undocumented.

1. Delete `oxidauth-postgres/src/user_authorities/mod.rs:95` (the second `.field("authority_status", &self.authority_status)`). Do **not** replace it with `authority_params` — the omission is the point (see Analysis).
2. Add a one-line comment above each of the two `impl fmt::Debug` blocks (`:39`, `:82`) retiring the ambiguity, e.g.:

   > `// Hand-written (crate-wide unique): `params`/`authority_params` are raw serde_json::Value here, so they are deliberately omitted from Debug — mirrors kernel JsonValue's blank Debug (oxidauth-kernel/src/lib.rs:61-66); user_authorities.params carries credential material (argon2 hashes). Do not replace with derive(Debug).`

3. Retire N-1 from `BUGS_AND_NOTES.md` §5 with a one-line correction, since the register's "omits `authority_updated_at`" is false and the surviving half (params omission) is intended behavior: "`PgUserAuthorityWithAuthority` Debug: duplicated `authority_status` removed; omission of `params`/`authority_params` is intentional redaction, now commented (OXA-000051)."

## Verification

- `grep -c 'field("authority_status"' oxidauth-postgres/src/user_authorities/mod.rs` drops from 2 to 1; the impl lists each of the 12 retained fields exactly once and still omits `params`/`authority_params` only.
- Compile check suffices — no consumer exists (see Analysis), so no test can observe the change: `cargo check -p oxidauth-postgres`.
- Existing user_authorities tests still pass untouched (`select_user_authorities_by_user_id` asserts on kernel types only); no test asserts on `Debug` output of either Pg row struct — re-grep `{:?}` under `user_authorities/` after the change to confirm all matches format `sqlx::Error`/`BoxedError`/kernel types.
- Related: OXA-000008 (Password raw-secret vs Debug-only masking — same redaction doctrine), OXA-000025 (the `BUG(pinned)` test in this module directory; unaffected).

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- Reviewed jointly (scout re-verification + owner ruling 2026-09-29). Confirmed: `PgUserAuthorityWithAuthority`'s hand-written `Debug` (`oxidauth-postgres/src/user_authorities/mod.rs:82-100`) prints `authority_status` twice (`:92`, `:95`) and omits `params` (`:68`) + `authority_params` (`:77`); `authority_updated_at` IS printed (`:97`).
- **Register correction recorded:** N-1's claim that `authority_updated_at` is omitted is FALSE. The real omissions are the two JSON columns, and they are intentional: these are the crate's only two hand-written row-struct `Debug` impls because the Pg rows store `serde_json::Value`, where the kernel's structural redaction (`JsonValue` blank Debug, `lib.rs:61-66`; `Password` `******`, `:87-94`) doesn't apply. `user_authorities.params` demonstrably carries credential material (tests seed argon2 hashes: `select_user_authorities_by_authority_id_and_user_identifier/mod.rs:67`, `update_user_authority/mod.rs:49`, `insert_user_authority/mod.rs:64`).
- Step 1 approved: delete `:95`'s duplicate `.field("authority_status", …)`. Explicitly do NOT swap in `authority_params` — the omission is the point.
- Step 2 approved: one-line comment above both hand-written impls (`:39`, `:82`) pinning the redaction rationale + "do not replace with derive(Debug)" — derive would print raw params and silently defeat the policy (precedent: how `Password`-with-derive aged, OXA-000008).
- Step 3 approved: retire N-1 with a one-line correction (duplicate removed; params omission is intended redaction), not bare deletion — the register propagated the wrong half-claim and the correction is the artifact.
- Zero consumers verified (module-private; instrument sites `skip(self)`; the only `{found:?}` formats kernel types; no test asserts these Debug strings) → compile-check is the whole acceptance: `grep -c 'field("authority_status"'` 2→1, `cargo check -p oxidauth-postgres`, existing user_authorities tests untouched. OXA-000025's pin in this directory is unaffected.
