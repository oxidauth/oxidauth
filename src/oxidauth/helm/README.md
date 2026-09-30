# Helm Chart for oxidauth

Deploys the oxidauth API to Kubernetes. oxidauth ships no web frontend: the
`web-*` templates are kept behind `web.enabled: false` so a future admin-UI
stack re-enables them without chart surgery.

Real deploys go through the repo-root `bin/deploy.sh <env>`, which layers
`devops/helm/values-<env>.yaml` (encrypted org/env values) under this chart's
`values-<env>.yaml` (chart-shape values). The plaintext of every
`values-{staging,production}.yaml` here is gitignored; the committed form is
the `.enc` (crypt-keeper, team `freshbrewlabs` — `crypt-keeper decrypt` at
the repo root materializes the plaintext).

## Structure

```
helm/
├── Chart.yaml                 # Chart metadata
├── values.yaml                # Default values
├── values-staging.yaml        # Staging chart-shape values (encrypted sibling)
├── values-production.yaml     # Production chart-shape values (encrypted sibling)
└── templates/
    ├── _helpers.tpl           # Helper templates (oxidauth.* names)
    ├── secret.yaml            # api env Secret (<fullname>-env, stringData)
    ├── configmap.yaml         # api env ConfigMap (non-secret .Values.env keys)
    ├── deployment.yaml        # api deployment (secretKeyRef/configMapKeyRef split)
    ├── service.yaml           # api service
    ├── ingress.yaml           # api ingress (behind ingress.enabled; TLS/force-ssl/body-size)
    ├── web-deployment.yaml    # web deployment (behind web.enabled)
    ├── web-service.yaml       # web service (behind web.enabled)
    ├── web-ingress.yaml       # web ingress (behind web.enabled)
    └── image-pull-secret.yaml # Docker registry credentials
```

## Values

| Field | Description | Default |
|-------|-------------|---------|
| `namespace` | Kubernetes namespace (`<env>-oxidauth` by deploy convention) | `default` |
| `replicaCount` / `api.replicas` | api replica counts | `1` |
| `image.pullPolicy` | image pull policy (api and web) | `IfNotPresent` |
| `api.containerPort` | api container port; the server binds `PORT` (default 80) and `service.targetPort` follows this value | `80` |
| `api.image.repository` / `api.image.tag` | api image (full registry path in env values) | `oxidauth-api` / `latest` |
| `api.ingress.host` | api ingress hostname | `oxidauth-api.local` |
| `ingress.enabled` | render the api ingress | `true` |
| `ingress.forceSsl` | nginx `force-ssl-redirect` (http→https); staging/production enable via encrypted values | `false` |
| `ingress.tls` | standard `[]` of `{hosts, secretName}` blocks; the TLS Secret itself is cluster-side | `[]` |
| `ingress.annotations` | extra/override ingress annotations (wins over chart defaults) | `{}` |
| `envSecretKeys` | env keys routed to the Secret; empty = helper default `["DATABASE_URL", "READ_DATABASE_URL", "OXIDAUTH_USERNAME_PASSWORD_PEPPER"]` | `[]` |
| `web.enabled` | render the web half — no oxidauth web image exists | `false` |
| `registryCredentials.*` | private-registry pull secret (real values in the encrypted env files) | `""` |

### Environment variables (`env`)
`env` is split by `oxidauth.secretEnvKeys`: keys in the secret list render
into the api **Secret** (`<fullname>-env`, stringData, `helm.sh/resource-policy:
keep`); the rest render into the **ConfigMap**. Both are injected into the api
container, so every `env` key still lands as an env var:

| Variable | Notes |
|----------|-------|
| `PORT` | server bind port; pinned to `80` to match `api.containerPort` |
| `ENVIRONMENT` | standard |
| `RUST_LOG` | standard, **optional** — unset boots at `INFO` (stderr notice `RUST_LOG not set; defaulting to INFO`); a non-UTF-8 value **aborts boot** (terminal prints `Error: RustLog(NotUnicode(..))`; the error's Display text is `env var RUST_LOG not found: NotUnicode(...)`) |
| `DATABASE_URL` | **Secret-routed** (when set) — required at boot (empty default fails fast) |
| `MIGRATIONS_ENABLED` | `"false"` = boot skips migrations (a **missing** var is the hard error); schema changes run host-side via `bin/reset-db.sh` (sqlx-cli) |
| `MIGRATIONS_PATH` | informational — the dir copied into the production image (parkinglot parity); the server embeds migrations via `sqlx::migrate!` and does not read this var |
| `OXIDAUTH_USERNAME_PASSWORD_PEPPER` | **Secret-routed** (when set) — deliberately absent from the chart default; only the encrypted env values set it, so an unconfigured deploy fails auth fast instead of hashing with an empty pepper |

## Validate

```bash
helm lint .
helm template oxidauth . -f ../../../devops/helm/values-staging.yaml
```
