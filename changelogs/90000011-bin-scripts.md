- [90000011](https://www.pivotaltracker.com/story/show/90000011) - bin/
  discovery-script system + hurl suite rework (migration plan 11).
    - **Deterministic local auth**: `.env` gains
      `OXIDAUTH_DEFAULT_CLIENT_KEY` (UUID) +
      `OXIDAUTH_DEFAULT_ADMIN_PASSWORD`, the env pins the bootstrap seeds
      (authority client_key + `oxidauth:admin` password; random when unset).
      `example.env` documents them as commented-out template lines. With them
      set, a fresh `bin/reset-db.sh` + `docker compose up -d` boots a stack
      the hurl suite can log into without scraping the boot log.
    - Root `bin/` now follows the template: `build.sh` (finds `build.sh` under
      `./src`; REGISTRY `registry.vizerapp.cloud/oxidauth`, rust `v1.89.0`,
      debian `12.12` — finds nothing until plan 12 ships
      `oxidauth-api/build/build.sh`), `unit_test.sh`
      (`cargo test --workspace --exclude *-postgres --exclude postgres` =
      zero-database), `database_test.sh` / `hurl.sh` (find-exec loops →
      `src/oxidauth/*.sh`), plus parkinglot `version.sh`
      (`cargo set-version --workspace` wrapper: show / --dry-run / apply —
      `bin/version.sh show` prints today's divergent set) and
      `crate_version.sh` (one crate's version via `cargo metadata`).
    - Deleted `bin/hurl-tests.sh` (→ the hurl.sh pair),
      `bin/build-server.sh` (cross-build + `docker buildx` multi-arch logic
      moves to plan 12; its `oxidauth-builder` builder-name pattern is
      preserved in the plan-11 Execution note) and `bin/cargo-watch.sh`
      (the dev loop is compose watchexec now). `README.md` example updated;
      the repo has no `.github/workflows`, so nothing referenced the removed
      names.
    - `bin/publish.sh` republished for the new tree: topological order
      permission → kernel → repository → postgres → services → http → rs →
      api (`oxidauth-rs` publishes as crate `oxidauth`; `permission` is the
      only root — the kernel path-depends on it), versions read via
      `bin/crate_version.sh`, and a final semver git tag
      `v<oxidauth-api version>` — gated on a **clean working tree** (cargo
      publishes the tree; a dirty run could not honestly tag HEAD).
    - `src/oxidauth/hurl.sh` replaces `bin/hurl-tests.sh`: resolves the
      target dynamically — `OXIDAUTH_HURL_HOST`/`_PORT`/`_SCHEME` override,
      else `docker compose port oxidauth-api 80` (the stack publishes on an
      **ephemeral** host port) — injects `host`/`scheme`/`stamp`/`api_version`
      plus the login pair `admin_client_key`/`admin_password` (read from the
      gitignored `.env`, never committed to `hurl/variables-local`) over the
      variables file, runs `public_keys_create.hurl` then the suite twice
      with one shared `stamp` so pass 2 only passes if pass 1's deletes
      stuck (the old cleanup-verification trick).
      `src/oxidauth/database_test.sh` sources `.env`, sets
      `MIGRATIONS_ENABLED=true` and runs `cargo test -p oxidauth-postgres
      -p postgres -- --nocapture`.
    - The hurl suite (17 files) was rewritten whole against the post-plan-07
      contract: every guarded call sends `Authorization: Bearer {{jwt}}`
      from a per-file `POST /auth/authenticate` with the pinned credentials
      (the old files were pre-auth and used the removed `strategy` register
      body); statuses match the handlers, not wishes — bare **401** from the
      jwt extractor (no/foreign token), **400** envelope +
      `RowNotFound`/`PermissionNotFoundError`/`SettingNotFoundError`/
      `Strategy(Oauth2)`/`duplicate key` debug strings for service errors,
      **422** for deserializer rejections (missing `jwt_nbf_offset`, bad
      uuid), `count == 0` list assertions replaced with seeded-state
      assertions, grant payloads pinned to the `{permission|role|child,
      grant}` / `{user_role|user_permission}` shapes, and the authority
      client_key **regeneration on update** asserted as-is. New
      `tests/healthcheck.hurl` + `tests/livecheck.hurl` assert the plan-07
      `{success, payload:{version, healthy}}` envelope with `version`
      cross-checked against `bin/crate_version.sh oxidauth-api` (placed in
      `tests/` so the `hurl/tests/*.hurl` glob actually runs them).
      `invitations`/`totp`/`oauth2` stay uncovered, as before — no
      honest-assertable flow existed for them pre-migration either.
