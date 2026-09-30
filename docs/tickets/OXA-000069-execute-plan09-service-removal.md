# OXA-000069 — Execute plan-09: migrate every named site off `Service`, then remove the trait

**Original ID:** plan-09 (removal step) · **Severity:** n/a · **Type:** architecture · **Status:** done
**Review tier:** Standalone plan slice (Service removal; not an easy-tier item) · SCHEDULED — sequence before 23/25/20/42/50/9


## Decision (2026-09-29)
- **Review tier: standalone plan slice (born from OXA-000067's Tier-1 review 2026-09-29; not an "easy" tier item).**

Owner approved the removal direction proposed during OXA-000067 review: **remove the deprecated `Service` trait (and `CanLayer`/`CanService`) after migrating all consumers to the named `*Query`/`*ServiceTrait` machinery.** "All of these should be migrated before removing" — and the compiler enforces exactly that: the trait is `pub`, so any un-migrated site stops compiling at removal. This ticket supersedes OXA-000067's T-7 rewrite (on removal, the T-7 register line is struck, not corrected).

**Key enabling fact: the deprecation was never committed.** `git log -S "generic request dispatch is retired"` → 0 commits; HEAD `oxidauth-kernel/src/service.rs` is attribute-free. The three `#[deprecated]` attributes exist only in the working tree, so the migration can land without the deprecation cycle ever shipping — the "removed for 2.0" promise is kept by never shipping the deprecation.

## Scope inventory (verified 2026-09-29, grep counts + OXA-000067 audit)

- `oxidauth-kernel/src/service.rs` — `Service` (trait + `call`), `CanLayer`, `CanService`, self-uses `:32-:80`, test mod `:104-161` → **deleted entirely**; plus 37 request-module re-exports `pub use crate::service::Service;` (30 standalone + 7 grouped, e.g. `users/create_user.rs:10`, `roles/create_role.rs:7`).
- `oxidauth-repository` — 60 × `pub trait XQuery: for<'a> Service<&'a Params, Response = R, Error = BoxedError> {}` + blanket `impl<T: Service<…>> XQuery for T` (shape verified at `roles/insert_role.rs:6-14`); 116 `Service<` trait-refs, 32 import files; **zero** allows, zero test mods. The blankets are the bridge: removing them is what forces every impl to name its trait.
- `oxidauth-postgres` — 58 `impl Service<&'a Params> for Pg*Repository` headers (+7 named imports) rewritten to `impl XQuery for Pg*Repository`; 46 `#[allow(deprecated)]` (all in `mod tests`) deleted. Bodies unchanged.
- `oxidauth-services` — 105 hand-written `impl Service<…> for Mock*` rewritten to the named traits; 60 allows deleted; use-case call sites `.call(&params)` → named method calls.
- `oxidauth-api` — `middleware/can.rs`: the one non-test `Service` impl + `MockService` test + `CanLayer` composition rebuilt on the stateless-permission invariant (OXA-000056 proved `CanService::call()` is a single `validate()`, zero state — it re-builds as a direct check/extractor, no composition machinery). 2 allows deleted. Highest-blast-radius file: every authenticated request passes it; hurl e2e is the gate.
- `oxidauth-rs`, `seedz`, `xlib`, cli, import-export, http — **zero** `Service` references; untouched.

## Design work (the only two non-mechanical decisions)

1. **60 method signatures.** `XQuery` traits own no methods today (they borrow `call`). Each named trait gets a real async method (`InsertRoleQuery::insert_role(&self, params: &CreateRole) -> Result<Role, BoxedError>`); names follow the trait (boring, but 60 decisions). Recreating a generic base trait under a new name is explicitly rejected — that is `Service` in a disguise.
2. **can.rs middleware rebuild** (above). Preserve the `/can` endpoint and extractor behavior verbatim; same call graph, named edges.

## Slice ordering (each step ends compile-green)

1. Audit: every `Service<&Params>` site has a matching `*Query`/`*ServiceTrait` + chosen method name (expect 1:1 across the 60 traits; report gaps before proceeding).
2. Give the 60 `XQuery` traits real methods; keep blanket `impl<T: Service<…>> XQuery for T` forwarding temporarily so everything still compiles.
3. Rewrite 58 postgres impls + 105 services mocks to the named traits; migrate use-case/API call sites to named methods; delete the blanket bridge in the same commit per entity as it empties.
4. Rebuild `can.rs` on the direct `validate()` check; delete `CanLayer`/`CanService` uses.
5. Delete `oxidauth-kernel/src/service.rs` entirely + 37 kernel re-exports + all 108 `#[allow(deprecated)]` (60 services / 46 postgres / 2 api).
6. Register/docs: strike T-7 from `BUGS_AND_NOTES.md` (per OXA-000067's supersede note); update `missing-tests.md` allow-inventory refs (`:353`, `:438`, `:637`); plan-09 changelog entry under `changelogs/`.

## Verification gates

- After steps 2–4: `cargo test --no-run --workspace` green while `Service` still exists (forwarding intact) — migration is behavior-neutral.
- At step 5: removal commit compiles ⇒ migration completeness proven by the compiler (no `pub Service` left to name).
- `cargo build -p oxidauth-repository -p oxidauth-postgres -p oxidauth-kernel 2>&1 | grep -c "use of deprecated"` → **0** (was ≈258; kills the desensitization window OXA-000067 accepted as interim cost).
- `bin/unit_test.sh`, `bin/database_test.sh`, hurl e2e suite green — behavior identical; can.rs is the file to watch (all authenticated routes).
- Test count unchanged (mocks rewritten in place; no tests added/removed).

## Coordination

- **Sequence BEFORE the service-touching MODERATE tickets** (OXA-000023, 25, 20, 42, 50, 9): they currently ripple type/mock changes through `Service` bounds; after removal they touch named traits only, once.
- Conflicts with anything rewriting services mocks or postgres impl headers while this slice is in flight — land it as one focused slice, not sprinkled commits.
- Semver: pre-1.0, in-tree-only consumers (publish.sh publishes `oxidauth-*`; nothing shipped ever deprecated `Service`, so no published contract is broken by removing an uncommitted-deprecated trait — the trait itself predates HEAD, so confirm the published kernel version before step 5 and bump accordingly).

## Status note

Scheduled. No implementation started. Execute as a dedicated plan slice after the Tier-1 docs batch.
