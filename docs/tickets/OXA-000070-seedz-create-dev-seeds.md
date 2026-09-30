# OXA-000070 — Build seedz dev seeding: land the fixture seeder per migration-plan 13

**Original ID:** plan-13 (seeding build step) · **Severity:** n/a · **Type:** task · **Status:** open
**Review tier:** Standalone plan slice (seedz dev seeding; not an easy-tier item) · unreviewed

**Context (2026-09-30)**

`docs/migration-plan/13-seedz-dev-seeding.md:3` reads `**Status**: done` — **the plan doc overstates reality**: at HEAD `src/seedz` is literally the template stub (`pub fn add(left, right)` + three `lib::tests`). HEAD's committed package is `oxidauth-seed` v0.4.0; the rename to `seedz` v0.9.0 exists only as uncommitted working-tree edits to `src/seedz/Cargo.toml` — decide the name as part of this ticket and make HEAD match. The plan's *spec* (fixture inventory, slot-derived `f0…` UUIDs, find-then-insert id-resolution, re-gate matrix) is sound and is the contract for this ticket; its status claim is not true of committed code.
**Input ruling:** rebuild from plan-13's inventory — the spec, not scratch files. Untracked working-tree `src/seedz/src/fixtures.rs` / `main.rs` (earlier seeding WIP, `??` in `git status`) are **reference-only**: they must NOT be committed as-is, and they are deleted when this ticket's implementation lands. Do not let a `git add -A` smuggle them in.

## Scope

1. **Implement seedz per plan-13:** fixtures (roles/users/permissions with slot-derived `f0…` UUIDs), find-then-insert natural-key id-resolution (reuse of pre-existing rows by `roles.name` / `users.username` / permission triple), `ENVIRONMENT=local` refusal guard, compose `seedz` profile with `service_healthy` gating, and the S13 fixture-graph validation tests (per-namespace duplicate-key asserts, username/email uniqueness, PK-level collision check).
2. **Intentional name-collision fixture** (absorbs OXA-000060's deferred record work): keep `ROLE.name == USERS[n].username` pairs (`seedz:viewer`, `seedz:editor`; auditor unpaired) mirroring bootstrap's `oxidauth:admin` role+user (`oxidauth-services/src/bootstrap/mod.rs:307`/`:437`); ship the pin test `role_names_deliberately_match_their_demo_usernames`, comments warning **rename both sides of a pair together** (slot-derived deterministic PKs make one-sided renames duplicate-key against every existing dev DB), and a module-docs line. Do NOT add cross-namespace uniqueness validation (OXA-000060 §Rejected: it would forbid the house pattern).
3. **Doc alignment (ownership split):** the N-10 rewrite of `BUGS_AND_NOTES.md:90` is owned by **OXA-000060** (its lore is false regardless of whether seeds ever ship) — at this ticket's landing, confirm that row's wording matches the shipped fixtures; do not double-edit. Correct `docs/migration-plan/13-seedz-dev-seeding.md:3` so `done` no longer implies the implementation is committed (point it at this ticket until the rebuild lands, then restore `done`).

## Constraints

- Grant plumbing stays id-only end to end; no by-name grant path may be introduced.
- The column-direct writer must route through validation or be listed as trusted-internal (see OXA-000041 step 3's note on seedz).
- All string lookups remain namespace-scoped (`roles.name`, `users.username` are per-table UNIQUE; `user_identifier` values come from usernames only).

## Verification

- `cargo test -p seedz` green: pin test + S13 validation tests + lib tests.
- Plan-13 re-gate matrix re-run, including the hand-inserted "collision DB" scenario (pre-existing rows reused by natural key, grants point at live ids).
- hurl suite green against a seedz-seeded stack; the `oxidauth:admin` role/user both-namespaces probe (`hurl/tests/roles.hurl`, `hurl/tests/authenticate.hurl`) must stay green on the same DB.
- `grep -rn 'BUG(pinned)' src/seedz` empty; `git status src/seedz` clean — implementation committed, untracked scratch fixtures gone, plan-13 status honest.
