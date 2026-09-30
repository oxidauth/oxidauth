- [OXA-000005](https://www.pivotaltracker.com/story/show/OXA-000005) - forgot_password stops being an anonymous TOTP oracle: route gated with `oxidauth:auth:forgot_password`, failure paths blinded (Step 1 hotfix only)
    - `POST /api/v1/auth/username_password/forgot_password` previously had no
      gate between the socket and the use case: any anonymous caller posting
      `{"user_id": "<uuid>"}` received the target user's live, current-window
      TOTP code (valid 600 s, the same code `update_password` accepts as the
      sole credential) and every refresh token of that user was deleted. The
      route now carries the established handler gate — `ExtractJwt` +
      `ExtractEntitlements` + `parse_and_validate` against
      `PERMISSION = "oxidauth:auth:forgot_password"` — the same shape as
      `list_all_authorities`/`create_invitation`.
    - **Consumer-visible deltas (the point of the hotfix):**
      - **Anonymous → refused.** No/invalid bearer → bodyless **401** from the
        jwt extractor; the use case (code mint + token wipe) never runs, so
        the anonymous revoke-and-takeover chain is closed.
      - **Authenticated but unpermitted → 401 `{"success":false}`**, the same
        gate envelope as every other guarded route.
      - **Failure paths blinded.** Unknown user, user without a TOTP secret,
        and refresh-token-delete failures used to answer **400** with the raw
        sqlx `Display` embedded — an existence/enrollment oracle that also
        leaked database internals. They now log the real error via
        `tracing::warn!` and answer the **generic success `{"success":true}`**
        (no payload, no errors field), so callers cannot tell the paths apart.
      - **Gated callers unchanged.** A caller whose entitlements satisfy the
        challenge (including the seeded admin `oxidauth:**:**` wildcard)
        still gets the byte-identical `200 {"success":true,"payload":{"code":…}}`
        — the support-desk workflow of triggering and delivering the code
        out-of-band is the flow this step preserves. **BREAKING for callers
        that exploited or depended on the anonymous leak**; the SDK
        (`username_password_forgot_password`) always sent a bearer and keeps
        working *iff* its credentials hold the new permission — see release
        note requirement below.
    - **Permissions seed:** `oxidauth-services::bootstrap` gains
      `FORGOT_PASSWORD_PERMISSION = "oxidauth:auth:forgot_password"` and
      seeds it into the permissions tree (find-or-create, same shape as
      `TOTP_VALIDATE_PERMISSION`), so the string is grantable on fresh
      installs; the seeded admin role needs nothing extra because its
      `oxidauth:**:**` wildcard already answers the challenge. The bootstrap
      only runs on an empty database, so **upgrading deployments must add the
      permission string to the tree (permissions CRUD/migration) and grant it
      to support roles** — adding a permission string always requires that
      seed step. `hurl/tests/permissions.hurl` seed-list comment updated.
    - **Tests:** no pre-existing test asserted anonymous success (the route
      had zero route-level tests; the wiremock contract suite always sent a
      bearer, so nothing to flip there — recorded explicitly). Four new
      handler tests in the axum handler module pin: anonymous → 401 with the
      service never invoked; valid jwt without the permission → 401 envelope,
      service never invoked; permitted + admin-wildcard callers → the exact
      pre-fix success envelope; use-case failure → generic `{"success":true}`
      with no error text on the wire. Bootstrap sequence assertions updated
      for the inserted seed step. The service-test comment that claimed "the
      auth gate lives in the calling route/permission layer" — false until
      this change — now references the real route gate. The
      `oxidauth-rs` turbofish `BUG(pinned)` is outside this change (removed
      by OXA-000035).
    - **Explicitly NOT in this step:** no webhook/email dispatch, no random
      single-use stored code, no revocation rework (Step 2, Tier 5), no
      deprecation (Step 3). `update_password` keeps its (flagged) anonymous
      reachability; with the leak closed, its code gate is no longer
      bypassable via this endpoint.
