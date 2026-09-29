# 12 — Helm chart + deploy pipeline

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — kube context / prod migration default / registry path resolved in-PR as flagged)
**Depends on**: 11
**Risk**: medium — infra-facing; validate with `helm template`, deploy behind
user go/no-go (never auto-deploy from this migration).

## Goal

Stack gains the deployable half of the template: `src/oxidauth/helm/` (from
stack-template, api half), `oxidauth-api/build/{Dockerfile,build.sh}`, and
`bin/{deploy,uninstall}.sh` wired through `devops/helm/values-*.yaml`.

## Changes

1. **Chart**: `cp -R <stack-template>/stack/helm src/oxidauth/helm` and
   substitute `{{project-name}}` → `oxidauth` (the template's own trick:
   `perl -pi -e 's/\{\{project-name\}\}/oxidauth/g'` on
   yaml/md/json/toml/txt — chart templates use Helm mustache so liquid must
   never touch them; `cargo-generate.toml` excludes `helm/**` for exactly this
   reason). Keep `_helpers.tpl` names (`oxidauth.name`, `oxidauth.fullname`,
   `oxidauth.api.labels`…).
2. **Web half disabled, not deleted**: keep `web-deployment.yaml`,
   `web-service.yaml`, `web-ingress.yaml` behind `values.yaml` flags
   (`web.enabled: false`) so a future admin-UI stack re-enables without
   chart surgery. `containerPort: 8080` default vs server bind `:80` — set
   `api.containerPort: 80` in `values.yaml` (template ships
   service 80→8080; either change values or make the server honor `PORT`;
   parkinglot's api listens on `PORT` env default 80 — **adopt PORT env
   support in `oxidauth-api/src/main.rs`** in this PR:
   `std::env::var("PORT").unwrap_or(80)`).
3. **Production image**:
   - `src/oxidauth/oxidauth-api/build/Dockerfile` — template/parkinglot
     multi-stage: `rust-base:$RUST_BASE_IMAGE_VERSION` builder →
     `debian:$DEBIAN_BASE_IMAGE_VERSION` runtime; `cargo build --release --bin
     oxidauth-api`; `COPY --from=builder` migrations dir alongside the binary
     (sqlx `migrate!` embeds, but parkinglot copies them — keep parity for
     sqlx-cli ops in-container); `EXPOSE 80`.
   - `build/build.sh`: `VERSION=$($PWD/../../../../bin/crate_version.sh
     oxidauth-api)`; `IMAGE=$REGISTRY/oxidauth-api:$VERSION` + `:latest`;
     `docker buildx create --use --name oxidauth-builder` +
     `--platform linux/amd64,linux/arm64 --push` (preserve the multi-arch
     logic from the deleted `bin/build-server.sh`, including semver+git tag
     pair if the registry needs git-tagged images).
4. **Env values**: `devops/helm/values-{staging,production}.yaml` — extend
   plan 01's skeletons with registry (`registry.vizerapp.cloud/oxidauth`),
   namespaces (`staging-oxidauth`, `production-oxidauth` — deploy.sh
   convention `<env>-<project>`), `RUST_LOG`, `ENVIRONMENT`, and secret
   material references (pepper, DATABASE_URL) — secrets via
   crypt-keeper/sops `.enc` like parkinglot
   (`devops/helm/values-production.yaml.enc`), plaintext ignored per plan 01
   `.gitignore`.

   **Caution:** the `devops/helm/values-{staging,production}.yaml` files copied
   in by plan 01 are byte-identical to project-template and carry another org's
   plaintext registry credentials (`registry.vizerapp.cloud`, user `drone-bot`,
   `statuswrangler` namespaces). They MUST be sanitized to oxidauth values
   before their first commit, and the `.enc` conversion below MUST NOT
   encrypt/commit the template files verbatim.

5. **`bin/deploy.sh`** (template, project-renamed): `kubectl config
   use-context freshbrewlabs` (confirm oxidauth's kube context; placeholder in
   PR), `helm upgrade --namespace $ENV-oxidauth --create-namespace --install
   $ENV-oxidauth-oxidauth . -f ../../../devops/helm/values-$ENV.yaml -f
   values-$ENV.yaml` run from `src/oxidauth/helm`; production confirmation
   prompt verbatim. **`bin/uninstall.sh`**: `helm uninstall
   $ENV-oxidauth-oxidauth`.
6. Chart `NOTES.txt`/README: trim web references or mark disabled.
7. Helm `values.yaml` api env: `MIGRATIONS_ENABLED: "false"` in-cluster
   default (jobs/hook run migrations; parkinglot pattern) — or keep `"true"`
   on staging + note in values comment. Decision recorded in PR.

## Verification

- `helm lint src/oxidauth/helm` + `helm template oxidauth
  src/oxidauth/helm -f devops/helm/values-staging.yaml` renders:
  deployment (image `registry.vizerapp.cloud/oxidauth/oxidauth-api:<ver>`),
  service, ingress, configmap; **no** web resources.
- `helm template` with `web.enabled=true` still renders (future-proof).
- `bin/build.sh` produces a manifest-list image with both
  `linux/amd64`+`linux/arm64` (or dry-run variant per plan 11 note).
- `bash -n bin/deploy.sh bin/uninstall.sh`; deploy run itself = **human gate**
  (out of migration scope).

## PR note

changelog `<id>-helm-deploy`. Status → `done` on merge (deploy execution is
operational, not migration, work).
