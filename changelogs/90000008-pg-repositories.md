- [90000008](https://www.pivotaltracker.com/story/show/90000008) - postgres:
  per-entity `Pg<Entity>Repository` structs replace `impl Service<&X> for
  Database`
    - **BREAKING (internal API):** `oxidauth-postgres` no longer implements
      any query service on the `Database` type. `grep -rn 'impl.*for
      Database' src/oxidauth/oxidauth-postgres/src` → 0. `Database` keeps
      only its `database!`-macro surface (`from_env`, pools, `migrate`,
      `PingTrait`).
    - each of the 16 entity modules (`users`, `authorities`,
      `user_authorities`, `roles`, `role_role_grants`,
      `role_permission_grants`, `permissions`, `user_permission_grants`,
      `user_role_grants`, `refresh_tokens`, `public_keys`, `private_keys`,
      `totp_secrets`, `invitations`, `settings`, `auth`) now declares
      `pub struct Pg<Entity>Repository { db: Database }` + `new(db)`
      (`#[derive(Debug, Clone)]`, parkinglot/template idiom):
      `PgUserRepository`, `PgAuthorityRepository`,
      `PgUserAuthorityRepository`, `PgRoleRepository`,
      `PgRoleRoleGrantRepository`, `PgRolePermissionGrantRepository`,
      `PgPermissionRepository`, `PgUserPermissionGrantRepository`,
      `PgUserRoleGrantRepository`, `PgRefreshTokenRepository`,
      `PgPublicKeyRepository`, `PgPrivateKeyRepository`,
      `PgTotpSecretRepository`, `PgInvitationRepository`,
      `PgSettingRepository`, `PgAuthRepository`.
    - all 62 query modules (`<entity>/<query>/{mod.rs,query.sql}` layout
      unchanged, `.sql` includes untouched) re-point their impl receivers
      from `Database` to the entity struct: `impl Service<&X> for
      Database` → `impl Service<&X> for Pg<Entity>Repository` (and the
      method-form traits `InsertUserAuthorityQuery`, `UpdatePermission`,
      `InsertTotpSecretsQuery`,
      `SelectWhereNoTotpSecretByAuthorityIdQuery` →
      `impl <Trait> for Pg<Entity>Repository`). 62 old `Database` impls
      deleted == 62 new struct impls; the
      `oxidauth-repository` traits (blanket `…Query` markers over
      `Service<…>` plus the four method-form traits) are now implemented by
      the structs instead of `Database` and remain untouched. Free
      `async fn *_query(conn, …)` helpers (incl. the async-recursion
      permission tree in `auth/tree`) unchanged.
    - pool access: every `self.write_pool()` / `self.read_pool()` became
      `self.db.write_pool()` / `self.db.read_pool()`; the per-file
      write/read split from plan 05 is preserved exactly (mutation
      queries → write pool, select queries → read pool, audited
      file-by-file).
    - provider wiring (`oxidauth-api/src/provider/services.rs`): every
      use-case arg that was a raw `db.clone()` is now
      `Pg<Entity>Repository::new(db.clone())` mapped to the trait each
      parameter requires (`AuthenticateUseCase`'s 7, `RegisterUseCase`'s 6,
      `ExchangeRefreshTokenUseCase`'s 7, etc.). Store order and count
      unchanged: 59 `provider.store::<…>` lines before and after; 105
      `db.clone()` occurrences before and after, now all inside 105
      `Pg<Entity>Repository::new(...)` calls. The pre-existing duplicate
      `CreateRolePermissionGrantService` store block is kept as-is
      (provider overwrite semantics; cleanup is plan 09). Services-side
      consolidation (shared repo instances) is deferred to plan 09.
