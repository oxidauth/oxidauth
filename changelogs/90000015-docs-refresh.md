- [90000015](https://www.pivotaltracker.com/story/show/90000015) - docs + README refresh (migration plan 15, final)
    - Root `README.md` rewritten to the project-template shape: 8-layer table
      with a this-stack column (rs exists as the published `oxidauth` crate;
      no web layer), getting started (`.env` + the two `OXIDAUTH_DEFAULT_*`
      bootstrap pins, compose up, healthcheck curl on the ephemeral
      `docker compose port` mapping, seedz, hurl, unit tests), Available
      Stacks + the link-stack-to-project checklist, Development (1.98.0
      toolchain pin, the `cargo +nightly-2025-07-01 fmt --all` convention,
      honest clippy deprecation note), the bin script table, and a 2.0
      follow-up section (deprecated kernel shims, xlib publish names, the
      wasm `Send` wall) each pointing at its plan execution note.
    - New `src/oxidauth/README.md`: stack README with the layer diagram
      verified against `cargo tree` edges (services does NOT depend on
      postgres; rs pulls services only for strategy param types), the api
      Provider-pattern walkthrough (named-alias wiring), crates table, and
      the eight-step adding-entities chain (kernel type → repository trait →
      Pg repository + `.sql` → services UseCase → provider block → api
      handler → `oxidauth-http` DTO → rs client method), plus the hurl
      suite pointer.
    - New `docs/CLIENT_MIGRATION.md` for 0.8→0.9 consumers (parkinglot):
      `server::api::v1::` prefix dropped from DTO paths, `response::Response`
      → root `Response`, `oxidauth-usecases` → `oxidauth-services` (client
      package stays `oxidauth`), generic `Service::call` → named methods,
      uniform 0.9.0 version floor.
    - `docs/SECURITY_REPORT.md` code paths refreshed to the 0.9 layout
      (findings/line numbers unchanged, provenance note added);
      `rfcs/4-forgot-password-flow/rfc.md` path references updated (paths
      only); `docs/OAUTH.md` example redirect URIs now name
      `api.oxidauth.localhost`; README gains a Security section linking the
      report. `docs/AUTHORITIES.md` and the other rfcs carry no stale paths.
    - `changelogs/README.md` now describes the migration arc and the
      `900000NN` ↔ plan `NN` mapping; `docs/migration-plan/README.md` gains
      the completion date and its pickup grouping note is now historical.
