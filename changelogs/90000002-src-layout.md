- [90000002](https://www.pivotaltracker.com/story/show/90000002) - Move crates into src/ layout
    - `git mv`'d oxidauth-{kernel,repository,postgres,usecases,http,rs,permission,
      cli,import-export} into `src/oxidauth/` (crate names unchanged; stub crates
      oxidauth-cli + oxidauth-import-export kept per review)
    - `git mv`'d `oxidauth-http/hurl` -> `src/oxidauth/hurl` and
      `oxidauth-seed` -> `src/seedz`
    - `oxidauth-telemetry` stays at the repo root until plan 06 removes it; it is
      now an explicit workspace member since it cannot match the `src/oxidauth/*` glob
    - root Cargo.toml: members = `["src/oxidauth/*", "src/seedz",
      "oxidauth-telemetry"]`, exclude = `["src/oxidauth/hurl",
      "src/oxidauth/helm"]` (hurl has no manifest; cargo globs require one);
      `resolver = "2"` and the num-bigint-dig dev-profile opt kept
    - repointed oxidauth-http's `oxidauth-telemetry` path dep to
      `../../../oxidauth-telemetry` (telemetry stayed at root); all other
      `../oxidauth-*` sibling deps verified resolvable unchanged; `src/seedz`
      is a dependency-free stub needing no repointing
    - bin/hurl-tests.sh: `pushd oxidauth-http` -> `pushd src/oxidauth` (hurl
      globs and variables-file paths unchanged, semantics identical)
    - bin/reset-db.sh: `pushd oxidauth-postgres` ->
      `pushd src/oxidauth/oxidauth-postgres` (DATABASE_URL left hardcoded for now)
    - bin/publish.sh: crate folders -> `src/oxidauth/oxidauth-$PROJECT`
    - bin/build-server.sh: version probe and Dockerfile path ->
      `src/oxidauth/oxidauth-http/`
