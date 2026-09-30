- [90000016](https://www.pivotaltracker.com/story/show/90000016) - helm chart
  security hardening (post-review follow-up to migration plan 12; found by the
  final security review of the migration arc, review approve 0-must-fix with
  these two remediations dispatched separately).
    - **api env secrets moved out of the ConfigMap.** New `secret.yaml` renders
      `<fullname>-env` (Opaque, `stringData`, `helm.sh/resource-policy: keep`)
      for the env keys produced by the new `oxidauth.secretEnvKeys` helper —
      default set `DATABASE_URL`, `READ_DATABASE_URL`,
      `OXIDAUTH_USERNAME_PASSWORD_PEPPER`, overridable wholesale via
      `values.envSecretKeys`. The helper returns a JSON OBJECT (helm 4's
      `fromJson` only decodes objects and the sprig list-membership form
      broke, so the templates use dict + `hasKey`). `configmap.yaml` renders
      the complement, `deployment.yaml` splits its env list `secretKeyRef` vs
      `configMapKeyRef` on the same predicate — Secret and ConfigMap together
      still cover exactly `.Values.env`, no dangling refs by construction.
      Only keys ACTUALLY present in `.Values.env` land in the Secret: the
      chart default ships just `DATABASE_URL: ""`, and the pepper is never
      fabricated (still set only in the encrypted devops values).
      Staging/production renders verified: `DATABASE_URL` +
      `OXIDAUTH_USERNAME_PASSWORD_PEPPER` moved ConfigMap→Secret; non-secret
      keys (PORT/ENVIRONMENT/RUST_LOG/MIGRATIONS_*) byte-identical.
    - **api ingress TLS + body-size posture.** `ingress.yaml` now renders
      `spec.tls` from `ingress.tls` (standard `{hosts, secretName}` blocks),
      adds `nginx.ingress.kubernetes.io/force-ssl-redirect: "true"` when
      `ingress.forceSsl`, and the proxy-body-size default drops from `"0"`
      (UNLIMITED upload) to `"1m"`. All three overridable via
      `ingress.annotations` (user map mergeOverwrites the chart defaults).
      Defaults stay render-compatible: `forceSsl: false` + `tls: []` ship no
      TLS/redirect on plain chart installs. The encrypted devops
      staging/production values set `forceSsl: true` with a `tls: []`
      placeholder + comment — the TLS Secret itself is cluster-side wiring
      (cert-manager issuer or a manually created Secret in the release
      namespace). All four values `.enc` re-encrypted via crypt-keeper
      (team `freshbrewlabs`), round-trip decrypt byte-compared.
