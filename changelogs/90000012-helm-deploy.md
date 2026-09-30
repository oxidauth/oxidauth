- [90000012](https://www.pivotaltracker.com/story/show/90000012) - Helm chart +
  deploy pipeline (migration plan 12).
    - `src/oxidauth/helm/` from the stack-template chart with
      `{{project-name}}` → `oxidauth`; `_helpers.tpl` keeps the `oxidauth.*`
      helper names. Template rework so the chart matches this server:
        - api image/port moved under `api.*` (`api.image.repository/tag`,
          `api.containerPort: 80`); `service.targetPort` follows
          `api.containerPort`. The server binds `$PORT` (plan item 2's PORT
          work, already live in `main.rs`) — chart `env.PORT` pins `"80"` so
          the bind and `containerPort` agree; the shipped
          service `80 → 8080` mismatch is gone.
        - web half DISABLED, NOT deleted: `web.enabled: false` gates
          `web-deployment/-service/-ingress.yaml` (the flag did not exist in
          the template); a future admin-UI stack re-enables without chart
          surgery. `web-deployment` image ref fixed to `web.image.*` (the
          template pointed it at the api image).
        - `configmap.yaml` now ranges over `.Values.env` — the template
          rendered only `ENVIRONMENT`/`RUST_LOG` while the Deployment
          configMapKeyRefs EVERY env key, so any extra key (DATABASE_URL…)
          booted pods against dangling keyRefs.
        - `ingress.yaml` honors `ingress.enabled` (the value existed, the
          gate did not).
        - `env` defaults: `MIGRATIONS_ENABLED: "false"` in-cluster (xlib
          `migrate()` treats any non-`"true"` as a skip — the hard error is a
          MISSING var — so boot is a no-op-migrate, verified in
          `src/xlib/postgres/src/lib.rs`; migrations run through
          sqlx-cli/`bin/reset-db.sh`), `MIGRATIONS_PATH` →
          `/etc/oxidauth/oxidauth-api/migrations` (where the image copies
          them), and `OXIDAUTH_USERNAME_PASSWORD_PEPPER` DELIBERATELY absent
          from chart defaults — an empty default would silently hash with an
          empty pepper; only the encrypted env values set it.
    - `oxidauth-api/build/{Dockerfile,build.sh}` (multi-stage
      `rust-base` → `debian`, `--release --bin oxidauth-api`, migrations dir
      copied to `/etc/oxidauth/oxidauth-api/migrations` for in-container
      sqlx-cli despite `migrate!` embedding, `EXPOSE 80`); the pre-split
      `oxidauth-api/Dockerfile` (single-stage `tmp/$TARGETPLATFORM` stager
      from the deleted `bin/build-server.sh`) is DELETED — no live refs.
      Root `.dockerignore` added (workspace-root build context must not drag
      `target/`, `.env`, secrets, `.enc`). `build.sh` resolves the repo root
      from its own location (runs from anywhere), tags
      `semver + :latest + git-sha` (pair preserved from the old
      build-server.sh), reuses the `oxidauth-builder` docker-container
      builder, and **defaults to a DRY multi-arch build** (`--output
      type=oci` tarball — the docker-container driver cannot export a
      manifest list to the local store): pushing to
      `registry.vizerapp.cloud/oxidauth` requires `PUSH=1`, an explicit
      human gate.
    - `bin/deploy.sh` + `bin/uninstall.sh` (parkinglot pattern): namespace
      `$ENV-oxidauth`, release `$ENV-oxidauth-oxidauth`, values layered
      devops-first (`-f ../../../devops/helm/values-$ENV.yaml -f
      values-$ENV.yaml` from `src/oxidauth/helm`), production confirmation
      prompt verbatim. **kube context = `fbl-k3s`** (the plan's
      `freshbrewlabs` placeholder): every freshbrewlabs project deploys to
      that k3s context — `parkinglot/bin/deploy.sh` proves it. Deploy
      itself remains a human gate; `uninstall.sh` also pins the context so a
      stale current-context can't aim the uninstall at the wrong cluster.
    - `devops/helm/values-{staging,production}.yaml` SANITIZED — the
      plan-01 copies were byte-identical project-template output carrying
      another project's values (`staging|production-statuswrangler`
      namespaces, `project: statuswrangler`, a stray `defaultReplicas` the
      chart never reads, statuswrangler affinity-comment keys). Now:
      `namespace: staging-oxidauth|production-oxidauth`,
      `api.image.repository: registry.vizerapp.cloud/oxidauth/oxidauth-api`
      (tag = crate version), `registryCredentials.project: oxidauth`
      (drone-bot bot creds kept — org-wide registry bot, same credentials
      parkinglot deploys with, inside the encrypted file), `env` =
      `RUST_LOG`/`ENVIRONMENT` + `DATABASE_URL` and
      `OXIDAUTH_USERNAME_PASSWORD_PEPPER` as marked `CHANGE_ME`
      placeholders (real values unknown until the in-cluster DB exists).
      All four values files (devops + chart `values-{staging,production}.yaml`)
      encrypted with crypt-keeper (keybase, team `freshbrewlabs`) —
      `.enc` committed-form, plaintext gitignored per plan-01 `.gitignore`;
      round-trip decrypt verified.
