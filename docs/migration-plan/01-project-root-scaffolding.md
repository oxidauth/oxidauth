# 01 — Project root scaffolding

**Status**: `done` (2026-09-29: implemented, reviewed, adjudicated — 3 fixes applied, 2 skips reasoned; gates green)
**Depends on**: —
**Risk**: low (no Rust code touched)

## Goal

Bring the project root file set to parity with `project-template` / parkinglot
before any structural moves, so every later plan lands on conventional ground.

## Changes

1. **`.gitignore`** — replace the 6-line file with the template set
   (parkinglot convention):
   - `target/`, `**/target/`, `dist/`
   - `*.env` with `!example.env` exception (keep current `.env` ignore)
   - IDE: `.vscode/`, `.idea/`; OS: `.DS_Store`
   - `devops/helm/values-staging.yaml`, `devops/helm/values-production.yaml`
     (parkinglot commits these as `.enc` via crypt-keeper/sops — see plan 12)
   - `*.log`, keep `**/.gitkeep`
2. **`crypt-keeper.toml`** — new file: `team = "freshbrewlabs"` (template/parkinglot
   literal; change the value if a separate oxidauth escrow team is desired — flag in PR).
3. **`secrets/.gitkeep`** — new empty file.
4. **`example.env`** — supersede current keys with the union of template + parkinglot
   + oxidauth-specific. Final content:
   ```
   # General
   ENVIRONMENT=local
   RUST_LOG=INFO
   DOCKER_PLATFORM=linux/amd64
   RUST_DEV_IMAGE_VERSION="v1.98.0"

   # Database
   POSTGRES_HOST=localhost
   POSTGRES_PORT=5434
   POSTGRES_USER=oxidauth
   POSTGRES_PASSWORD=oxidauth
   POSTGRES_DATABASE=oxidauth
   DATABASE_URL=postgres://oxidauth:oxidauth@127.0.0.1:5434/oxidauth
   READ_DATABASE_URL=postgres://oxidauth:oxidauth@127.0.0.1:5434/oxidauth
   MIGRATIONS_ENABLED=true

   # Oxidauth
   OXIDAUTH_USERNAME_PASSWORD_PEPPER=pepper
   ```
   Keep `DOCKER_PLATFORM=linux/amd64` if your dev box targets amd64; template
   default is arm64. Bump `RUST_DEV_IMAGE_VERSION` v1.76.0 → v1.98.0 (template value).
5. **`devops/` tree** — create from project-template:
   - `devops/postgres/init.sh` (verbatim from template: loops `$SERVICES`,
     creates DB + user, grants on public schema)
   - `devops/helm/values-staging.yaml`, `devops/helm/values-production.yaml`
     (copy template shape; registry/namespace values filled in plan 12)
   - Skip `devops/docker/{leptos,stylance,tailwind}` — no web layer in this stack.
6. **Root `README.md`** — the repo has none. Add one in project-template shape:
   title, 8-layer architecture list, Getting Started (docker compose up -d,
   cargo build, bin scripts), "Available Stacks" section listing `oxidauth`.
   (Full prose pass is plan 15; create the skeleton now so the file exists.)
7. **`SECURITY_REPORT.md`** → `docs/SECURITY_REPORT.md` (root stays scaffold-only).
8. Keep as-is (no change): `rustfmt.toml` (repo-specific; templates ship none,
   parkinglot/aka have none — keeping it is an accepted deviation), `rfcs/`,
   `docs/`, `changelogs/`, `.github/PULL_REQUEST_TEMPLATE.md`.

## Verification

- `docker compose config` still parses old compose (untouched here).
- `cargo check --workspace` unaffected — zero Rust changes.
- `cat example.env` lists every var later plans (05/06/10/13) will require;
  copy to `.env` locally.

## PR note

Add `changelogs/<id>-root-scaffolding.md` per repo changelog convention; set
this file's Status → `done` and update the README table.

## Review feedback

Reviewed 2026-09-29 against working tree + `/Users/georgewheeler/dev/freshbrewlabs/project-template` and `…/parkinglot`.

### Checklist (plan bullet → status)

| # | Bullet | Status | Evidence |
|---|--------|--------|----------|
| 1 | `.gitignore` template set | **met** | All planned entries present: `target/`+`**/target/`+`dist/` (.gitignore:2-4), `.env`/`*.env`/`!example.env` (:8-10), `.vscode/` `.idea/` `.DS_Store` (:13-19), helm values ignores (:27-28), `!**/.gitkeep` (:24), `*.log` (:31). `git check-ignore example.env` → clean; `git ls-files --others --ignored --exclude-standard` lists only `devops/helm/values-{staging,production}.yaml` + pre-existing `tmp/**` among new paths → `example.env` and `secrets/.gitkeep` survive, only the two values files are ignored (intended). Kept `tmp/**` (:5) is not in the plan's bullet-1 list but is correct (existing `tmp/linux/*/oxidauth-http` artifacts stay ignored) and disclosed in the changelog. |
| 2 | `crypt-keeper.toml` | **met** | `team = "freshbrewlabs"` — template literal (template file also lacks trailing newline; repo adds one — irrelevant). "Flag in PR if separate oxidauth team" not verifiable from tree. |
| 3 | `secrets/.gitkeep` | **met** | Exists, 0 bytes, not ignored (see bullet-1 evidence). |
| 4 | `example.env` union | **met (byte-exact to plan block)** | `diff` of plan's fenced block vs `example.env` → identical, incl. amd64 keep and `v1.98.0` bump. But see DEFECT-1: the localhost `DATABASE_URL` breaks the containerized flow. No Rust-code consumer exists for `POSTGRES_HOST`/`POSTGRES_PORT`/`MIGRATIONS_ENABLED` yet, so those are inert today. |
| 5 | `devops/` tree | **met** | `diff devops/postgres/init.sh` vs template → SAME, mode `-rwxr-xr-x` preserved. Both helm values files byte-SAME to template (incl. third-party values — DEFECT-4). `devops/docker/*` correctly skipped; `devops/` contains only `postgres/` + `helm/`. Note: `init.sh` is inert until plan 10 mounts it (compose untouched — per plan). |
| 6 | Root `README.md` | **met with defects** | Skeleton exists: title, 8-layer list (verbatim template text), Getting Started, "Available Stacks" listing oxidauth. Added `## Development` section is unauthorized drift (harmless). See DEFECT-1 and DEFECT-2. |
| 7 | `SECURITY_REPORT.md` → `docs/` | **met** | Root copy gone; `docs/SECURITY_REPORT.md` present, not ignored. Was never tracked (`git ls-tree HEAD` has no root entry), so the move has no git-visible delete — expected. No stale refs to the root path anywhere (`grep -rn SECURITY_REPORT` hits only changelog/plan text; plan 15 adds the README link). |
| 8 | Keep-as-is files | **met** | `rustfmt.toml`, `.github/PULL_REQUEST_TEMPLATE.md`, `rfcs/`, `docs/`, `changelogs/` untouched (mtime = June; `git status` shows no changes to them). |
| — | PR note | **partial** | Changelog `changelogs/90000001-root-scaffolding.md` added, format matches `changelogs/example.md`. BUT Status still `in-progress` (this file, line 3) and `docs/migration-plan/README.md:90` table row still `in-progress` — the plan's own PR note (line 74) requires both flips. DEFECT-3. |

### Defects

**DEFECT-1 — should-fix** — `example.env:13-14` + `README.md:21-24`: Getting Started instructs `cp example.env .env` → `docker compose up -d`, but `docker-compose.yml` passes `.env` to the `oxidauth-api` container via `env_file`, so the containerized API receives `DATABASE_URL=postgres://oxidauth:oxidauth@127.0.0.1:5434/oxidauth`. Inside the bridge-networked container, `127.0.0.1:5434` is the API container itself, not the postgres container (which is reachable as `postgres.oxidauth.localhost`, or `postgres:5432`); `oxidauth-postgres/src/lib.rs:42` (`env::var("DATABASE_URL")`) then fails to connect. Pre-patch, the var was absent (loud `MissingEnvVar`-style failure); now the documented happy path silently points at the wrong host until plan 10's compose `environment:` override lands. Fix now by either adding `DATABASE_URL: postgresql://oxidauth:oxidauth@postgres:5432/oxidauth` to the `oxidauth-api` `environment:` block, or adding a README/compose note that host-oriented `DATABASE_URL` needs the container override in the interim.

**DEFECT-2 — nit** — `README.md:37-38`: "Available Stacks" parenthetical lists `(kernel, repository, postgres, http, cli, import-export, permission crates)` — 7 of the 11 workspace members in `Cargo.toml:6-17`; omits `oxidauth-rs`, `oxidauth-seed`, `oxidauth-usecases`, `oxidauth-telemetry`. Factual drift vs repo reality (the 8-layer list above it is verbatim template and fine; this list was authored here). One-line fix: complete the list or write "all `oxidauth-*` workspace crates".

**DEFECT-3 — should-fix** — `01-project-root-scaffolding.md:3` + `docs/migration-plan/README.md:90`: PR note (this file, line 71-74) explicitly requires Status → `done` and the README table update; both still say `in-progress` (and the table shows plan 02 as `reviewed` while its dependency 01 is not done — workflow bookkeeping is inverted until 01 flips). Flip both when this feedback is addressed.

**DEFECT-4 — nit** — `devops/helm/values-staging.yaml:1,6,9` (+ values-production mirror): files copied byte-identical from template carry **another org's live-looking registry credential** (`password: [REDACTED-REGISTRY-SECRET]`, `registry.vizerapp.cloud`, `statuswrangler` namespaces). Plan authorized "copy template shape; filled in plan 12" and `.gitignore:27-28` keeps them uncommitted, so nothing leaks into history — but they persist in every clone/worktree, and crypt-keeper in plan 12 must encrypt **oxidauth-substituted** files, not these. Recommend a one-line `# TODO plan 12: replace placeholders before .enc` comment in both files so a forgotten swap can't slip through. Related: the ignore rules are path-scoped to `devops/helm/` while template/parkinglot use `**/values-{staging,beta,production}.yaml` globs — `src/oxidauth/helm/` values files arriving with plan 12 will NOT be ignored and could be committed in plaintext (DEFECT-5).

**DEFECT-5 — nit** — `.gitignore:27-28` (and set-wide vs template/parkinglot): the plan's bullet-1 set is narrower than both model repos — drops `*.env.secrets` / `*.env.development` / `*.env.staging` / `*.env.production` (parkinglot keeps all four) and swaps the `**/values-*.yaml` globs (also covering a hypothetical `values-beta.yaml` and chart-local values) for two fixed paths. Harmless today; becomes a plaintext-secret hazard exactly where plan 12 places chart values. Suggest restoring the `**/values-{staging,production}.yaml` globs (parkinglot convention the plan itself cites).

### Verified non-issues

- `example.env` matches the plan's spec block byte-for-byte; `DOCKER_PLATFORM=linux/amd64` kept and `RUST_DEV_IMAGE_VERSION="v1.98.0"` match plan/template/parkinglot (`parkinglot/example.env:11`).
- Port consistency: `POSTGRES_PORT=5434` / `DATABASE_URL` host port vs `docker-compose.yml` mapping `'5434:5432'` → correct for host-side `cargo run`; postgres image derives user/DB `oxidauth` from `POSTGRES_USER` (POSTGRES_DATABASE is unread by the image but was already inert pre-patch).
- `RUST_DEV_IMAGE_VERSION` bump only feeds `registry.vizerapp.cloud/lib/rust-dev:$TARGETVERSION` (`dev.Dockerfile:4`); tag existence there is unverifiable offline — run one `docker compose build` before merge as smoke proof.
- `tmp/**` retention, `!/**/.gitkeep` broadening to `**`, and the added `## Development` README section are the only content drift from plan/template; all benign.

**Verdict: fixes-needed** — 0 must-fix, 2 should-fix (DEFECT-1, DEFECT-3), 3 nit (DEFECT-2, DEFECT-4, DEFECT-5).

---

### Adjudication (worker)

- **DEFECT-1 — IMPLEMENTED**: `docker-compose.yml:16-17` — `oxidauth-api`
  `environment:` block now sets `DATABASE_URL` to the container-network URL
  (`postgresql://` on host `postgres`, port `5432`, oxidauth user/password,
  with a `# TODO(plan 10)` removal comment); service env overrides `env_file`,
  so the containerized API no longer points at itself. `example.env` keeps the
  host-side `127.0.0.1:5434` URLs for host tooling. Plan 10 replaces this
  compose file wholesale.
- **DEFECT-2 — SKIPPED**: plan 15 is an authorized full prose rewrite of
  `README.md` (incl. Available Stacks); fixing the crate list now would be
  churn. (Verdict directed by orchestrator.)
- **DEFECT-3 — SKIPPED**: Status→`done` (this file, line 3) and the
  `docs/migration-plan/README.md` table row are orchestrator-owned bookkeeping;
  flipped at plan close, not by the worker. (Verdict directed by orchestrator.)
- **DEFECT-4 — IMPLEMENTED**: `docs/migration-plan/12-helm-deploy.md:54-59` —
  bolded caution under Changes item 4 (the plan-12 section governing
  `devops/helm/values-{staging,production}.yaml`; orchestrator said "§5" but
  item 5 is `bin/deploy.sh`): template-copied values files carry another org's
  plaintext registry credentials, MUST be sanitized before first commit, and
  the `.enc` conversion MUST NOT commit them verbatim.
- **DEFECT-5 — IMPLEMENTED**: `.gitignore:10-13` restores
  `*.env.secrets` / `*.env.development` / `*.env.staging` /
  `*.env.production`; `.gitignore:31-32` replaces the two fixed devops/helm
  paths with parkinglot-style `**/values-staging.yaml` +
  `**/values-production.yaml`. Verified: `git check-ignore example.env` → no
  match; `git check-ignore devops/helm/values-staging.yaml` → still ignored;
  `git status --short` unchanged for the changeset; `git ls-files |
  git check-ignore --stdin` → no tracked file newly ignored.
