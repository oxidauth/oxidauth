- [90000004](https://www.pivotaltracker.com/story/show/90000004) - Provider wiring: xlib provider + provider/ split
    - oxidauth-http now builds its dependency container from the vendored
      xlib `provider::Provider` (new path dep; `oxidauth-kernel` remains for
      service traits + `BoxedError`); the 637-line `provider::setup()` is now
      the template triad `provider/{mod.rs, postgres.rs, services.rs}` with
      `init()` -> `postgres::init()` (connect, ping, migrate, store Database)
      -> `services::init()` (all service stores, original order preserved,
      region-grouped); the main bin calls `provider::init()`
    - handler/middleware service lookups use `provider.fetch_unchecked::<T>()`,
      panic-on-missing semantics identical to the old kernel `fetch`;
      `crate::provider::Provider` re-export keeps handler imports unchanged
    - BREAKING (oxidauth-usecases consumers): `SudoUserBootstrapUseCase::new`
      now takes `&provider::Provider` (xlib crate) instead of
      `oxidauth_kernel::provider::Provider`; bootstrap fetch behavior is
      unchanged (panics when a service is missing)
    - `oxidauth_kernel::provider::Provider` is marked
      `#[deprecated(since = "0.5.0", note = "use the provider crate (xlib)")]`;
      the kernel definition stays until the coordinated 2.0 of the published
      crates (plan 14)
