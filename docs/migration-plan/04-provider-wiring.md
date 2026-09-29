# 04 — Provider wiring: xlib provider + provider/ split

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — kernel Provider deprecated-kept until 2.0)
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
