# 04 — Provider wiring: xlib provider + provider/ split

**Status**: `done` (2026-09-29: impl + review approve (0 must-fix); store parity 62=62 verified, live boot smoke: migrate+bootstrap+health_check 200+authenticate JWT via xlib provider)
**Depends on**: 03
**Risk**: high churn, mechanical. ~637-line `setup()` becomes three conventional
files; DI type swaps.

## Goal

Server-side DI moves from `oxidauth_kernel::provider::Provider` to
`provider::Provider` (xlib), and the monolithic
`src/oxidauth/oxidauth-http/src/provider/mod.rs` (`setup()`, 60
`provider.store` calls) becomes the template triad
`provider/{mod.rs, postgres.rs, services.rs}`.

Template contract (`/tmp/migcmp/tmpdemo/src/tmpstack/tmpstack-api/src/provider/mod.rs`):
```rust
pub async fn init() -> Result<Provider, BoxedError> {
    let mut provider = Provider::new();
    postgres::init(&mut provider).await?;
    services::init(&mut provider).await?;
    Ok(provider)
}
```

## Changes

1. **Dep swap** in `oxidauth-http/Cargo.toml`: add
   `provider = { version = "0.1.0", path = "../../xlib/provider" }`; remove
   any reliance on `oxidauth_kernel::provider`.
2. **Import sweep** across the server crate only:
   `oxidauth_kernel::provider::Provider` → `provider::Provider`.
   (`oxidauth_kernel::error::BoxedError` import stays; it's just re-exported —
   template kernels define `BoxedError` identically:
   `pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;`)
   Command: `grep -rl 'oxidauth_kernel::provider' src/oxidauth/oxidauth-http | xargs perl -pi -e 's/oxidauth_kernel::provider::Provider/provider::Provider/g'`
   plus fixing `use` lines. Note xlib Provider is `#[derive(Clone)]` — the
   server's `Server { provider }`/`.with_state(provider)` already fit.
3. **Split `provider/mod.rs`** — partition the existing `setup()` body:
   - `provider/postgres.rs`: connect `oxidauth_postgres::Database` +
     `db.ping()` + `db.migrate()` + `provider.store(db.clone())`. (Exact
     Database construction modernized in plan 05; keep current
     `Database::from_env()` for now.)
   - `provider/services.rs`: ALL remaining `provider.store::<XService>(...)`
     lines (usecases wiring, key material, CanLayer/permission bits, env read
     of `OXIDAUTH_USERNAME_PASSWORD_PEPPER`). Group with `// region users`,
     `// region auth`, … comments mirroring the `server/api/v1` domain modules
     so 60 stores stay scannable.
   - `provider/mod.rs`: `pub mod postgres; pub mod services;` + `init()` as in
     the template snippet; keep `pub use provider::Provider;` re-export so
     handler imports (`use crate::provider::Provider;`) need no edit.
   - Call-site rename: `provider::setup()` → `provider::init()` (main.rs).
   - Behavior-preserving: no reordering of stores; Provider is a type map —
     order only matters for `take()`, which nothing server-side uses yet.
4. **Kernel deprecation, not deletion**: `oxidauth-kernel/src/provider/` stays
   (published consumers + `oxidauth-seed`/tests may import it). Add
   `#[deprecated(since = "0.5.0", note = "use the provider crate (xlib)")]`
   on the kernel `Provider` type. Removal is gated on the coordinated 2.0 of
   published crates (tracked in plan 14 notes).
5. `#[async_trait]`-free: Provider is sync; no signature changes elsewhere.

## Verification

- `cargo check -p oxidauth-http` green; `grep -c 'provider.store'
  src/oxidauth/oxidauth-http/src/provider/services.rs` equals the old count
  (60 ± bootstrap entries) — no service lost in the move.
- Boot smoke: `DATABASE_URL=*** MIGRATIONS_ENABLED=true cargo run --bin
  oxidauth-http` reaches `"http booting..."` and serves
  `GET /api/v1/__meta/health_check` → 200 (health handler still fetches
  `provider.fetch::<Database>()` — now the xlib Provider; same stored type).
- Existing hurl suite `bin/hurl-tests.sh` green (no wire changes).
- Deprecation warning visible building kernel tests only, not in server code
  (`cargo check 2>&1 | grep -c 'use of deprecated'` server crates = 0).

## PR note

changelog `<id>-provider-xlib-split`; Status → `done`.

## Execution note (2026-09-29, worker)

- **Store-count check**: original `setup()` — `grep -c 'provider.store'` = **60**
  lines (incl. the `store::<Database>` bootstrap entry) plus **2** wrapped
  `provider`↵`.store::<..>` continuations = **62 store calls** total. After:
  `postgres.rs` = 1, `services.rs` = **59** lines + the same 2 wrapped =
  **61 service stores**. 62 before / 62 after, order preserved — the moved
  body diffs **byte-identical** against `git show HEAD` `setup()` lines 23–634
  (region comments the only additions). The pre-existing duplicate
  `CreateRolePermissionGrantService` block is preserved verbatim.
- **Files changed**: `oxidauth-http/Cargo.toml` (+provider path dep);
  `oxidauth-http/src/provider/{mod.rs (rewritten), postgres.rs (new),
  services.rs (new)}`; `main.rs` (`setup()`→`init()`); import sweep
  `server/api/v1/{meta/{health,live,mod},invitations/mod}.rs` →
  `use provider::Provider;`; AST codemod `.fetch::<T>()` →
  `.fetch_unchecked::<T>()` at 61 sites under `server/` + `middleware/`;
  `oxidauth-kernel/src/provider/mod.rs` (`#[deprecated]` on the struct);
  `oxidauth-usecases/{Cargo.toml, src/bootstrap/mod.rs}` (deviation below);
  `changelogs/90000004-provider-xlib-split.md`.
- **Deviation (approved by orchestrator)**: `oxidauth-usecases` bootstrap
  migrated from `oxidauth_kernel::provider::Provider` to xlib
  `provider::Provider` + `fetch_unchecked()` (1 import, 16 call sites, +1 path
  dep). `main.rs:27` passes the same provider instance that `init()` builds to
  `SudoUserBootstrapUseCase::new(&provider)`, so keeping usecases on the
  kernel type makes the boot smoke uncompilable. Panic-on-missing semantics
  unchanged; this is also the plan-09 end state. Flagged BREAKING in the
  changelog. Post-sweep `grep -rn 'oxidauth_kernel::provider' src/oxidauth/oxidauth-{http,usecases}`
  → zero. cli/import-export/seedz/oxidauth-rs never referenced kernel
  Provider, so nothing else could warn server-side.
- **Deviation**: `postgres.rs` adds `db.ping()` (old `setup()` had none — the
  plan's triad spec and the template both include it; behaviorally fail-fast-
  equivalent since `migrate()` connects anyway) plus the template's three
  `info!` lines. `Database::from_env()` kept per plan (modernization is plan 05).
- **Non-event**: plan item 3's `OXIDAUTH_USERNAME_PASSWORD_PEPPER` / CanLayer
  key-material bits do not exist in today's `setup()` — nothing to move;
  services.rs carries exactly the 61 stores that exist.
- No cargo/fmt/clippy/test runs (orchestrator gates, incl. DB boot smoke).

## Review feedback (2026-09-29, reviewer)

### (a) Store parity — verified (stronger than the count check)
- Ordered stored-type lists extracted from `git show HEAD` `provider/mod.rs`
  (62 entries) vs `postgres.rs` + `services.rs` (62): `diff` **identical** —
  every type present exactly once (minus the known dup), same order. Worker's
  62=62 claim TRUE.
- Whitespace/comment-normalized diff of the whole moved body: **byte-identical**
  against HEAD `setup()`. Additions are only: region comments, the
  `init(&mut Provider)` preamble, and `let db = provider.fetch::<Database>()?
  .clone();` atop `services.rs` (services no longer constructs `db`;
  `mod.rs::init()` runs `postgres::init` first so the fetch always hits, and
  `?` returns `Err` — gentler than a panic, and `Database` is `#[derive
  (Clone)]` pool-handle so store-by-value + fetch-clone == old store-clone +
  local-use).
  - nit: Execution note's "region comments the only additions" understates
    that `let db = …fetch…` line.
- Duplicate `CreateRolePermissionGrantService`: present **twice, verbatim, in
  both HEAD and the split** (two byte-identical blocks). xlib `store` is
  `HashMap::insert` → duplicate store **silently overwrites**, exactly like the
  kernel container: last-wins before and after, first block dead work before
  and after. Parity of behavior holds; not a new footgun. Pre-existing
  duplicate → recommend deleting one block in a later housekeeping plan, not
  here.

### (b) fetch semantics — verified
- Kernel `fetch` returned `&T` and panicked on missing (`"error getting {}
  from provider"`), so a Result-handling kernel call site was impossible by
  construction; xlib `fetch_unchecked` = `fetch().unwrap_or_else(|e| panic!)`
  → same panic-on-missing.
- Sampled 12 sites (health, live, register, authenticate, oauth2
  redirect/callback, forgot/update_password, save/fetch_setting, create_role,
  find_role_by_id) — all plain `let svc = provider.fetch_unchecked::<T>();`
  bindings; repo grep proves **zero** `?`/`match`/`if` guards around any
  `fetch_unchecked`. Grep count 61 under `server/`+`middleware/` matches the
  worker claim. No Result-handling site was converted to unchecked-panic.
- nit: panic text differs — old `"error getting <T> from provider"` vs
  `"provider error: ProviderError { type_name: \"<T>\" }"` (xlib panics with
  `Debug`; its Display `"<T> not found"` is unused on that path). Cosmetic
  unless logs grep panic text; owned by the vendored template file.

### (c) usecases bootstrap deviation (approved) — verified
- All 16 `fetch()` → `fetch_unchecked()` sites were plain bindings of the
  panicking kernel `fetch` — panic-semantics preserved, none Result-handling.
- `provider` path dep added to `oxidauth-usecases/Cargo.toml` (+ Cargo.lock);
  changelog flags **BREAKING** for `SudoUserBootstrapUseCase::new` consumers. ✓
- The orchestrator's live boot smoke ran bootstrap against the `init()`
  provider, exercising all 16 lookups — runtime proof every bootstrapped type
  is stored.

### (d) Orchestrator changes (post-worker; undocumented until now)
1. `server/api/v1/invitations/accept_invitation.rs`: the plan's sweep regex
   only matches the full path `oxidauth_kernel::provider::Provider`; this
   file's grouped `use oxidauth_kernel::{…, provider::Provider, …}` slipped
   it. Orchestrator split it into `use oxidauth_kernel::{…}` +
   `use provider::Provider;`. The plan's `grep -rn
   'oxidauth_kernel::provider'` → zero verification now holds.
2. `main.rs`: `PORT` env override (`std::env::var("PORT")`, default `"80"`,
   the container default compose maps) replacing the hardcoded
   `0.0.0.0:80` — parkinglot convention, required to run the host-boot smoke
   while host ports are taken. No behavior change when unset; a malformed
   `PORT` fails loudly at address parse.

### (e) Deprecation — verified
- `#[deprecated(since = "0.5.0", note = "use the provider crate (xlib)")]` on
  kernel `Provider` struct: present.
- Repo-wide grep: **zero** references to `oxidauth_kernel::provider` outside
  the kernel crate (seed/cli/import-export/oxidauth-rs never used it; no
  other kernel module references it) → `use of deprecated` warning count 0,
  server crates and workspace-wide. The plan's "warnings in kernel tests
  only" expectation is moot — there are none.

### (f) Assessment
- **Must-fix: 0. Should-fix: 0. Nits: 3** — (1) Execution-note "only
  additions" wording; (2) xlib panic Debug-format text; (3) local
  `crate::provider` module and extern `provider` crate share a name: root
  `provider::init()` lands on the module while submodules'
  `use provider::Provider;` and the `pub use provider::Provider;` re-export
  (keeping ~55 `crate::provider::Provider` handler imports compiling) resolve
  consistently to the same xlib type — workspace `cargo check` green confirms;
  template-intentional shape, flagged only for future maintainers.
- Pre-existing, out of patch scope: duplicate grant-service store (see (a)).

**Verdict**: `approve` — parity, fetch semantics, deviation scope, and
deprecation all verified; the two orchestrator follow-ups are recorded above.
Status may move `in-progress` → `done`.
